use std::time::Duration;

use shared::data::{
    character::{CharacterSpecs, CharacterState},
    character_status::StatusType,
};

pub fn update_character_state(
    elapsed_time: Duration,
    character_specs: &CharacterSpecs,
    character_state: &mut CharacterState,
) {
    if !character_state.is_alive {
        return;
    }

    if character_state.health < 0.5 {
        character_state.health = 0.0;
        character_state.is_alive = false;
        character_state.just_died = true;
        return;
    }

    character_state.health = character_specs.max_life.min(
        character_state.health
            + (elapsed_time.as_secs_f64() * character_specs.life_regen * character_specs.max_life
                / 100.0),
    );

    character_state
        .statuses
        .retain(|status_type, status_state| {
            match status_type {
                StatusType::DamageOverTime(_) => {
                    character_state.health -=
                        status_state.value * elapsed_time.as_secs_f64().min(status_state.duration)
                }
                StatusType::Stunned => {}
            }
            status_state.duration -= elapsed_time.as_secs_f64();
            status_state.duration > 0.0
        });
}

pub fn reset_character(character_state: &mut CharacterState) {
    character_state.just_died = false;
    character_state.just_hurt = false;
    character_state.just_hurt_crit = false;
    character_state.just_blocked = false;
}
