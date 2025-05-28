use std::time::Duration;

use shared::data::{
    character::{CharacterId, CharacterSpecs, CharacterState},
    character_status::StatusType,
};

use crate::game::data::event::{EventsQueue, GameEvent};

use super::characters_controller;

pub fn update_character_state(
    events_queue: &mut EventsQueue,
    elapsed_time: Duration,
    character_id: CharacterId,
    character_specs: &CharacterSpecs,
    character_state: &mut CharacterState,
) {
    if !character_state.is_alive {
        return;
    }

    let elapsed_time_f64 = elapsed_time.as_secs_f64();

    character_state.life = character_specs.max_life.min(
        character_state.life
            + (elapsed_time_f64 * character_specs.life_regen * character_specs.max_life / 100.0),
    );

    character_state.mana = character_specs.max_mana.min(
        character_state.mana
            + (elapsed_time_f64 * character_specs.mana_regen * character_specs.max_mana / 100.0),
    );

    character_state
        .statuses
        .retain(|status_type, status_states| {
            match status_type {
                StatusType::DamageOverTime { .. } => {
                    for status in status_states.iter() {
                        characters_controller::damage_character(
                            character_specs,
                            &mut character_state.life,
                            &mut character_state.mana,
                            status.value * elapsed_time_f64.min(status.duration),
                        );
                    }
                }
                _ => {}
            }

            let old_len = status_states.len();
            status_states.retain_mut(|status| {
                status.duration -= elapsed_time_f64;
                status.duration > 0.0
            });

            if let StatusType::StatModifier(_) = status_type {
                if old_len != status_states.len() {
                    character_state.buff_status_change = true;
                }
            }

            !status_states.is_empty()
        });

    character_state.life = character_state.life.clamp(0.0, character_specs.max_life);
    character_state.mana = character_state.mana.clamp(0.0, character_specs.max_mana);

    if character_state.life < 0.5 {
        character_state.life = 0.0;
        character_state.is_alive = false;
        events_queue.register_event(GameEvent::Kill {
            target: character_id,
        });
        return;
    }
}

pub fn reset_character(character_state: &mut CharacterState) {
    character_state.just_hurt = false;
    character_state.just_hurt_crit = false;
    character_state.just_blocked = false;
}
