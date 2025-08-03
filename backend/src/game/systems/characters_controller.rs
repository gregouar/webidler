use std::collections::HashMap;

use shared::data::{
    character::{CharacterId, CharacterSpecs, CharacterState},
    character_status::{StatusState, StatusType},
    item::SkillRange,
    skill::{DamageType, RestoreType, SkillType},
};

use crate::game::{
    data::event::{EventsQueue, GameEvent, HitEvent},
    utils::{increase_factors, rng},
};

pub type Target<'a> = (CharacterId, (&'a CharacterSpecs, &'a mut CharacterState));

pub fn attack_character(
    events_queue: &mut EventsQueue,
    target: &mut Target,
    attacker: CharacterId,
    damage: HashMap<DamageType, f64>,
    skill_type: SkillType,
    range: SkillRange,
    is_crit: bool,
) {
    let (target_id, (target_specs, target_state)) = target;

    let amount: f64 = damage
        .iter()
        .map(|(damage_type, amount)| {
            compute_damage(target_specs, *amount, *damage_type, skill_type, false)
        })
        .sum();

    let is_blocked = skill_type == SkillType::Attack
        && rng::random_range(0.0..=1.0).unwrap_or(1.0) <= target_specs.block;

    let is_hurt = amount > 0.0 && !is_blocked;

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

    let take_from_mana =
        mana.min(amount * (character_specs.take_from_mana_before_life as f64).clamp(0.0, 1.0));
    let take_from_life = amount - take_from_mana;

    *mana -= take_from_mana;
    *life -= take_from_life;
}

pub fn restore_character(target: &mut Target, restore_type: RestoreType, amount: f64) {
    let (_, (_, target_state)) = target;

    if amount <= 0.0 {
        return;
    }

    if target_state.is_alive {
        match restore_type {
            RestoreType::Life => target_state.life += amount,
            RestoreType::Mana => target_state.mana += amount,
        }
    }
}

pub fn resuscitate_character(target: &mut Target) {
    let (_, (target_specs, target_state)) = target;
    target_state.is_alive = true;
    target_state.life = target_specs.max_life;
}

pub fn apply_status(
    target: &mut Target,
    status_type: StatusType,
    skill_type: SkillType,
    value: f64,
    duration: f64,
    cumulate: bool,
) {
    let (_, (target_specs, target_state)) = target;

    if duration <= 0.0 || !target_state.is_alive {
        return;
    }

    let value = match status_type {
        StatusType::DamageOverTime {
            damage_type,
            ignore_armor,
        } => compute_damage(target_specs, value, damage_type, skill_type, ignore_armor),
        _ => value,
    };

    let statuses = target_state
        .statuses
        .entry(status_type)
        .or_insert_with(Vec::new);

    if cumulate {
        statuses.push(StatusState {
            value,
            duration,
            cumulate,
        });
    } else if let Some(cur_status) = statuses.iter_mut().find(|s| !s.cumulate) {
        if value * duration > cur_status.value * cur_status.duration {
            cur_status.value = value;
            cur_status.duration = duration;
        }
    } else {
        statuses.push(StatusState {
            value,
            duration,
            cumulate,
        })
    }

    if let StatusType::StatModifier { .. } = status_type {
        target_state.buff_status_change = true;
    }
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
            .unwrap_or(0.0))
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
        * match damage_type {
            DamageType::Physical => {
                1.0 - increase_factors::diminishing(
                    target_specs.armor,
                    increase_factors::ARMOR_FACTOR,
                )
            }
            DamageType::Fire => {
                1.0 - increase_factors::diminishing(
                    target_specs.fire_armor,
                    increase_factors::ARMOR_FACTOR,
                )
            }
            DamageType::Poison => {
                1.0 - increase_factors::diminishing(
                    target_specs.poison_armor,
                    increase_factors::ARMOR_FACTOR,
                )
            }
        }
}
