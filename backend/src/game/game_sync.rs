use anyhow::Result;

use indexmap::IndexSet;
use shared::{
    data::{passive::PurchasedNodes, user::UserCharacterId},
    messages::server::{InitGameMessage, SyncGameStateMessage},
};

use super::game_data::GameInstanceData;

use crate::websocket::WebSocketConnection;

pub async fn sync_init_game(
    client_conn: &mut WebSocketConnection,
    character_id: &UserCharacterId,
    game_data: &mut GameInstanceData,
    passives_tree_build: PurchasedNodes,
    last_skills_bought: IndexSet<String>,
) -> Result<()> {
    game_data.reset_syncers();
    client_conn
        .send(
            &InitGameMessage {
                character_id: *character_id,
                area_specs: game_data.area_blueprint.specs.clone(),
                area_state: game_data.area_state.read().clone(),
                passives_tree_specs: game_data.passives_tree_specs.clone(),
                passives_tree_state: game_data.passives_tree_state.read().clone(),
                passives_tree_build,
                player_specs: game_data.player_specs.read().clone(),
                player_state: game_data.player_state.clone(),
                last_skills_bought,
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
                area_state: game_data.area_state.sync(),
                area_threat: game_data.area_threat.clone(),
                passives_tree_state: game_data.passives_tree_state.sync(),
                player_specs: game_data.player_specs.sync(),
                player_inventory: game_data.player_inventory.sync(),
                player_state: game_data.player_state.clone(),
                player_resources: game_data.player_resources.sync(),
                player_stamina: game_data.player_stamina,
                monster_specs: game_data.monster_base_specs.sync(),
                monster_states: game_data.monster_states.clone(),
                queued_loot: game_data.queued_loot.sync(),
                game_stats: game_data.game_stats.clone(),
                quest_rewards: game_data.quest_rewards.sync(),
            }
            .into(),
        )
        .await
}
