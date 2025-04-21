use std::time::Duration;

use shared::data::{PlayerPrototype, PlayerState};

use super::characters_updater;

pub fn update_player_state(
    elapsed_time: Duration,
    player_prototype: &PlayerPrototype,
    player_state: &mut PlayerState,
) {
    if !player_state.character_state.is_alive {
        return;
    }

    characters_updater::update_character_state(
        elapsed_time,
        &player_prototype.character_prototype,
        &mut player_state.character_state,
    );

    player_state.mana = player_prototype
        .max_mana
        .min(player_state.mana + (elapsed_time.as_secs_f64() * player_prototype.mana_regen));
}
