use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::backend_client;
use crate::components::game::game_instance::GameInstance;
use crate::components::websocket::Websocket;

#[component]
pub fn GamePage() -> impl IntoView {
    let params = use_params_map();
    let backend_client = use_context::<backend_client::BackendClient>().unwrap();

    let character_id = {
        params
            .read()
            .get_str("characterid")
            .and_then(|character_id| uuid::Uuid::parse_str(character_id).ok())
    };

    character_id.map(|character_id| {
        view! {
            <Websocket url=backend_client.get_game_ws_url()>
                <GameInstance character_id=character_id />
            </Websocket>
        }
    })
}
