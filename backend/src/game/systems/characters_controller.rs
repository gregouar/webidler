use shared::data::character::{CharacterSpecs, CharacterState};

pub fn damage_character(
    damage: f64,
    target_state: &mut CharacterState,
    target_specs: &CharacterSpecs,
) {
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
