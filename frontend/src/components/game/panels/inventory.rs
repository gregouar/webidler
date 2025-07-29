use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use strum::IntoEnumIterator;

use leptos::{html::*, prelude::*};
use leptos_use::on_click_outside;

use shared::data::{
    item::{ItemCategory, ItemSlot},
    player::EquippedSlot,
};
use shared::messages::client::{
    EquipItemMessage, FilterLootMessage, SellItemsMessage, UnequipItemMessage,
};

use crate::assets::img_asset;
use crate::components::{
    ui::{
        buttons::{CloseButton, MenuButton},
        confirm::ConfirmContext,
        menu_panel::MenuPanel,
        tooltip::DynamicTooltipPosition,
    },
    websocket::WebsocketContext,
};

use crate::components::game::{
    game_context::GameContext, item_card::ItemCard, player_card::PlayerName, tooltips::ItemTooltip,
};

#[derive(Clone, Default)]
struct SellQueue(RwSignal<HashSet<usize>>);

#[component]
pub fn InventoryPanel(open: RwSignal<bool>) -> impl IntoView {
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
                <EquippedItemsCard class:col-span-2 class:justify-self-end />
                <BagCard open=open class:col-span-5 class:justify-self-start />
            </div>
        </MenuPanel>
    }
}

#[component]
pub fn EquippedItemsCard() -> impl IntoView {
    const EQUIPPED_SLOTS: &[(ItemSlot, &str, &str)] = &[
        (ItemSlot::Accessory, "ui/accessory.webp", "Accessory"),
        (ItemSlot::Helmet, "ui/helmet.webp", "Helmet"),
        (ItemSlot::Amulet, "ui/amulet.webp", "Amulet"),
        (ItemSlot::Weapon, "ui/weapon.webp", "Weapon"),
        (ItemSlot::Body, "ui/shirt.webp", "Body Armor"),
        (ItemSlot::Shield, "ui/shield.webp", "Shield"),
        (ItemSlot::Gloves, "ui/gloves.webp", "Gloves"),
        (ItemSlot::Boots, "ui/boots.webp", "Boots"),
        (ItemSlot::Ring, "ui/ring.webp", "Ring"),
    ];

    view! {
        <div class="w-full flex flex-col gap-2 p-2 bg-zinc-800 rounded-md h-full shadow-xl ring-1 ring-zinc-950">
            <div>
                <PlayerName />
            </div>
            <div class="grid grid-rows-3 grid-cols-3 gap-3 p-4 bg-neutral-900 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                {EQUIPPED_SLOTS
                    .iter()
                    .map(|(slot, asset, alt)| {
                        view! {
                            <EquippedItem item_slot=*slot fallback_asset=*asset fallback_alt=*alt />
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn EquippedItem(
    item_slot: ItemSlot,
    fallback_asset: &'static str,
    fallback_alt: &'static str,
) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let show_menu = RwSignal::new(false);

    let render_fallback = move || {
        view! {
            <EmptySlot>
                <img
                    src=img_asset(fallback_asset)
                    alt=fallback_alt
                    class="object-contain max-w-full max-h-full opacity-20"
                />
            </EmptySlot>
        }
        .into_any()
    };

    let equipped_item = move || {
        game_context
            .player_inventory
            .read()
            .equipped
            .get(&item_slot)
            .cloned()
    };

    view! {
        <div class="relative group w-full aspect-[2/3]">
            {move || match equipped_item() {
                Some(EquippedSlot::MainSlot(item_specs)) => {
                    let rc_item_specs = Arc::new(*item_specs.clone());
                    let is_being_unequipped = RwSignal::new(false);
                    view! {
                        <div class="relative w-full h-full overflow-visible">
                            <ItemCard
                                item_specs=*item_specs
                                on:click=move |_| show_menu.set(true)
                                tooltip_position=DynamicTooltipPosition::Auto
                            />

                            <Show when=move || is_being_unequipped.get()>
                                <div class="absolute inset-0 z-30 w-full rounded-md
                                bg-gradient-to-br from-gray-800/80 via-gray-900/80 to-black"></div>
                            </Show>

                            <Show when=move || show_menu.get()>
                                <EquippedItemContextMenu
                                    item_slot=item_slot
                                    is_being_unequipped=is_being_unequipped
                                    on_close=Callback::new(move |_| show_menu.set(false))
                                />
                                <div class="absolute top-0 right-0 translate-x-full pl-2 whitespace-nowrap z-20 transition-opacity duration-150">
                                    <ItemTooltip item_specs=rc_item_specs.clone() />
                                </div>
                            </Show>
                        </div>
                    }
                        .into_any()
                }
                Some(EquippedSlot::ExtraSlot(main_slot)) => {
                    if let Some(EquippedSlot::MainSlot(item_specs)) = game_context
                        .player_inventory
                        .read()
                        .equipped
                        .get(&main_slot)
                        .cloned()
                    {
                        view! {
                            <EmptySlot>
                                <img
                                    src=img_asset(&item_specs.base.icon)
                                    alt=fallback_alt
                                    class="object-contain max-w-full max-h-full opacity-50"
                                />
                            </EmptySlot>
                        }
                            .into_any()
                    } else {
                        render_fallback()
                    }
                }
                None => render_fallback(),
            }}
        </div>
    }
}

#[component]
pub fn EquippedItemContextMenu(
    item_slot: ItemSlot,
    on_close: Callback<()>,
    is_being_unequipped: RwSignal<bool>,
) -> impl IntoView {
    let confirm_context = expect_context::<ConfirmContext>();

    let unequip = Arc::new({
        let conn = expect_context::<WebsocketContext>();

        move || {
            conn.send(&UnequipItemMessage { item_slot }.into());
            is_being_unequipped.set(true);
            set_timeout(
                move || is_being_unequipped.set(false),
                Duration::from_millis(1000),
            );
        }
    });

    let try_unequip = {
        let game_context = expect_context::<GameContext>();
        move |_| {
            let inventory = game_context.player_inventory.read();
            let need_confirm = inventory
                .equipped
                .get(&item_slot)
                .map(|x| {
                    if let EquippedSlot::MainSlot(x) = x {
                        x.weapon_specs.is_some()
                    } else {
                        false
                    }
                })
                .unwrap_or_default();

            if need_confirm {
                (confirm_context
                        .confirm)(
                        "Unequipping your weapon will reset the weapon attack skill upgrade level to 1, are you sure?"
                            .to_string(),
                        unequip.clone(),
                    );
            } else {
                unequip();
            }
            on_close.run(());
        }
    };

    view! {
        <ContextMenu on_close=on_close>
            <button
                class="w-full text-lg font-semibold text-green-300 hover:text-green-100 hover:bg-green-800/40  py-2"
                on:click=try_unequip
            >
                "Unequip"
            </button>

            <button
                class="w-full text-base text-gray-400 hover:text-white hover:bg-gray-400/40 py-4"
                on:click=move |_| on_close.run(())
            >
                "Cancel"
            </button>
        </ContextMenu>
    }
}

#[component]
fn BagCard(open: RwSignal<bool>) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    view! {
        <div class="bg-zinc-800 rounded-md h-full w-full gap-2 p-2 shadow-lg ring-1 ring-zinc-950 relative flex flex-col">
            <div class="px-4 relative z-10 flex items-center justify-between gap-2 flex-wrap">
                <div class="flex flex-col sm:flex-row sm:items-center gap-1 sm:gap-2">
                    <span class="text-shadow-md shadow-gray-950 text-amber-200 text-xl font-semibold">
                        "Inventory"
                    </span>
                    <span class="text-shadow-md shadow-gray-950 text-gray-400 text-md font-medium">
                        {move || {
                            format!(
                                "({} / {})",
                                game_context.player_inventory.read().bag.len(),
                                game_context.player_inventory.read().max_bag_size,
                            )
                        }}
                    </span>
                </div>

                <div class="flex items-center gap-2">
                    <span class="text-gray-300 text-sm">"Loot Preference:"</span>
                    <LootFilterDropdown />
                </div>

                <div class="flex items-center gap-2">
                    <SellAllButton />
                    <CloseButton on:click=move |_| open.set(false) />
                </div>
            </div>
            // overflow-y-auto
            <div class="relative flex-1 overflow-x-visible max-h-[80vh]">
                <div class="grid grid-cols-5 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10 gap-3 p-4 relative
                bg-neutral-900 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                    <For
                        each=move || (0..game_context.player_inventory.read().max_bag_size as usize)
                        key=|i| *i
                        let(i)
                    >
                        <BagItem item_index=i />
                    </For>
                </div>
            </div>

        </div>
    }
}

#[component]
fn BagItem(item_index: usize) -> impl IntoView {
    let is_being_equipped = RwSignal::new(false);

    let game_context = expect_context::<GameContext>();
    let maybe_item = move || {
        is_being_equipped.set(false);
        game_context
            .player_inventory
            .read()
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
                        let rc_item_specs = Arc::new(item_specs.clone());
                        view! {
                            <div class="relative w-full h-full overflow-visible">
                                <ItemCard
                                    item_specs=item_specs.clone()
                                    on:click=move |_| show_menu.set(true)
                                    tooltip_position=DynamicTooltipPosition::Auto
                                />

                                <Show when=is_queued_for_sale>
                                    <div class="absolute top-1 right-1 px-2 py-0.5 text-xs font-semibold bg-red-500 text-white rounded shadow">
                                        "SELL"
                                    </div>
                                </Show>

                                <Show when=move || is_being_equipped.get()>
                                    <div class="absolute inset-0 z-30 w-full rounded-md
                                    bg-gradient-to-br from-gray-800/80 via-gray-900/80 to-black"></div>
                                </Show>

                                <Show when=move || show_menu.get()>
                                    <BagItemContextMenu
                                        item_index=item_index
                                        on_close=Callback::new(move |_| show_menu.set(false))
                                        is_being_equipped=is_being_equipped
                                    />
                                    <div class="absolute top-0 left-0 -translate-x-full pr-2 whitespace-nowrap z-20 transition-opacity duration-150">
                                        <ItemTooltip item_specs=rc_item_specs.clone() />
                                    </div>
                                </Show>
                            </div>
                        }
                            .into_any()
                    }
                    None => view! { <EmptySlot>{}</EmptySlot> }.into_any(),
                }
            }}
        </div>
    }
}

#[component]
pub fn BagItemContextMenu(
    item_index: usize,
    on_close: Callback<()>,
    is_being_equipped: RwSignal<bool>,
) -> impl IntoView {
    let sell_queue = expect_context::<SellQueue>();
    let confirm_context = expect_context::<ConfirmContext>();

    let equip = Arc::new({
        let conn = expect_context::<WebsocketContext>();
        move || {
            sell_queue.0.write().remove(&item_index);
            conn.send(
                &EquipItemMessage {
                    item_index: item_index as u8,
                }
                .into(),
            );
            is_being_equipped.set(true);
            set_timeout(
                move || is_being_equipped.set(false),
                Duration::from_millis(1000),
            );
        }
    });

    let try_equip = {
        let game_context = expect_context::<GameContext>();
        move |_| {
            let inventory = game_context.player_inventory.read();
            let need_confirm = inventory
                .bag
                .get(item_index)
                .and_then(|x| inventory.equipped.get(&x.base.slot))
                .and_then(|x| match x {
                    EquippedSlot::ExtraSlot(item_slot) => inventory.equipped.get(item_slot),
                    x => Some(x),
                })
                .map(|x| {
                    if let EquippedSlot::MainSlot(x) = x {
                        x.weapon_specs.is_some()
                    } else {
                        false
                    }
                })
                .unwrap_or_default();

            if need_confirm {
                (confirm_context
                        .confirm)(
                        "Equipping a new weapon will reset the weapon attack skill upgrade level to 1, are you sure?"
                            .to_string(),
                        equip.clone(),
                    );
            } else {
                equip();
            }

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
        <ContextMenu on_close=on_close>
            <button
                class="w-full text-lg font-semibold text-green-300 hover:text-green-100 hover:bg-green-800/40  py-2"
                on:click=try_equip
            >
                "Equip"
            </button>

            <button
                class="w-full text-lg font-semibold text-amber-300 hover:text-amber-100 hover:bg-amber-800/40 py-2"
                on:click=move |_| toggle_sell_mark()
            >
                {move || if sell_queue.0.get().contains(&item_index) { "Unsell" } else { "Sell" }}
            </button>

            <button
                class="w-full text-base text-gray-400 hover:text-white hover:bg-gray-400/40 py-4"
                on:click=move |_| on_close.run(())
            >
                "Cancel"
            </button>
        </ContextMenu>
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
pub fn ContextMenu(on_close: Callback<()>, children: Children) -> impl IntoView {
    let node_ref = NodeRef::new();

    let _ = on_click_outside(node_ref, move |_| {
        on_close.run(());
    });

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
            node_ref=node_ref
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
            {children()}
        </div>
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

#[component]
pub fn LootFilterDropdown() -> impl IntoView {
    let node_ref = NodeRef::new();

    let options = std::iter::once(None)
        .chain(ItemCategory::iter().map(Some))
        .collect::<Vec<_>>();

    let is_open = RwSignal::new(false);

    let toggle = move |_| {
        is_open.update(|open| *open = !*open);
    };

    let _ = on_click_outside(node_ref, move |_| {
        is_open.set(false);
    });

    let select_option = {
        let conn = expect_context::<WebsocketContext>();
        let game_context = expect_context::<GameContext>();

        move |opt| {
            game_context.loot_preference.set(opt);
            is_open.set(false);
            conn.send(
                &FilterLootMessage {
                    preferred_loot: opt,
                }
                .into(),
            );
        }
    };

    let game_context = expect_context::<GameContext>();
    view! {
        <style>
            ".dropdown-transition {
            opacity: 0;
            transform: scaleY(0.5);
            transform-origin: top;
            transition: all 150ms ease-out;
            pointer-events: none;
            }
            
            .dropdown-transition.open {
            opacity: 1;
            transform: scaleY(1);
            pointer-events: auto;
            }
            
            ul::-webkit-scrollbar {
            width: 8px;
            }
            
            ul::-webkit-scrollbar-track {
            background: #1f1f1f;
            border-radius: 4px;
            }
            
            ul::-webkit-scrollbar-thumb {
            background-color: #525252;
            border-radius: 4px;
            border: 2px solid #1f1f1f;
            }
            
            ul {
            scrollbar-width: thin;
            scrollbar-color: #525252 #1f1f1f;
            }
            
            ul::-webkit-scrollbar-thumb:hover {
            background-color: #737373;
            }
            "
        </style>
        <div class="relative w-60 z-20">
            <button
                on:click=toggle
                class="w-full text-left px-4 py-2 rounded-md text-white bg-gradient-to-t from-zinc-900 to-zinc-800 shadow-md border border-zinc-950 hover:from-zinc-800 hover:to-zinc-700 focus:outline-none"
            >
                {move || loot_filter_category_to_str(game_context.loot_preference.get())}
                <span class="float-right">"▼"</span>
            </button>

            <ul
                class=move || {
                    format!(
                        "dropdown-transition absolute mt-1 w-full rounded-md bg-zinc-800 border border-zinc-950 shadow-lg max-h-80 overflow-auto {}",
                        if is_open.get() { "open" } else { "" },
                    )
                }
                node_ref=node_ref
            >
                {options
                    .iter()
                    .cloned()
                    .map(|opt| {
                        view! {
                            <li
                                on:click={
                                    let select_option = select_option.clone();
                                    move |_| select_option(opt)
                                }
                                class="cursor-pointer px-4 py-2 hover:bg-zinc-700 text-white"
                            >
                                {loot_filter_category_to_str(opt)}
                            </li>
                        }
                    })
                    .collect::<Vec<_>>()}
            </ul>
        </div>
    }
}

fn loot_filter_category_to_str(opt: Option<ItemCategory>) -> &'static str {
    match opt {
        Some(item_category) => match item_category {
            ItemCategory::Armor => "Any Armor",
            ItemCategory::AttackWeapon => "Any Attack Weapon",
            ItemCategory::SpellWeapon => "Any Spell Weapon",
            ItemCategory::MeleeWeapon => "Any Melee Weapon",
            ItemCategory::RangedWeapon => "Any Ranged Weapon",
            ItemCategory::Shield => "Shield",
            ItemCategory::Focus => "Magical Focus",
            ItemCategory::Jewelry => "Any Jewelry",
            ItemCategory::Accessory => "Any Accessory",
            ItemCategory::Body => "Body Armor",
            ItemCategory::Boots => "Boots",
            ItemCategory::Cloak => "Cloak",
            ItemCategory::Gloves => "Gloves",
            ItemCategory::Helmet => "Helmet",
            ItemCategory::Ring => "Ring",
            ItemCategory::Amulet => "Amulet",
            ItemCategory::Relic => "Relic",
        },
        None => "Any Item",
    }
}
