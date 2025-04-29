use std::time::Duration;

use shared::data::character::{CharacterSpecs, CharacterState};

pub fn update_character_state(
    elapsed_time: Duration,
    specs: &CharacterSpecs,
    state: &mut CharacterState,
) {
    if !state.is_alive {
        return;
    }

    state.health = specs
        .max_health
        .min(state.health + (elapsed_time.as_secs_f64() * specs.health_regen));
}

// TODO: Should figure out a better way to trace this?
/// Return whether the character died from the damages
pub fn damage_character(
    damages: f64,
    target_state: &mut CharacterState,
    target_specs: &CharacterSpecs,
) -> bool {
    target_state.health = (target_state.health - damages)
        .max(0.0)
        .min(target_specs.max_health);

    if damages > 0.0 {
        target_state.just_hurt = true;
    }

    if target_state.is_alive && target_state.health == 0.0 {
        target_state.is_alive = false;
        return true;
    }
    false
}
