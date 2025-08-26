use codee::string::JsonSerdeCodec;
use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use leptos_use::{storage, use_interval_fn};

use shared::data::user::UserCharacterId;

use crate::components::{
    backend_client::{BackendClient, BackendError},
    town::{
        header_menu::HeaderMenu, panels::ascend::AscendPanel, town_scene::TownScene, TownContext,
    },
    ui::tooltip::DynamicTooltip,
};

#[component]
pub fn TownPage() -> impl IntoView {
    let town_context = TownContext::new();
    provide_context(town_context);

    let (get_character_id_storage, _, _) =
        storage::use_session_storage::<UserCharacterId, JsonSerdeCodec>("character_id");

    let (get_jwt_storage, _, _) = storage::use_local_storage::<String, JsonSerdeCodec>("jwt");

    let passives_tree_specs = LocalResource::new({
        let backend = use_context::<BackendClient>().unwrap();
        move || async move {
            backend
                .get_passives()
                .await
                .map(|response| response.passives_tree_specs)
                .unwrap_or_default()
        }
    });

    let fetch_data = {
        let backend = use_context::<BackendClient>().unwrap();

        move || async move {
            match backend
                .get_character_details(&get_jwt_storage.get(), &get_character_id_storage.get())
                .await
            {
                Ok(response) => {
                    town_context.character.set(response.character);
                    town_context.areas.set(response.areas);
                    town_context
                        .passives_tree_state
                        .set(response.passives_tree_state);
                }
                Err(BackendError::Unauthorized(_) | BackendError::NotFound) => {
                    use_navigate()("/", Default::default())
                }
                _ => {} // TODO: Toast error ?
            }
        }
    };

    let initial_load = LocalResource::new(move || fetch_data());

    use_interval_fn(move || spawn_local(fetch_data()), 5_000);

    view! {
        <main class="my-0 mx-auto w-full text-center overflow-x-hidden flex flex-col min-h-screen">
            <DynamicTooltip />

            <Transition fallback=move || {
                view! { <p class="text-gray-400">"Loading..."</p> }
            }>
                {move || Suspend::new(async move {
                    initial_load.await;
                    town_context.passives_tree_specs.set(passives_tree_specs.await);
                    view! {
                        <HeaderMenu />
                        <div class="relative flex-1">
                            <TownScene />
                            // <MarketPanel open=town_context.open_market />
                            <AscendPanel open=town_context.open_ascend />
                        // <ForgePanel open=town_context.open_forge />
                        </div>
                    }
                })}
            </Transition>

        </main>
    }
}
