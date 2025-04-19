use leptos::prelude::*;

use crate::components::game::game_instance::GameInstance;
use crate::components::websocket::Websocket;

#[component]
pub fn Game() -> impl IntoView {
    view! {
        <Websocket>
            <GameInstance/>
        </Websocket>
    }
}
