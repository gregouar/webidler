use std::time::Duration;

use shared::data::{PlayerSpecs, PlayerState};

use super::characters_updater;

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

    player_state.mana = player_specs
        .max_mana
        .min(player_state.mana + (elapsed_time.as_secs_f64() * player_specs.mana_regen));
}
