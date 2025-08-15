use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_use::{storage, use_interval_fn};

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
        move || async move {
            let _ = refresh_trigger.read();

            backend
                .get_character_details(&get_jwt_storage.get(), &get_character_id_storage.get())
                .await
                .map(|response| (response.character, response.areas))
                .ok()
        }
    });

    let _ = use_interval_fn(
        move || {
            refresh_trigger.update(|n| *n += 1);
        },
        5_000,
    );

    Effect::new(move |_| {
        character_and_areas.with(|data| {
            if let Some(send_wrapper) = data {
                if let Some((character, _)) = send_wrapper.as_ref() {
                    let _ = character;
                    // TODO: update state and menu if not in town, disable stuff, need to trigger some polling refresh?
                    // if let UserCharacterActivity::Grinding(_, _) = character.activity {
                    //     // If character in game, we redirect to game
                    //     use_navigate()("/game", Default::default());
                    // }
                } else {
                    // If no character, we redirect to main menu
                    use_navigate()("/", Default::default());
                }
            }
        });
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
                        // TODO: arc it

                        view! {
                            <HeaderMenu character=character.clone() />
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
