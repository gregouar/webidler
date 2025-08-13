use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::data::user::UserCharacterId;

use crate::components::{
    backend_client::BackendClient,
    town::{header_menu::HeaderMenu, town_scene::TownScene},
    ui::tooltip::DynamicTooltip,
};

#[component]
pub fn TownPage() -> impl IntoView {
    let (get_character_id_storage, _, _) =
        storage::use_session_storage::<UserCharacterId, JsonSerdeCodec>("character_id");

    let (get_jwt_storage, _, _) = storage::use_local_storage::<String, JsonSerdeCodec>("jwt");

    let refresh_trigger = RwSignal::new(0u64);

    let character_and_areas = LocalResource::new({
        let backend = use_context::<BackendClient>().unwrap();
        let refresh_trigger = refresh_trigger.clone();
        move || async move {
            let _ = refresh_trigger.read();

            backend
                .get_character_details(&get_jwt_storage.get(), &get_character_id_storage.get())
                .await
                .map(|response| (response.character, response.areas))
                .ok()
        }
    });

    Effect::new({
        let navigate = use_navigate();
        move || {
            if character_and_areas
                .get()
                .map(|x| x.is_none())
                .unwrap_or_default()
            {
                navigate("/", Default::default());
            }
        }
    });

    view! {
        <main class="my-0 mx-auto w-full text-center overflow-x-hidden flex flex-col min-h-screen">
            <DynamicTooltip />
            <Transition fallback=move || {
                view! { <p class="text-gray-400">"Loading..."</p> }
            }>
                {move || {
                    Suspend::new(async move {
                        let (character, areas) = character_and_areas.await.unwrap_or_default();

                        view! {
                            <HeaderMenu />
                            <div class="relative flex-1">
                                <TownScene character=character areas=areas />
                            // <MarketPanel open=town_context.open_market />
                            // <AscendPanel open=town_context.open_ascend />
                            // <ForgePanel open=town_context.open_forge />
                            </div>
                        }
                    })
                }}
            </Transition>

        </main>
    }
}
