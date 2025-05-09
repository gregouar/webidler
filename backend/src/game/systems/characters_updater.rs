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

pub fn reset_character(character_state: &mut CharacterState) {
    character_state.just_hurt = false;
    character_state.just_died = false;
}
