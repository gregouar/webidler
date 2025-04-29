use shared::data::character::{CharacterSpecs, CharacterState};

// TODO: Should figure out a better way to trace this?
/// Return whether the character died from the damages
pub fn damage_character(
    damage: f64,
    target_state: &mut CharacterState,
    target_specs: &CharacterSpecs,
) -> bool {
    target_state.health = (target_state.health - damage)
        .max(0.0)
        .min(target_specs.max_health);

    if damage > 0.0 {
        target_state.just_hurt = true;
    }

    if target_state.is_alive && target_state.health == 0.0 {
        target_state.is_alive = false;
        return true;
    }
    false
}
