use std::time::Duration;

use shared::data::player::{PlayerSpecs, PlayerState};

use super::{characters_updater, skills_updater};

pub fn update_player_state(
    elapsed_time: Duration,
    player_specs: &PlayerSpecs,
    player_state: &mut PlayerState,
) {
    if !player_state.character_state.is_alive {
        return;
    }

    characters_updater::update_character_state(
        elapsed_time,
        &player_specs.character_specs,
        &mut player_state.character_state,
    );
    skills_updater::update_skills_states(
        elapsed_time,
        &player_specs.skill_specs,
        &mut player_state.skill_states,
    );

    player_state.mana = player_specs
        .max_mana
        .min(player_state.mana + (elapsed_time.as_secs_f64() * player_specs.mana_regen));
}

pub fn reset_player(player_state: &mut PlayerState) {
    player_state.just_leveled_up = false;
    characters_updater::reset_character(&mut player_state.character_state);
    skills_updater::reset_skills(&mut player_state.skill_states);
}
