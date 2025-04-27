use leptos::prelude::*;

use crate::components::game::game_instance::GameInstance;
use crate::components::websocket::Websocket;

#[component]
pub fn Game() -> impl IntoView {
    view! {
        <Websocket url="wss://webidler.gregoirenaisse.be/ws">
            <GameInstance />
        </Websocket>
    }
}

#[component]
pub fn LocalGame() -> impl IntoView {
    view! {
        <Websocket url="ws://127.0.0.1:4200/ws">
            <GameInstance />
        </Websocket>
    }
}
