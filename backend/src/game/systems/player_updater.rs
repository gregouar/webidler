use std::time::Duration;

use shared::data::{
    character::CharacterId,
    player::{PlayerSpecs, PlayerState},
};

use crate::game::data::event::EventsQueue;

use super::{characters_updater, skills_updater};

pub fn update_player_state(
    events_queue: &mut EventsQueue,
    elapsed_time: Duration,
    player_specs: &PlayerSpecs,
    player_state: &mut PlayerState,
) {
    if !player_state.character_state.is_alive {
        return;
    }

    characters_updater::update_character_state(
        events_queue,
        elapsed_time,
        CharacterId::Player,
        &player_specs.character_specs,
        &mut player_state.character_state,
    );
    skills_updater::update_skills_states(
        elapsed_time,
        &player_specs.skills_specs,
        &mut player_state.skills_states,
    );

    player_state.mana = player_specs.max_mana.min(
        player_state.mana
            + (elapsed_time.as_secs_f64() * player_specs.mana_regen * player_specs.max_mana
                / 100.0),
    );
}

pub fn reset_player(player_state: &mut PlayerState) {
    player_state.just_leveled_up = false;
    characters_updater::reset_character(&mut player_state.character_state);
    skills_updater::reset_skills(&mut player_state.skills_states);
}
