use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_use::storage;

use shared::{
    data::user::{UserCharacterActivity, UserCharacterId},
    http::server::GetCharacterDetailsResponse,
};

use crate::components::{
    backend_client::{BackendClient, BackendError},
    shared::player_count::PlayerCount,
    town::{
        header_menu::HeaderMenu,
        panels::{
            ascend::AscendPanel, forge::ForgePanel, inventory::TownInventoryPanel,
            market::MarketPanel, temple::TemplePanel,
        },
        town_scene::TownScene,
        TownContext,
    },
    ui::tooltip::DynamicTooltip,
};

#[component]
pub fn TownPage() -> impl IntoView {
    let town_context = TownContext::default();
    provide_context(town_context);

    let (get_character_id_storage, _, _) =
        storage::use_session_storage::<UserCharacterId, JsonSerdeCodec>("character_id");

    let passives_tree_specs = LocalResource::new({
        let backend = expect_context::<BackendClient>();
        move || async move {
            backend
                .get_passives()
                .await
                .map(|response| response.passives_tree_specs)
                .unwrap_or_default()
        }
    });

    let benedictions_specs = LocalResource::new({
        let backend = expect_context::<BackendClient>();
        move || async move {
            backend
                .get_benedictions()
                .await
                .map(|response| response.benedictions_specs)
                .unwrap_or_default()
        }
    });

    let initial_load = LocalResource::new({
        let backend = expect_context::<BackendClient>();

        move || async move {
            match backend
                .get_character_details(&get_character_id_storage.get())
                .await
            {
                Ok(GetCharacterDetailsResponse {
                    character,
                    areas,
                    inventory,
                    ascension,
                    benedictions,
                    last_grind,
                }) => {
                    if let UserCharacterActivity::Grinding(_, _) = character.activity {
                        use_navigate()("/game", Default::default())
                    }
                    town_context.character.set(character);
                    town_context.areas.set(areas);
                    town_context.inventory.set(inventory);
                    town_context.passives_tree_ascension.set(ascension);
                    town_context.player_benedictions.set(benedictions);
                    town_context.last_grind.set(last_grind);
                }
                Err(BackendError::Unauthorized(_) | BackendError::NotFound) => {
                    use_navigate()("/", Default::default())
                }
                _ => {} // TODO: Toast error ?
            }
        }
    });

    view! {
        <main class="my-0 mx-auto w-full text-center overflow-x-hidden flex flex-col min-h-screen">
            <DynamicTooltip />
            <PlayerCount />

            <Transition fallback=move || {
                view! { <p class="text-gray-400">"Loading..."</p> }
            }>
                {move || Suspend::new(async move {
                    initial_load.await;
                    town_context.passives_tree_specs.set(passives_tree_specs.await);
                    town_context.benedictions_specs.set(benedictions_specs.await);
                    view! {
                        <HeaderMenu />
                        <div class="relative flex-1">
                            <TownScene />
                            <TemplePanel open=town_context.open_temple />
                            <MarketPanel open=town_context.open_market />
                            <AscendPanel open=town_context.open_ascend />
                            <ForgePanel open=town_context.open_forge />
                            <TownInventoryPanel open=town_context.open_inventory />
                        </div>
                    }
                })}
            </Transition>

        </main>
    }
}
