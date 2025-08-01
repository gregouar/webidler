use anyhow::Result;

use shared::messages::server::{InitGameMessage, SyncGameStateMessage};

use super::game_data::GameInstanceData;

use crate::websocket::WebSocketConnection;

pub async fn sync_init_game(
    client_conn: &mut WebSocketConnection,
    game_data: &mut GameInstanceData,
) -> Result<()> {
    game_data.reset_syncers();
    client_conn
        .send(
            &InitGameMessage {
                world_specs: game_data.world_blueprint.specs.clone(),
                world_state: game_data.world_state.read().clone(),
                passives_tree_specs: game_data.passives_tree_specs.clone(),
                passives_tree_state: game_data.passives_tree_state.read().clone(),
                player_specs: game_data.player_specs.read().clone(),
                player_state: game_data.player_state.clone(),
            }
            .into(),
        )
        .await
}

/// Send whole game state to client
pub async fn sync_update_game(
    client_conn: &mut WebSocketConnection,
    game_data: &mut GameInstanceData,
) -> Result<()> {
    client_conn
        .send(
            &SyncGameStateMessage {
                world_state: game_data.world_state.sync(),
                passives_tree_state: game_data.passives_tree_state.sync(),
                player_specs: game_data.player_specs.sync(),
                player_inventory: game_data.player_inventory.sync(),
                player_state: game_data.player_state.clone(),
                player_resources: game_data.player_resources.sync(),
                monster_specs: game_data.monster_base_specs.sync(),
                monster_states: game_data.monster_states.clone(),
                queued_loot: game_data.queued_loot.sync(),
                game_stats: game_data.game_stats.clone(),
            }
            .into(),
        )
        .await
}
