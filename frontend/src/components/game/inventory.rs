use leptos::html::*;
use leptos::prelude::*;

use shared::data::item::{ItemCategory, ItemSpecs};

use crate::assets::img_asset;
use crate::components::ui::{menu_panel::MenuPanel, tooltip::Tooltip};

use super::game_context::GameContext;
use super::player_card::PlayerName;

#[derive(Clone, Debug)]
struct InventoryContext {
    hovered_item: RwSignal<Option<ItemSpecs>>,
}

#[component]
pub fn Inventory(open: RwSignal<bool>) -> impl IntoView {
    let inventory_context = InventoryContext {
        hovered_item: RwSignal::new(None),
    };
    provide_context(inventory_context.clone());

    let show_tooltip = Signal::derive({
        let inventory_context = inventory_context.clone();
        move || inventory_context.hovered_item.get().is_some()
    });

    view! {
        <Tooltip show=show_tooltip>
            {move || {
                inventory_context
                    .hovered_item
                    .get()
                    .map(|item| {
                        view! { <ItemTooltip item_specs=item /> }
                    })
            }}
        </Tooltip>
        <MenuPanel open=open>
            <EquippedItems class:col-span-1 class:justify-self-end />
            <ItemsGrid class:col-span-2 class:justify-self-start />
        </MenuPanel>
    }
}

#[component]
pub fn EquippedItems() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    view! {
        <div class="w-full flex flex-col gap-2 p-2 bg-zinc-800 rounded-md h-full shadow-md ring-1 ring-zinc-950">
            <div>
                <PlayerName />
            </div>
            <div class="grid grid-rows-3 grid-cols-5">
                // TODO: Dynamic + handle None
                <ItemCard item_specs=game_context
                    .player_specs
                    .read()
                    .inventory
                    .weapon_specs
                    .clone()
                    .unwrap_or_default() />
            </div>
        </div>
    }
}

#[component]
fn ItemsGrid() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    view! {
        <div class="bg-zinc-800 rounded-md h-full w-full shadow-lg ring-1 ring-zinc-950">
            <div class="grid grid-rows-3 grid-cols-10 gap-2 p-4">

                <For
                    each=move || {
                        game_context
                            .player_specs
                            .read()
                            .inventory
                            .bag
                            .clone()
                            .into_iter()
                            .enumerate()
                    }
                    // We need a unique key to replace old elements
                    key=move |(index, _)| *index
                    children=move |(_, specs)| {
                        view! { <ItemCard item_specs=specs /> }
                    }
                />
            </div>
        </div>
    }
}
#[component]
// TODO: None
fn ItemCard(item_specs: ItemSpecs) -> impl IntoView {
    let inventory_context = expect_context::<InventoryContext>();
    view! {
        <div
            class="relative group"
            on:mouseenter=move |_| { inventory_context.hovered_item.set(Some(item_specs.clone())) }
            on:mouseleave=move |_| inventory_context.hovered_item.set(None)
        >
            <img src=img_asset(&item_specs.icon) class="border-4 border-stone-500" />
        </div>
    }
}

#[component]
fn ItemTooltip(item_specs: ItemSpecs) -> impl IntoView {
    // TODO: use li etc
    view! {
        <div>
            <strong>{item_specs.name}</strong>
            <br />
            <br />
            {item_specs.item_level}
            <br />
            {match item_specs.item_category {
                ItemCategory::Trinket => {}
                ItemCategory::Weapon(ws) => {
                    view! {
                        <p>
                            "Damages: " {ws.min_damages}- {ws.max_damages} <br /> "Speed: "
                            {ws.cooldown} <br />
                        </p>
                    }
                }
            }}
            <br />
            {item_specs.description}
        </div>
    }
}
