use leptos::html::*;
use leptos::prelude::*;
use std::collections::HashSet;

use shared::messages::client::{EquipItemMessage, SellItemsMessage};

use crate::assets::img_asset;
use crate::components::{
    game::item_card::ItemCard,
    ui::{
        buttons::{CloseButton, MenuButton},
        menu_panel::MenuPanel,
        tooltip::DynamicTooltipPosition,
    },
    websocket::WebsocketContext,
};

use super::game_context::GameContext;
use super::player_card::PlayerName;

#[derive(Clone, Default)]
pub struct SellQueue(RwSignal<HashSet<usize>>);

#[component]
pub fn Inventory(open: RwSignal<bool>) -> impl IntoView {
    let sell_queue = SellQueue::default();
    provide_context(sell_queue.clone());

    Effect::new(move || {
        if !open.get() {
            sell_queue.0.write().drain();
        }
    });

    view! {
        <MenuPanel open=open>
            <div class="grid grid-cols-7 justify-items-stretch flex items-start gap-4 p-4">
                <EquippedItems class:col-span-2 class:justify-self-end />
                <ItemsGrid open=open class:col-span-5 class:justify-self-start />
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
fn ItemsGrid(open: RwSignal<bool>) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let total_slots = game_context.player_specs.read().inventory.max_bag_size as usize;

    view! {
        <div class="bg-zinc-800 rounded-md h-full w-full gap-2 p-2 shadow-lg ring-1 ring-zinc-950 relative flex flex-col">
            <div class="px-4 relative z-10 flex items-center justify-between">
                <div>
                    <span class="text-shadow-md shadow-gray-950 text-amber-200 text-xl font-semibold">
                        "Inventory "
                    </span>
                    <span class="text-shadow-md shadow-gray-950 text-gray-400 text-md font-medium">
                        {format!(
                            " ({} / {})",
                            game_context.player_specs.read().inventory.bag.len(),
                            game_context.player_specs.read().inventory.max_bag_size,
                        )}
                    </span>
                </div>

                <SellAllButton />

                <CloseButton on:click=move |_| open.set(false) />
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

    let sell_queue = expect_context::<SellQueue>();
    let is_queued_for_sale = move || sell_queue.0.read().contains(&item_index);

    let show_menu = RwSignal::new(false);

    view! {
        <div class="relative group w-full aspect-[2/3]">
            {move || {
                match maybe_item() {
                    Some(item_specs) => {
                        view! {
                            <div class="relative w-full h-full">
                                <ItemCard
                                    item_specs=item_specs.clone()
                                    on:click=move |_| show_menu.set(true)
                                    tooltip_position=DynamicTooltipPosition::BottomRight
                                />

                                <Show when=is_queued_for_sale>
                                    <div class="absolute top-1 right-1 px-2 py-0.5 text-xs font-semibold bg-red-500 text-white rounded shadow">
                                        "SELL"
                                    </div>
                                </Show>

                                <Show when=move || show_menu.get()>
                                    <ItemContextMenu
                                        item_index=item_index
                                        on_close=Callback::new(move |_| show_menu.set(false))
                                    />
                                </Show>
                            </div>
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
pub fn ItemContextMenu(item_index: usize, on_close: Callback<()>) -> impl IntoView {
    let sell_queue = expect_context::<SellQueue>();

    let equip = {
        let conn = expect_context::<WebsocketContext>();
        move || {
            sell_queue.0.write().remove(&item_index);
            conn.send(
                &EquipItemMessage {
                    item_index: item_index as u8,
                }
                .into(),
            );
            on_close.run(());
        }
    };

    let toggle_sell_mark = {
        move || {
            sell_queue.0.update(|set| {
                if !set.remove(&item_index) {
                    set.insert(item_index);
                }
            });
            on_close.run(());
        }
    };

    view! {
        <style>
            "
            @keyframes fade-in {
                from { opacity: 0; transform: scale(0.95); }
                to { opacity: 1; transform: scale(1); }
            }
            "
        </style>
        <div
            class="
            absolute inset-0 z-30 flex flex-col justify-center items-center
            w-full
            rounded-md  shadow-lg shadow-gray-900
            bg-gradient-to-br from-gray-800/80 via-gray-900/80 to-black
            border border-gray-600 ring-2 ring-gray-700
            text-center
            "
            style="animation: fade-in 0.2s ease-out forwards"
        >
            <button
                class="w-full text-xl font-semibold text-green-300 hover:text-green-100 hover:bg-green-800/40  py-2"
                on:click=move |_| equip()
            >
                "Equip"
            </button>

            <button
                class="w-full text-xl font-semibold text-amber-300 hover:text-amber-100 hover:bg-amber-800/40 py-2"
                on:click=move |_| toggle_sell_mark()
            >
                {if sell_queue.0.get().contains(&item_index) { "Unsell" } else { "Sell" }}
            </button>

            <button
                class="w-full text-lg text-gray-400 hover:text-white hover:bg-gray-400/40 py-4"
                on:click=move |_| on_close.run(())
            >
                "Cancel"
            </button>
        </div>
    }
}

#[component]
fn EmptySlot(children: Children) -> impl IntoView {
    view! {
        <div class="
        relative group flex items-center justify-center w-full h-full
        rounded-md border-2 border-zinc-700 bg-gradient-to-br from-zinc-800 to-zinc-900 opacity-70
        ">{children()}</div>
    }
}

#[component]
fn SellAllButton() -> impl IntoView {
    let sell = {
        let sell_queue = expect_context::<SellQueue>();
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            conn.send(
                &SellItemsMessage {
                    item_indexes: sell_queue.0.write().drain().map(|x| x as u8).collect(),
                }
                .into(),
            );
        }
    };

    let disabled = Signal::derive({
        let sell_queue = expect_context::<SellQueue>();
        move || sell_queue.0.read().is_empty()
    });

    view! {
        <MenuButton on:click=sell disabled=disabled>
            "Sell all marked items"
        </MenuButton>
    }
}
