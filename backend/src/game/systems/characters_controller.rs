use shared::data::{
    character::{CharacterSpecs, CharacterState},
    skill::DamageType,
};

use crate::game::utils::increase_factors;

const ARMOR_FACTOR: f64 = 100.0;

pub fn damage_character(
    damage: f64,
    damage_type: DamageType,
    target_state: &mut CharacterState,
    target_specs: &CharacterSpecs,
    is_crit: bool,
) {
    let damage = damage
        * match damage_type {
            DamageType::Physical => {
                1.0 - increase_factors::diminishing(target_specs.armor, ARMOR_FACTOR)
            }
            DamageType::Fire => 1.0, // TODO
        };

    if damage <= 0.0 {
        return;
    }

    target_state.health = (target_state.health - damage)
        .max(0.0)
        .min(target_specs.max_health);

    if damage > 0.0 {
        target_state.just_hurt = true;
        if is_crit {
            target_state.just_hurt_crit = true;
        }
        // TODO: just critically hurt
    }

    if target_state.is_alive && target_state.health == 0.0 {
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
            .min(target_specs.max_health);
    }
}
