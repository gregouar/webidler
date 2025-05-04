use shared::data::{
    character::{CharacterSpecs, CharacterState},
    skill::DamageType,
};

pub fn damage_character(
    damage: f64,
    damage_type: DamageType,
    target_state: &mut CharacterState,
    target_specs: &CharacterSpecs,
) {
    let _ = damage_type; // TODO

    if damage <= 0.0 {
        return;
    }

    target_state.health = (target_state.health - damage)
        .max(0.0)
        .min(target_specs.max_health);

    if damage > 0.0 {
        target_state.just_hurt = true;
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
