use shared::data::{
    character::{CharacterSpecs, CharacterState},
    skill::{DamageType, SkillType},
    status::{StatusState, StatusType},
};

use crate::game::utils::{increase_factors, rng};

pub fn damage_character(
    amount: f64,
    damage_type: DamageType,
    skill_type: SkillType,
    target_state: &mut CharacterState,
    target_specs: &CharacterSpecs,
    is_crit: bool,
) {
    let is_blocked = skill_type == SkillType::Attack
        && rng::random_range(0.0..=1.0).unwrap_or(1.0) <= target_specs.block;

    if is_blocked {
        target_state.just_blocked = true;
        return;
    }

    let amount = decrease_damage_from_armor(amount, damage_type, target_specs);

    if amount <= 0.0 {
        return;
    }

    target_state.health = (target_state.health - amount)
        .max(0.0)
        .min(target_specs.max_life);

    if amount > 0.0 {
        target_state.just_hurt = true;
        if is_crit {
            target_state.just_hurt_crit = true;
        }
    }

    if target_state.is_alive && target_state.health < 0.5 {
        target_state.health = 0.0;
        target_state.is_alive = false;
        target_state.just_died = true;
    }
}

pub fn heal_character(
    amount: f64,
    target_state: &mut CharacterState,
    target_specs: &CharacterSpecs,
) {
    if amount <= 0.0 {
        return;
    }

    if target_state.is_alive {
        target_state.health = (target_state.health + amount)
            .max(0.0)
            .min(target_specs.max_life);
    }
}

pub fn apply_status(
    status_type: StatusType,
    value: f64,
    duration: f64,
    target_state: &mut CharacterState,
    target_specs: &CharacterSpecs,
) {
    if duration <= 0.0 || !target_state.is_alive {
        return;
    }

    let mut value = value;

    match status_type {
        StatusType::Stunned => {}
        StatusType::DamageOverTime(damage_type) => {
            value = decrease_damage_from_armor(value, damage_type, target_specs);
        }
    }

    target_state
        .statuses
        .entry(status_type)
        .and_modify(|cur_status| {
            if value * duration > cur_status.value * cur_status.duration {
                cur_status.value = value;
                cur_status.duration = duration;
            }
        })
        .or_insert(StatusState { value, duration });
}

fn decrease_damage_from_armor(
    amount: f64,
    damage_type: DamageType,
    target_specs: &CharacterSpecs,
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
