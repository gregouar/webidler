use leptos::html::*;
use leptos::prelude::*;

use shared::messages::client::ClientConnectMessage;
use shared::messages::server::ServerMessage;

use crate::components::websocket::WebsocketContext;

use super::header_menu::HeaderMenu;
use super::monsters_grid::MonstersGrid;
use super::player_card::PlayerCard;
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
                handle_message(&game_context, &m);
            }
        }
    });

    view! {
        <main class="my-0 mx-auto text-center overflow-x-hidden">
            <Show
                when=move || game_context.started.get()
                fallback=move || view! { <p>"Connecting..."</p> }
            >
                <HeaderMenu />
                <div class="grid grid-cols-3 justify-items-stretch flex items-start gap-4 m-4 ">
                    <PlayerCard class:col-span-1 class:justify-self-end />
                    <MonstersGrid class:col-span-2 class:justify-self-start />
                </div>
            </Show>
        </main>
    }
}

fn handle_message(game_context: &GameContext, message: &ServerMessage) {
    match message {
        ServerMessage::Connect(_) => {}
        ServerMessage::InitGame(m) => {
            game_context.started.set(true);
            game_context
                .player_prototype
                .set(m.player_prototype.clone());
            game_context.player_state.set(m.player_state.clone());
        }
        ServerMessage::UpdateGame(m) => {
            game_context.player_state.set(m.player_state.clone());
            if let Some(monster_prototypes) = m.monster_prototypes.as_ref() {
                *game_context.monster_wave.write() += 1; // TODO: Overflow
                game_context
                    .monster_prototypes
                    .set(monster_prototypes.clone());
            }
            game_context.monster_states.set(m.monster_states.clone());
        }
    }
}
