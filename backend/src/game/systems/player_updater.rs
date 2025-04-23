use std::time::Duration;

use shared::data::{PlayerSpecs, PlayerState};

use super::characters_updater;

pub fn update_player_state(
    elapsed_time: Duration,
    player_specs: &mut PlayerSpecs,
    player_state: &mut PlayerState,
) -> bool {
    let mut update_player_specs = false;

    if !player_state.character_state.is_alive {
        return update_player_specs;
    }

    characters_updater::update_character_state(
        elapsed_time,
        &player_specs.character_specs,
        &mut player_state.character_state,
    );

    player_state.mana = player_specs
        .max_mana
        .min(player_state.mana + (elapsed_time.as_secs_f64() * player_specs.mana_regen));

    // TODO: Move somewhere else, should be result of client query :o)
    while player_state.experience >= player_specs.experience_needed {
        level_up(player_specs, player_state);
        update_player_specs = true;
    }

    return update_player_specs;
}

fn level_up(player_specs: &mut PlayerSpecs, player_state: &mut PlayerState) {
    player_specs.level += 1;
    player_state.experience -= player_specs.experience_needed;
    player_specs.experience_needed = player_specs.experience_needed * 10.0;

    player_state.character_state.health += 10.0;
    player_specs.character_specs.max_health += 10.0;
}
