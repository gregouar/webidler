use shared::data::{
    character::{CharacterSpecs, CharacterState},
    skill::{DamageType, SkillType},
};

use crate::game::utils::{increase_factors, rng};

const ARMOR_FACTOR: f64 = 100.0;

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

    let amount = amount
        * match damage_type {
            DamageType::Physical => {
                1.0 - increase_factors::diminishing(target_specs.armor, ARMOR_FACTOR)
            }
            DamageType::Fire => {
                1.0 - increase_factors::diminishing(target_specs.fire_armor, ARMOR_FACTOR)
            }
            DamageType::Poison => {
                1.0 - increase_factors::diminishing(target_specs.poison_armor, ARMOR_FACTOR)
            }
        };

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
