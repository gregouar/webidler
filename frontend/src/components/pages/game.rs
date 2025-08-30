use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
// use leptos_router::hooks::use_params_map;
use leptos_use::storage;
use shared::data::user::UserCharacterId;

use crate::components::backend_client;
use crate::components::game::game_instance::GameInstance;
use crate::components::websocket::Websocket;

#[component]
pub fn GamePage() -> impl IntoView {
    // let params = use_params_map();
    let backend_client = expect_context::<backend_client::BackendClient>();

    // let character_id = {
    //     params
    //         .read()
    //         .get_str("characterid")
    //         .and_then(|character_id| uuid::Uuid::parse_str(character_id).ok())
    // };

    let (get_character_id_storage, _, _) =
        storage::use_session_storage::<UserCharacterId, JsonSerdeCodec>("character_id");

    view! {
        <Websocket url=backend_client.get_game_ws_url()>
            <GameInstance character_id=get_character_id_storage.get_untracked() />
        </Websocket>
    }
}
