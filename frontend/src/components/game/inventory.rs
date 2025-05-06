use leptos::html::*;
use leptos::prelude::*;

use shared::messages::client::EquipItemMessage;

use crate::assets::img_asset;
use crate::components::game::item_card::ItemCard;
use crate::components::ui::menu_panel::MenuPanel;
use crate::components::ui::tooltip::DynamicTooltipPosition;
use crate::components::websocket::WebsocketContext;

use super::game_context::GameContext;
use super::player_card::PlayerName;

#[component]
pub fn Inventory(open: RwSignal<bool>) -> impl IntoView {
    view! {
        <MenuPanel open=open>
            <div class="grid grid-cols-7 justify-items-stretch flex items-start gap-4 p-4">
                <EquippedItems class:col-span-2 class:justify-self-end />
                <ItemsGrid class:col-span-5 class:justify-self-start />
            </div>
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
            <div class="grid grid-rows-3 grid-cols-3 gap-3 p-4 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                <EmptySlot>{()}</EmptySlot>
                <EmptySlot>
                    <img
                        src=img_asset("ui/head2.webp")
                        alt="Head"
                        class="object-contain max-w-full max-h-full"
                    />
                </EmptySlot>
                <EmptySlot>{()}</EmptySlot>
                {move || match &game_context.player_specs.read().inventory.weapon_specs {
                    Some(weapon) => {
                        view! {
                            <ItemCard
                                item_specs=weapon.clone()
                                tooltip_position=DynamicTooltipPosition::BottomRight
                            />
                        }
                            .into_any()
                    }
                    None => view! { <EmptySlot>{()}</EmptySlot> }.into_any(),
                }}
                <EmptySlot>{()}</EmptySlot>
                <EmptySlot>{()}</EmptySlot>
                <EmptySlot>{()}</EmptySlot>
                <EmptySlot>{()}</EmptySlot>
                <EmptySlot>{()}</EmptySlot>
            </div>
        </div>
    }
}

#[component]
fn ItemsGrid() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let total_slots = game_context.player_specs.read().inventory.max_bag_size as usize;

    view! {
        <div class="bg-zinc-800 rounded-md h-full w-full gap-2 p-2 shadow-lg ring-1 ring-zinc-950 overflow-hidden relative flex flex-col">
            <div class="px-4 relative z-10 flex items-center justify-between">
                <p class="text-shadow-md shadow-gray-950 text-amber-200 text-xl font-semibold">
                    "Inventory"
                </p>

                <p class="text-shadow-md shadow-gray-950 text-gray-400 text-md font-medium">
                    {format!(
                        "{} / {}",
                        game_context.player_specs.read().inventory.bag.len(),
                        game_context.player_specs.read().inventory.max_bag_size,
                    )}
                </p>
            </div>

            <div class="relative flex-1 overflow-y-auto max-h-[80vh]">
                <div class="grid grid-cols-5 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10 gap-3 p-4 relative shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                    <For each=move || (0..total_slots) key=|i| *i let(i)>
                        <ItemInBag item_index=i />
                    </For>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ItemInBag(item_index: usize) -> impl IntoView {
    let game_context = expect_context::<GameContext>();
    let maybe_item = move || {
        game_context
            .player_specs
            .read()
            .inventory
            .bag
            .get(item_index)
            .cloned()
    };

    view! {
        <div class="group relative w-full aspect-[2/3]">
            {move || {
                match maybe_item() {
                    Some(item_specs) => {
                        let conn = expect_context::<WebsocketContext>();
                        let equip_item = move |_| {
                            conn.send(
                                &EquipItemMessage {
                                    item_index: item_index as u8,
                                }
                                    .into(),
                            );
                        };
                        view! {
                            <ItemCard
                                item_specs=item_specs
                                on:click=equip_item
                                tooltip_position=DynamicTooltipPosition::BottomRight
                            />
                        }
                            .into_any()
                    }
                    None => view! { <EmptySlot>{()}</EmptySlot> }.into_any(),
                }
            }}
        </div>
    }
}

#[component]
fn EmptySlot(children: Children) -> impl IntoView {
    view! {
        // "relative group rounded-md p-1 bg-gradient-to-br {} border-4 {} ring-2 {} shadow-md {}",
        <div class="
        relative group flex items-center justify-center w-full h-full
        rounded-md border-2 border-zinc-700 bg-gradient-to-br from-zinc-800 to-zinc-900 opacity-70
        ">{children()}</div>
    }
}
