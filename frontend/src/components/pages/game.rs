use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::components::{
    backend_client::BackendClient, data_context::DataContext, game::game_instance::GameInstance,
    websocket::Websocket,
};

#[component]
pub fn GamePage() -> impl IntoView {
    let backend: BackendClient = expect_context();
    let data_context: DataContext = expect_context();

    let data_load = LocalResource::new({
        move || async move {
            if data_context.load_data(backend).await.is_err() {
                use_navigate()("/", Default::default());
            }
        }
    });

    view! {
        <Transition fallback=move || {
            view! { <p class="text-gray-400">"Loading..."</p> }
        }>
            {move || Suspend::new(async move {
                data_load.await;
                view! {
                    <Websocket url=backend.get_game_ws_url()>
                        <GameInstance />
                    </Websocket>
                }
            })}
        </Transition>
    }
}
