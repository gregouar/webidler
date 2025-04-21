use leptos::html::*;
use leptos::prelude::*;

use crate::assets::img_asset;
use crate::components::ui::tooltip::Tooltip;

use super::menu_panel::MenuPanel;
use super::player_card::PlayerName;

// TODO: Move
#[derive(Clone, Debug)]
pub struct Item {
    name: String,
    description: String,
    icon: String,
}

#[derive(Clone, Debug)]
struct InventoryContext {
    hovered_item: RwSignal<Option<Item>>,
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
                        view! { <ItemTooltip item=item /> }
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
    view! {
        <div class="w-full flex flex-col gap-2 p-2 bg-zinc-800 rounded-md h-full shadow-md ring-1 ring-zinc-950">
            <div>
                <PlayerName />
            </div>
            <div class="grid grid-rows-3 grid-cols-5">
                <ItemCard item=Item {
                    name: "Shortsword".to_string(),
                    description: "Fasty Slicy".to_string(),
                    icon: "items/shortsword.webp".to_string(),
                } />
            </div>
        </div>
    }
}

#[component]
fn ItemsGrid() -> impl IntoView {
    view! {
        <div class="bg-zinc-800 rounded-md h-full w-full shadow-lg ring-1 ring-zinc-950">
            <div class="grid grid-rows-3 grid-cols-10 gap-2 p-4">
                <ItemCard item=Item {
                    name: "Battleaxe".to_string(),
                    description: "A shiny thing".to_string(),
                    icon: "items/battleaxe.webp".to_string(),
                } />
            </div>
        </div>
    }
}
#[component]
fn ItemCard(item: Item) -> impl IntoView {
    let inventory_context = expect_context::<InventoryContext>();
    view! {
        <div
            class="relative group"
            on:mouseenter=move |_| { inventory_context.hovered_item.set(Some(item.clone())) }
            on:mouseleave=move |_| inventory_context.hovered_item.set(None)
        >
            <img src=img_asset(&item.icon) class="border-4 border-stone-500" />
        </div>
    }
}

#[component]
fn ItemTooltip(item: Item) -> impl IntoView {
    view! {
        <div>
            <strong>{item.name}</strong>
            <br />
            {item.description}
        </div>
    }
}
