use codee::string::JsonSerdeCodec;
use leptos::{html::*, prelude::*};
use leptos_use::storage;

use shared::data::user::UserCharacterId;
use shared::messages::client::ClientConnectMessage;
use shared::messages::server::{ErrorType, InitGameMessage, ServerMessage, SyncGameStateMessage};

use crate::components::ui::{toast::*, tooltip::DynamicTooltip};
use crate::components::websocket::WebsocketContext;

use super::battle_scene::BattleScene;
use super::header_menu::HeaderMenu;
use super::panels::{InventoryPanel, PassivesPanel, SkillsPanel, StatisticsPanel};
use super::GameContext;

#[component]
pub fn GameInstance(character_id: UserCharacterId) -> impl IntoView {
    let game_context = GameContext::new();
    provide_context(game_context.clone());

    let (get_jwt_storage, _, _) = storage::use_local_storage::<String, JsonSerdeCodec>("jwt");

    let (get_area_id_storage, _, _) =
        storage::use_session_storage::<Option<String>, JsonSerdeCodec>("area_id");

    Effect::new({
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            if conn.connected.get() {
                conn.send(
                    &ClientConnectMessage {
                        jwt: get_jwt_storage.get(),
                        character_id,
                        area_id: get_area_id_storage.get_untracked(),
                    }
                    .into(),
                );
            }
        }
    });

    Effect::new({
        let game_context = game_context.clone();
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            if let Some(message) = conn.message.get() {
                handle_message(&game_context, message);
            }
        }
    });

    view! {
        <main class="my-0 mx-auto w-full text-center overflow-x-hidden flex flex-col min-h-screen">
            <DynamicTooltip />
            <Show
                when=move || game_context.started.get()
                fallback=move || view! { <p>"Connecting..."</p> }
            >
                <HeaderMenu />
                <div class="relative flex-1">
                    <BattleScene />
                    <InventoryPanel open=game_context.open_inventory />
                    <PassivesPanel open=game_context.open_passives />
                    <StatisticsPanel open=game_context.open_statistics />
                    <SkillsPanel open=game_context.open_skills />
                </div>
            </Show>
        </main>
    }
}

fn handle_message(game_context: &GameContext, message: ServerMessage) {
    match message {
        ServerMessage::Connect(_) => {}
        ServerMessage::InitGame(m) => {
            init_game(game_context, m);
        }
        ServerMessage::UpdateGame(m) => {
            sync_game(game_context, m);
        }
        ServerMessage::Error(error_message) => {
            let toaster = expect_context::<Toasts>();
            show_toast(
                toaster,
                error_message.message,
                match error_message.error_type {
                    ErrorType::Server => ToastVariant::Error,
                    ErrorType::Game => ToastVariant::Warning,
                },
            );
        }
    }
}

fn init_game(game_context: &GameContext, init_message: InitGameMessage) {
    let InitGameMessage {
        area_specs,
        area_state,
        passives_tree_specs,
        passives_tree_state,
        player_specs,
        player_state,
    } = init_message;

    game_context.started.set(true);
    game_context.area_specs.set(area_specs);
    game_context.area_state.set(area_state);
    game_context.passives_tree_specs.set(passives_tree_specs);
    game_context.passives_tree_state.set(passives_tree_state);
    game_context.player_specs.set(player_specs);
    game_context.player_state.set(player_state);
}

fn sync_game(game_context: &GameContext, sync_message: SyncGameStateMessage) {
    let SyncGameStateMessage {
        area_state,
        passives_tree_state,
        player_specs,
        player_inventory,
        player_state,
        player_resources,
        monster_specs,
        monster_states,
        queued_loot,
        game_stats,
    } = sync_message;

    if let Some(area_state) = area_state {
        game_context.area_state.set(area_state);
    }
    if let Some(passives_tree_state) = passives_tree_state {
        game_context.passives_tree_state.set(passives_tree_state);
    }
    if let Some(player_specs) = player_specs {
        game_context.player_specs.set(player_specs);
    }
    if let Some(player_inventory) = player_inventory {
        game_context.player_inventory.set(player_inventory);
    }
    if let Some(player_resources) = player_resources {
        game_context.player_resources.set(player_resources);
    }
    game_context.player_state.set(player_state);
    if let Some(monster_specs) = monster_specs {
        *game_context.monster_wave.write() += 1; // TODO: Overflow
        game_context.monster_specs.set(monster_specs);
    }
    game_context.monster_states.set(monster_states);
    if let Some(queued_loot) = queued_loot {
        game_context.queued_loot.set(queued_loot);
    }
    game_context.game_stats.set(game_stats);
}
