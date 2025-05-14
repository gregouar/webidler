use codee::string::JsonSerdeCodec;
use leptos::html::*;
use leptos::prelude::*;
use leptos_use::storage::use_session_storage;

use shared::messages::client::ClientConnectMessage;
use shared::messages::server::{ErrorType, InitGameMessage, ServerMessage, SyncGameStateMessage};
use shared::messages::SessionKey;

use crate::components::ui::toast::*;
use crate::components::ui::tooltip::DynamicTooltip;
use crate::components::websocket::WebsocketContext;

use super::battle_scene::BattleScene;
use super::header_menu::HeaderMenu;
use super::inventory::Inventory;
use super::GameContext;

#[component]
pub fn GameInstance() -> impl IntoView {
    let game_context = GameContext::new();
    provide_context(game_context.clone());

    // TODO: CHECK LOCAL STORE FOR SESSION_KEY
    let (session_key, set_session_key, _) =
        use_session_storage::<Option<SessionKey>, JsonSerdeCodec>("session_key");

    Effect::new({
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            if conn.connected.get() {
                conn.send(
                    &ClientConnectMessage {
                        user_id: String::from("Le Pou"),
                        session_key: session_key.get_untracked(),
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
                handle_message(&game_context, set_session_key, message);
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
                    <Inventory open=game_context.open_inventory />
                </div>
            </Show>
        </main>
    }
}

fn handle_message(
    game_context: &GameContext,
    set_session_key: WriteSignal<Option<SessionKey>>,
    message: ServerMessage,
) {
    match message {
        ServerMessage::Connect(m) => {
            set_session_key.set(Some(m.session_key));
        }
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
        world_specs,
        world_state,
        player_specs,
        player_state,
    } = init_message;

    game_context.started.set(true);
    game_context.world_specs.set(world_specs);
    game_context.world_state.set(world_state);
    game_context.player_specs.set(player_specs);
    game_context.player_state.set(player_state);
}

fn sync_game(game_context: &GameContext, sync_message: SyncGameStateMessage) {
    let SyncGameStateMessage {
        world_state,
        player_specs,
        player_inventory,
        player_state,
        player_resources,
        monster_specs,
        monster_states,
        queued_loot,
    } = sync_message;

    game_context.world_state.set(world_state);
    if let Some(player_specs) = player_specs {
        game_context.player_specs.set(player_specs);
    }
    if let Some(player_inventory) = player_inventory {
        game_context.player_inventory.set(player_inventory);
    }
    game_context.player_resources.set(player_resources);
    game_context.player_state.set(player_state);
    if let Some(monster_specs) = monster_specs {
        *game_context.monster_wave.write() += 1; // TODO: Overflow
        game_context.monster_specs.set(monster_specs);
    }
    game_context.monster_states.set(monster_states);
    if let Some(queued_loot) = queued_loot {
        game_context.queued_loot.set(queued_loot);
    }
}
