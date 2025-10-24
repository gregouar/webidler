use leptos::{prelude::*, Params};
use leptos_router::{
    hooks::{use_navigate, use_params},
    params::Params,
};

use shared::{data::user::UserCharacterId, http::server::GetCharacterDetailsResponse};

use crate::components::{
    backend_client::{BackendClient, BackendError},
    shared::{
        player_count::PlayerCount,
        resources::{GemsCounter, ShardsCounter},
    },
    town::{
        panels::{ascend::AscendPanel, inventory::TownInventoryPanel},
        town_scene::TownScene,
        TownContext,
    },
    ui::{buttons::MenuButton, fullscreen::FullscreenButton, tooltip::DynamicTooltip},
};

#[derive(Params, PartialEq)]
struct CharacterParams {
    character_id: Option<UserCharacterId>,
}

#[component]
pub fn ViewCharacterPage() -> impl IntoView {
    let town_context = TownContext::default();
    provide_context(town_context);

    let params = use_params::<CharacterParams>();

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

    let fetch_data = {
        let backend = expect_context::<BackendClient>();

        let character_id = params
            .read()
            .as_ref()
            .ok()
            .and_then(|params| params.character_id)
            .unwrap_or_default();

        move || async move {
            match backend.get_character_details(&character_id).await {
                Ok(GetCharacterDetailsResponse {
                    character,
                    areas,
                    inventory,
                    ascension,
                }) => {
                    town_context.character.set(character);
                    town_context.areas.set(areas);
                    town_context.inventory.set(inventory);
                    town_context.passives_tree_ascension.set(ascension);
                }
                Err(BackendError::Unauthorized(_) | BackendError::NotFound) => {
                    use_navigate()("/", Default::default());
                }
                _ => {} // TODO: Toast error ?
            }
        }
    };

    let initial_load = LocalResource::new(fetch_data);

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
                    view! {
                        <HeaderMenu />
                        <div class="relative flex-1">
                            <TownScene view_only=true />
                            <AscendPanel open=town_context.open_ascend view_only=true />
                            <TownInventoryPanel open=town_context.open_inventory view_only=true />
                        </div>
                    }
                })}
            </Transition>

        </main>
    }
}

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let gems = Signal::derive(move || town_context.character.read().resource_gems);
    let shards = Signal::derive(move || town_context.character.read().resource_shards);

    let navigate_quit = {
        let navigate = leptos_router::hooks::use_navigate();
        move |_| {
            navigate("/", Default::default());
        }
    };

    let disable_inventory =
        Signal::derive(move || town_context.character.read().max_area_level == 0);

    view! {
        <div class="relative z-50 w-full flex justify-between items-center p-1 xl:p-2 bg-zinc-800 shadow-md h-auto">
            <div class="flex justify-around w-full items-center">
                <GemsCounter value=gems />
                <ShardsCounter value=shards />
            </div>
            <div class="flex justify-end space-x-1 xl:space-x-2 w-full">
                <FullscreenButton />
                <MenuButton
                    on:click=move |_| {
                        town_context.open_inventory.set(!town_context.open_inventory.get());
                    }
                    disabled=disable_inventory
                >
                    "Inventory"
                </MenuButton>
                <MenuButton
                    on:click=move |_| {
                        town_context.open_ascend.set(!town_context.open_ascend.get());
                        town_context.open_market.set(false);
                        town_context.open_forge.set(false);
                    }
                    disabled=disable_inventory
                >
                    "Ascend"
                </MenuButton>
                <MenuButton on:click=navigate_quit>"Back"</MenuButton>
            </div>
        </div>
    }
}
