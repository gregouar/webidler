use std::ops::ControlFlow;
use tokio::task::yield_now;

use shared::messages::{
    client::ClientMessage,
    server::{ErrorMessage, ErrorType},
};

use crate::{
    game::{data::master_store::MasterStore, systems::quests_controller},
    websocket::WebSocketConnection,
};

use super::{
    game_data::GameInstanceData,
    systems::{loot_controller, passives_controller, player_controller, skills_controller},
};

/// Handle client events, return whether the game should stop or continue
pub async fn handle_client_inputs(
    client_conn: &mut WebSocketConnection,
    game_data: &mut GameInstanceData,
    master_store: &MasterStore,
) -> ControlFlow<(), ()> {
    // We limit the amount of events we handle in one loop
    for _ in 1..100 {
        match client_conn.poll_receive() {
            ControlFlow::Continue(Some(m)) => {
                if let Some(error_message) = handle_client_message(master_store, game_data, m) {
                    if let Err(e) = client_conn.send(&error_message.into()).await {
                        tracing::warn!("failed to send error to client: {}", e)
                    }
                }
            }
            ControlFlow::Continue(None) => return ControlFlow::Continue(()), // No more messages
            ControlFlow::Break(_) => return ControlFlow::Break(()),          // Connection closed
        }
        yield_now().await;
    }
    ControlFlow::Continue(())
}

fn handle_client_message(
    master_store: &MasterStore,
    game_data: &mut GameInstanceData,
    msg: ClientMessage,
) -> Option<ErrorMessage> {
    match msg {
        ClientMessage::Heartbeat => {}
        ClientMessage::EndQuest => {
            quests_controller::end_quest(master_store, game_data);
        }
        ClientMessage::TerminateQuest(m) => {
            if let Err(err) = quests_controller::terminate_quest(game_data, m.item_index) {
                return Some(ErrorMessage {
                    error_type: ErrorType::Game,
                    message: err.to_string(),
                    must_disconnect: false,
                });
            }
        }
        ClientMessage::UseSkill(m) => {
            game_data
                .player_controller
                .use_skills
                .push(m.skill_index as usize);
        }
        ClientMessage::SetAutoSkill(m) => {
            if let Some(x) = game_data
                .player_specs
                .mutate()
                .auto_skills
                .get_mut(m.skill_index as usize)
            {
                *x = m.auto_use;
            }
        }
        ClientMessage::LevelUpSkill(m) => {
            for _ in 0..m.amount {
                if let Some(skill_specs) = game_data
                    .player_specs
                    .mutate()
                    .skills_specs
                    .get_mut(m.skill_index as usize)
                {
                    skills_controller::level_up_skill(
                        skill_specs,
                        game_data.player_resources.mutate(),
                    );
                }
            }
        }
        ClientMessage::BuySkill(m) => {
            player_controller::buy_skill(
                &master_store.skills_store,
                game_data.player_specs.mutate(),
                &mut game_data.player_state,
                game_data.player_resources.mutate(),
                &m.skill_id,
            );
        }
        ClientMessage::LevelUpPlayer(m) => {
            for _ in 0..m.amount {
                player_controller::level_up(
                    game_data.player_specs.mutate(),
                    &mut game_data.player_state,
                    game_data.player_resources.mutate(),
                );
            }
        }
        ClientMessage::EquipItem(m) => {
            if let Err(err) = player_controller::equip_item_from_bag(
                game_data.player_specs.mutate(),
                game_data.player_inventory.mutate(),
                &mut game_data.player_state,
                m.item_index,
            ) {
                return Some(ErrorMessage {
                    error_type: ErrorType::Game,
                    message: err.to_string(),
                    must_disconnect: false,
                });
            }
        }
        ClientMessage::UnequipItem(m) => {
            if let Err(err) = player_controller::unequip_item_to_bag(
                game_data.player_specs.mutate(),
                game_data.player_inventory.mutate(),
                &mut game_data.player_state,
                m.item_slot,
            ) {
                return Some(ErrorMessage {
                    error_type: ErrorType::Game,
                    message: err.to_string(),
                    must_disconnect: false,
                });
            }
        }
        ClientMessage::SellItems(m) => {
            let mut item_indexes = m.item_indexes;
            item_indexes.sort_by_key(|&i| i);
            for &item_index in item_indexes.iter().rev() {
                player_controller::sell_item_from_bag(
                    &game_data.area_blueprint.specs,
                    game_data.player_specs.read(),
                    game_data.player_inventory.mutate(),
                    game_data.player_resources.mutate(),
                    item_index,
                )
            }
        }
        ClientMessage::FilterLoot(m) => {
            game_data.player_controller.preferred_loot = m.preferred_loot;
        }
        ClientMessage::PickupLoot(m) => {
            if m.sell {
                if let Some(item_specs) =
                    loot_controller::take_loot(game_data.queued_loot.mutate(), m.loot_identifier)
                {
                    player_controller::sell_item(
                        &game_data.area_blueprint.specs,
                        game_data.player_specs.read(),
                        game_data.player_resources.mutate(),
                        &item_specs,
                    );
                }
            } else if let Err(e) = loot_controller::pickup_loot(
                &game_data.player_controller,
                game_data.player_inventory.mutate(),
                game_data.queued_loot.mutate(),
                m.loot_identifier,
            ) {
                return Some(ErrorMessage {
                    error_type: ErrorType::Game,
                    message: e.to_string(),
                    must_disconnect: false,
                });
            }
        }
        ClientMessage::SetAutoProgress(m) => game_data.area_state.mutate().auto_progress = m.value,
        ClientMessage::GoBack(m) => {
            let area_state = game_data.area_state.mutate();
            area_state.going_back += m.amount;
            area_state.auto_progress = false;
        }
        ClientMessage::SetRushMode(m) => game_data.area_state.mutate().rush_mode = m.value,
        ClientMessage::PurchasePassive(m) => passives_controller::purchase_node(
            game_data.player_resources.mutate(),
            &game_data.passives_tree_specs,
            game_data.passives_tree_state.mutate(),
            m.node_id,
        ),
        ClientMessage::Connect(_) => {
            tracing::warn!("received unexpected message: {:?}", msg);
            return Some(ErrorMessage {
                error_type: ErrorType::Server,
                message: "unexpected message received from client".to_string(),
                must_disconnect: true,
            });
        }
    }
    None
}
