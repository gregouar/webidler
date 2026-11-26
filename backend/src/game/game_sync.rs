use anyhow::Result;

use shared::{
    data::{
        cosmetics::CharacterCosmetics,
        monster::MonsterSpecsPatch,
        passive::PurchasedNodes,
        pets::PlayerPets,
        user::{UserCharacterId, UserUnlocks},
    },
    messages::server::{InitGameMessage, SyncGameStateMessage},
};

use super::game_data::GameInstanceData;

use crate::websocket::WebSocketConnection;

pub async fn sync_init_game(
    client_conn: &mut WebSocketConnection,
    character_id: &UserCharacterId,
    game_data: &mut GameInstanceData,
    passives_tree_build: PurchasedNodes,
    character_cosmetics: CharacterCosmetics,
    user_unlocks: UserUnlocks,
    player_pets: PlayerPets,
    has_ever_looted_gems: bool,
) -> Result<()> {
    game_data.reset_syncers();
    client_conn
        .send(
            &InitGameMessage {
                character_id: *character_id,
                character_cosmetics,
                user_unlocks,
                achievements: game_data.achievements.clone(),
                has_ever_looted_gems,
                realm: game_data.realm,
                area_id: game_data.area_id.clone(),
                map_item: game_data.map_item.clone(),
                area_specs: game_data.area_specs.clone(),
                area_state: game_data.area_state.read().clone(),
                quest_completed: game_data.quest_completed,
                passives_tree_specs: game_data.passives_tree_specs.clone(),
                passives_tree_state: game_data.passives_tree_state.read().clone(),
                passives_tree_build,
                player_base_specs: game_data.player_base_specs.read().clone(),
                skill_mastery_skill_specs: game_data.skill_mastery_skill_specs.clone(),
                player_specs: game_data.player_specs.read().clone(),
                player_state: game_data.player_state.clone(),
                player_pets,
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

    let monster_specs_patch = (!game_data.monster_specs_to_patch.is_empty()).then(|| {
        game_data
            .monster_specs_to_patch
            .take_indices()
            .filter_map(|monster_idx| {
                game_data
                    .monster_specs
                    .get(monster_idx as usize)
                    .map(|monster_specs| {
                        (
                            monster_idx,
                            MonsterSpecsPatch {
                                character_attrs: monster_specs
                                    .character_specs
                                    .character_attrs
                                    .clone(),
                                skill_specs_targets: monster_specs
                                    .character_specs
                                    .skills_specs
                                    .iter()
                                    .map(|skills_specs| skills_specs.targets.clone())
                                    .collect(),
                            },
                        )
                    })
            })
            .collect()
    });

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
        monster_specs_patch,
        monster_states: game_data.monster_states.clone(),
        queued_loot: game_data.queued_loot.sync(),
        game_stats: game_data.game_stats.clone(),
        grind_rewards: game_data.grind_rewards.sync(),
    };

    client_conn.start_background_send(&message.into())?;

    Ok(true)
}
