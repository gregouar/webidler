use leptoaster::expect_toaster;
use leptos::html::*;
use leptos::prelude::*;

use shared::messages::client::ClientConnectMessage;
use shared::messages::server::{ErrorType, InitGameMessage, ServerMessage, SyncGameStateMessage};

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

    Effect::new({
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            if conn.connected.get() {
                conn.send(
                    &ClientConnectMessage {
                        bearer: String::from("Le Pou"),
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
            if let Some(m) = conn.message.get() {
                handle_message(&game_context, m);
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

fn handle_message(game_context: &GameContext, message: ServerMessage) {
    let toaster = expect_toaster();
    match message {
        ServerMessage::Connect(_) => {}
        ServerMessage::InitGame(InitGameMessage {
            world_specs,
            world_state,
            player_specs,
            player_state,
        }) => {
            game_context.started.set(true);
            game_context.world_specs.set(world_specs);
            game_context.world_state.set(world_state);
            game_context.player_specs.set(player_specs);
            game_context.player_state.set(player_state);
        }
        ServerMessage::UpdateGame(SyncGameStateMessage {
            world_state,
            player_resources,
            player_specs,
            player_state,
            monster_specs,
            monster_states,
            queued_loot,
        }) => {
            game_context.world_state.set(world_state);
            if let Some(player_specs) = player_specs {
                game_context.player_specs.set(player_specs);
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
        ServerMessage::Error(error_message) => match error_message.error_type {
            ErrorType::Server => toaster.error(error_message.message),
            ErrorType::Game => toaster.info(error_message.message),
        },
    }
}
