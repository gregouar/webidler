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
    data::event::{EventsQueue, GameEvent, HitEvent, StatusEvent},
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
) {
    let (target_id, (target_specs, target_state)) = target;

    let is_blocked = target_specs.block.roll()
        & match skill_type {
            SkillType::Attack => true,
            SkillType::Spell => target_specs.block_spell.roll(),
            SkillType::Blessing | SkillType::Curse => false,
        };

    let is_hurt = damage_character(
        target_specs,
        &mut target_state.life,
        &mut target_state.mana,
        &damage,
        skill_type,
        is_blocked,
    ) > 0.0;

    if is_blocked {
        target_state.just_blocked = true;
    }

    if is_hurt {
        target_state.just_hurt = true;
        if is_crit {
            target_state.just_hurt_crit = true;
        }
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
    damage: &HashMap<DamageType, f64>,
    skill_type: SkillType,
    is_blocked: bool,
) -> f64 {
    let amount = damage
        .iter()
        .map(|(damage_type, amount)| {
            compute_damage(
                character_specs,
                *amount,
                *damage_type,
                skill_type,
                is_blocked,
            )
        })
        .sum();

    if amount <= 0.0 {
        return 0.0;
    }

    let take_from_mana = mana
        .min(amount * (character_specs.take_from_mana_before_life as f64 * 0.01).clamp(0.0, 1.0));
    let take_from_life = amount - take_from_mana;

    *mana -= take_from_mana;
    *life -= take_from_life;

    amount
}

fn compute_damage(
    character_specs: &CharacterSpecs,
    amount: f64,
    damage_type: DamageType,
    skill_type: SkillType,
    is_blocked: bool,
) -> f64 {
    let resistance_factor = (1.0
        - character_specs
            .damage_resistance
            .get(&(skill_type, damage_type))
            .cloned()
            .unwrap_or(0.0)
            * 0.01)
        .max(0.0);

    let armor_factor = (1.0
        - computations::diminishing(
            character_specs
                .armor
                .get(&damage_type)
                .cloned()
                .unwrap_or_default(),
            ARMOR_FACTOR,
        ))
    .max(0.0);

    let block_factor = if is_blocked {
        character_specs.block_damage as f64 * 0.01
    } else {
        1.0
    };

    resistance_factor * armor_factor * block_factor * amount
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

pub fn should_apply_status(
    target: &Target,
    status_specs: &StatusSpecs,
    value: f64,
    duration: Option<f64>,
    cumulate: bool,
    replace_on_value_only: bool,
) -> bool {
    let (_, (_, target_state)) = target;

    if duration.unwrap_or(1.0) <= 0.0 || !target_state.is_alive {
        return false;
    }

    match status_specs {
        StatusSpecs::DamageOverTime { .. } | StatusSpecs::StatModifier { .. } => {
            if value <= 0.0 {
                return false;
            }
        }
        StatusSpecs::Trigger(_) | StatusSpecs::Stun => {}
    }

    if cumulate {
        return true;
    }

    if let Some((_, cur_status_state)) = target_state
        .statuses
        .unique_statuses
        .get(&status_specs.into())
    {
        return compute_effect_weight(value, duration, replace_on_value_only)
            > compute_effect_weight(
                cur_status_state.value,
                cur_status_state.duration,
                replace_on_value_only,
            );
    }

    true
}

#[allow(clippy::too_many_arguments)]
pub fn apply_status(
    events_queue: &mut EventsQueue,
    target: &mut Target,
    attacker: CharacterId,
    status_specs: &StatusSpecs,
    skill_type: SkillType,
    value: f64,
    duration: Option<f64>,
    cumulate: bool,
    is_triggered: bool,
) -> bool {
    let (target_id, (_, target_state)) = target;

    if duration.unwrap_or(1.0) <= 0.0 || !target_state.is_alive {
        return false;
    }

    match status_specs {
        StatusSpecs::DamageOverTime { .. } | StatusSpecs::StatModifier { .. } => {
            if value <= 0.0 {
                return false;
            }
        }
        StatusSpecs::Trigger(_) | StatusSpecs::Stun => {}
    }

    // Long duration are considered as forever
    let duration = match duration {
        Some(duration) if duration > 9999.0f64 => None,
        _ => duration,
    };

    let mut new_status_specs = status_specs.clone();
    if let StatusSpecs::Trigger(ref mut trigger_specs) = new_status_specs {
        trigger_specs.triggered_effect.owner = Some(attacker);
    }

    let mut applied = true;
    if cumulate {
        target_state.statuses.cumulative_statuses.push((
            new_status_specs,
            StatusState {
                value,
                duration,
                cumulate,
                skill_type,
            },
        ));
    } else {
        target_state
            .statuses
            .unique_statuses
            .entry(status_specs.into())
            .and_modify(|(cur_status_specs, cur_status_state)| {
                if compute_effect_weight(value, duration, false)
                    > compute_effect_weight(
                        cur_status_state.value,
                        cur_status_state.duration,
                        false,
                    )
                {
                    cur_status_state.value = value;
                    cur_status_state.duration = duration;
                    *cur_status_specs = new_status_specs.clone();
                } else {
                    applied = false;
                }
            })
            .or_insert((
                new_status_specs,
                StatusState {
                    value,
                    duration,
                    cumulate,
                    skill_type,
                },
            ));
    }

    if !applied {
        return false;
    }

    events_queue.register_event(GameEvent::StatusApplied(StatusEvent {
        source: attacker,
        target: *target_id,
        skill_type,
        status_type: status_specs.into(),
        value,
        duration,
        is_triggered,
    }));

    if let StatusSpecs::StatModifier { .. } | StatusSpecs::Trigger { .. } = status_specs {
        target_state.dirty_specs = true;
    }

    true
}

fn compute_effect_weight(value: f64, duration: Option<f64>, value_only: bool) -> f64 {
    let value = value + 1.0;
    if value_only {
        if duration.unwrap_or(1.0) < 0.2 {
            value * 0.1
        } else {
            value
        }
    } else {
        value * duration.unwrap_or(10_000.0)
    }
}
