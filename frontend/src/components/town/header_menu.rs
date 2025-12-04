use leptos::{html::*, prelude::*};

use crate::components::{
    shared::resources::{GemsCounter, GoldCounter, ShardsCounter},
    town::TownContext,
    ui::{buttons::MenuButton, fullscreen::FullscreenButton, wiki::WikiButton},
};

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let gold = Signal::derive(move || town_context.character.read().resource_gold);
    let gems = Signal::derive(move || town_context.character.read().resource_gems);
    let shards = Signal::derive(move || town_context.character.read().resource_shards);

    let navigate_quit = {
        let navigate = leptos_router::hooks::use_navigate();
        move |_| {
            navigate("/user-dashboard", Default::default());
        }
    };

    let disable_panels = Signal::derive(move || town_context.character.read().max_area_level == 0);

    view! {
        <div class="relative z-50 w-full flex justify-between items-center p-1 xl:p-2 bg-zinc-800 shadow-md h-auto">
            <div class="flex justify-around w-full items-center">
                <GoldCounter value=gold />
                <GemsCounter value=gems />
                <ShardsCounter value=shards />
            </div>
            <div class="flex justify-end space-x-1 xl:space-x-2 w-full">
                <FullscreenButton />
                <WikiButton />
                <MenuButton
                    on:click=move |_| {
                        town_context.open_inventory.set(!town_context.open_inventory.get());
                        town_context.open_ascend.set(false);
                        town_context.open_temple.set(false);
                    }
                    disabled=disable_panels
                >
                    "Inventory"
                </MenuButton>
                <MenuButton
                    on:click=move |_| {
                        town_context.open_stash.set(!town_context.open_stash.get());
                        town_context.open_ascend.set(false);
                        town_context.open_market.set(false);
                        town_context.open_forge.set(false);
                        town_context.open_temple.set(false);
                        town_context.open_inventory.set(false);
                    }
                    disabled=disable_panels
                >
                    "Stash"
                </MenuButton>
                <MenuButton
                    on:click=move |_| {
                        town_context.open_temple.set(!town_context.open_temple.get());
                        town_context.open_ascend.set(false);
                        town_context.open_market.set(false);
                        town_context.open_forge.set(false);
                        town_context.open_inventory.set(false);
                        town_context.open_stash.set(false);
                    }
                    disabled=disable_panels
                >
                    "Temple"
                </MenuButton>
                <MenuButton
                    on:click=move |_| {
                        town_context.open_market.set(!town_context.open_market.get());
                        town_context.open_ascend.set(false);
                        town_context.open_forge.set(false);
                        town_context.open_temple.set(false);
                        town_context.open_inventory.set(false);
                        town_context.open_stash.set(false);
                    }
                    disabled=disable_panels
                >
                    "Market"
                </MenuButton>
                <MenuButton
                    on:click=move |_| {
                        town_context.open_forge.set(!town_context.open_forge.get());
                        town_context.open_market.set(false);
                        town_context.open_ascend.set(false);
                        town_context.open_temple.set(false);
                        town_context.open_inventory.set(false);
                        town_context.open_stash.set(false);
                    }
                    disabled=disable_panels
                >
                    "Forge"
                </MenuButton>
                <MenuButton
                    on:click=move |_| {
                        town_context.open_ascend.set(!town_context.open_ascend.get());
                        town_context.open_market.set(false);
                        town_context.open_forge.set(false);
                        town_context.open_temple.set(false);
                        town_context.open_inventory.set(false);
                        town_context.open_stash.set(false);
                    }
                    disabled=disable_panels
                >
                    "Ascend"
                </MenuButton>
                <MenuButton on:click=navigate_quit>"Quit"</MenuButton>
            </div>
        </div>
    }
}
