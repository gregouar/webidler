use anyhow::Result;

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
) -> Result<()> {
    game_data.reset_syncers();
    client_conn
        .send(
            &InitGameMessage {
                character_id: *character_id,
                map_item: game_data.map_item.clone(),
                area_specs: game_data.area_specs.clone(),
                area_state: game_data.area_state.read().clone(),
                passives_tree_specs: game_data.passives_tree_specs.clone(),
                passives_tree_state: game_data.passives_tree_state.read().clone(),
                passives_tree_build,
                player_base_specs: game_data.player_base_specs.read().clone(),
                player_specs: game_data.player_specs.read().clone(),
                player_state: game_data.player_state.clone(),
                auto_skills: game_data.player_controller.auto_skills.read().clone(),
            }
            .into(),
        )
        .await
}

/// Start sending the current game state to the client if the previous update has finished.
/// Returns false when an update is still in flight; syncers are left untouched in that case.
pub async fn sync_update_game(
    client_conn: &mut WebSocketConnection,
    game_data: &mut GameInstanceData,
) -> Result<bool> {
    if !client_conn.poll_pending_send().await? {
        return Ok(false);
    }

    let message = SyncGameStateMessage {
        area_state: game_data.area_state.sync(),
        area_threat: game_data.area_threat.clone(),
        passives_tree_state: game_data.passives_tree_state.sync(),
        player_base_specs: game_data.player_base_specs.sync(),
        player_specs: game_data.player_specs.sync(),
        player_inventory: game_data.player_inventory.sync(),
        player_state: game_data.player_state.clone(),
        auto_skills: game_data.player_controller.auto_skills.sync(),
        player_resources: game_data.player_resources.sync(),
        player_stamina: game_data.player_stamina,
        monster_specs: game_data.monster_base_specs.sync(),
        monster_states: game_data.monster_states.clone(),
        queued_loot: game_data.queued_loot.sync(),
        game_stats: game_data.game_stats.clone(),
        quest_rewards: game_data.quest_rewards.sync(),
    };

    client_conn.start_background_send(&message.into())?;

    Ok(true)
}
