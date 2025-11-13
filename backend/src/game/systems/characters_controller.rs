use std::collections::HashMap;

use shared::{
    computations,
    constants::ARMOR_FACTOR,
    data::{
        character::{CharacterId, CharacterSpecs, CharacterState},
        character_status::{StatusSpecs, StatusState},
        item::SkillRange,
        skill::{DamageType, RestoreType, SkillType},
        stat_effect::Modifier,
    },
};

use crate::game::{
    data::event::{EventsQueue, GameEvent, HitEvent},
    utils::rng::Rollable,
};

pub type Target<'a> = (CharacterId, (&'a CharacterSpecs, &'a mut CharacterState));

#[allow(clippy::too_many_arguments)]
pub fn attack_character(
    events_queue: &mut EventsQueue,
    target: &mut Target,
    attacker: CharacterId,
    damage: HashMap<DamageType, f64>,
    skill_type: SkillType,
    range: SkillRange,
    is_crit: bool,
    is_triggered: bool,
    ignore_armor: bool,
) {
    let (target_id, (target_specs, target_state)) = target;

    let mut amount: f64 = damage
        .iter()
        .map(|(damage_type, amount)| {
            compute_damage(
                target_specs,
                *amount,
                *damage_type,
                skill_type,
                ignore_armor,
            )
        })
        .sum();

    let is_blocked = target_specs.block.roll()
        & match skill_type {
            SkillType::Attack => true,
            SkillType::Spell => target_specs.block_spell.roll(),
        };

    if is_blocked {
        amount *= target_specs.block_damage as f64 * 0.01;
    }

    let is_hurt = amount > 0.0;

    if is_blocked {
        target_state.just_blocked = true;
    }

    if is_hurt && is_crit {
        target_state.just_hurt_crit = true;
    }

    if is_hurt {
        damage_character(
            target_specs,
            &mut target_state.life,
            &mut target_state.mana,
            amount,
        );
        target_state.just_hurt = true;
    }

    events_queue.register_event(GameEvent::Hit(HitEvent {
        source: attacker,
        target: *target_id,
        skill_type,
        range,
        damage,
        is_crit,
        is_blocked,
        is_hurt,
        is_triggered,
    }));
}

pub fn damage_character(
    character_specs: &CharacterSpecs,
    life: &mut f64,
    mana: &mut f64,
    amount: f64,
) {
    if amount <= 0.0 {
        return;
    }

    let take_from_mana = mana
        .min(amount * (character_specs.take_from_mana_before_life as f64 * 0.01).clamp(0.0, 1.0));
    let take_from_life = amount - take_from_mana;

    *mana -= take_from_mana;
    *life -= take_from_life;
}

pub fn restore_character(
    target: &mut Target,
    restore_type: RestoreType,
    amount: f64,
    modifier: Modifier,
) -> bool {
    let (_, (target_specs, target_state)) = target;

    if amount <= 0.0 || !target_state.is_alive {
        return false;
    }

    let factor = match modifier {
        Modifier::Multiplier => match restore_type {
            RestoreType::Life => target_specs.max_life * 0.01,
            RestoreType::Mana => target_specs.max_mana * 0.01,
        },
        Modifier::Flat => 1.0,
    };

    match restore_type {
        RestoreType::Life => {
            if target_state.life < target_specs.max_life {
                target_state.life += amount * factor;
                true
            } else {
                false
            }
        }
        RestoreType::Mana => {
            if target_state.mana < target_specs.max_mana {
                target_state.mana += amount * factor;
                true
            } else {
                false
            }
        }
    }
}

pub fn resuscitate_character(target: &mut Target) -> bool {
    let (_, (target_specs, target_state)) = target;
    if target_state.is_alive {
        return false;
    }

    target_state.is_alive = true;
    target_state.life = target_specs.max_life;

    target_state
        .statuses
        .unique_statuses
        .retain(|_, (_, status_state)| status_state.duration.is_none());
    target_state
        .statuses
        .cumulative_statuses
        .retain(|(_, status_state)| status_state.duration.is_none());

    true
}

pub fn apply_status(
    target: &mut Target,
    status_specs: &StatusSpecs,
    skill_type: SkillType,
    value: f64,
    duration: Option<f64>,
    cumulate: bool,
) -> bool {
    let (_, (target_specs, target_state)) = target;

    if duration.unwrap_or(1.0) <= 0.0 || !target_state.is_alive {
        return false;
    }

    // Long duration are considered as forever
    let duration = match duration {
        Some(duration) if duration > 9999.0f64 => None,
        _ => duration,
    };

    let value = match status_specs {
        StatusSpecs::DamageOverTime {
            damage_type,
            ignore_armor,
        } => compute_damage(target_specs, value, *damage_type, skill_type, *ignore_armor),
        _ => value,
    };

    let mut applied = true;
    if cumulate {
        target_state.statuses.cumulative_statuses.push((
            status_specs.clone(),
            StatusState {
                value,
                duration,
                cumulate,
            },
        ));
    } else {
        target_state
            .statuses
            .unique_statuses
            .entry(status_specs.into())
            .and_modify(|(cur_status_specs, cur_status_state)| {
                if value * duration.unwrap_or(10_000.0)
                    > cur_status_state.value * cur_status_state.duration.unwrap_or(10_000.0)
                {
                    cur_status_state.value = value;
                    cur_status_state.duration = duration;
                    *cur_status_specs = status_specs.clone();
                } else {
                    applied = false;
                }
            })
            .or_insert((
                status_specs.clone(),
                StatusState {
                    value,
                    duration,
                    cumulate,
                },
            ));
    }

    if !applied {
        return false;
    }

    if let StatusSpecs::StatModifier { .. } | StatusSpecs::Trigger { .. } = status_specs {
        target_state.dirty_specs = true;
    }

    true
}

pub fn compute_damage(
    target_specs: &CharacterSpecs,
    amount: f64,
    damage_type: DamageType,
    skill_type: SkillType,
    ignore_armor: bool,
) -> f64 {
    let amount = (1.0
        - target_specs
            .damage_resistance
            .get(&(skill_type, damage_type))
            .cloned()
            .unwrap_or(0.0)
            * 0.01)
        .max(0.0)
        * amount;

    if ignore_armor {
        amount
    } else {
        decrease_damage_from_armor(target_specs, amount, damage_type)
    }
}

fn decrease_damage_from_armor(
    target_specs: &CharacterSpecs,
    amount: f64,
    damage_type: DamageType,
) -> f64 {
    amount
        * (1.0
            - computations::diminishing(
                target_specs
                    .armor
                    .get(&damage_type)
                    .cloned()
                    .unwrap_or_default(),
                ARMOR_FACTOR,
            ))
}
