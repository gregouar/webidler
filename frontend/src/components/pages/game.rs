use leptos::prelude::*;

use crate::components::backend_client;
use crate::components::game::game_instance::GameInstance;
use crate::components::websocket::Websocket;

#[component]
pub fn GamePage() -> impl IntoView {
    let backend_client = use_context::<backend_client::BackendClient>().unwrap();
    view! {
        <Websocket url=backend_client.get_game_ws_url()>
            <GameInstance />
        </Websocket>
    }
}
