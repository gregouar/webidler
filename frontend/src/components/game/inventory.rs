use leptos::html::*;
use leptos::prelude::*;

use shared::data::item::{
    ItemCategory, ItemRarity, ItemSpecs, WeaponMagicPrefix, WeaponMagicSuffix,
};

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
                <EmptySlot />
                <EmptySlot />
                <EmptySlot />
                {move || match &game_context.player_specs.read().inventory.weapon_specs {
                    Some(weapon) => view! { <ItemCard item_specs=weapon.clone() /> }.into_any(),
                    None => view! { <EmptySlot /> }.into_any(),
                }}
                <EmptySlot />
                <EmptySlot />
                <EmptySlot />
                <EmptySlot />
                <EmptySlot />
            </div>
        </div>
    }
}

#[component]
fn ItemsGrid() -> impl IntoView {
    let game_context = expect_context::<GameContext>();
    let inventory = move || game_context.player_specs.read().inventory.bag.clone();

    let total_slots = game_context.player_specs.read().inventory.max_bag_size as usize;

    view! {
        <div class="bg-zinc-800 rounded-md h-full w-full shadow-lg ring-1 ring-zinc-950 overflow-hidden relative gap-2 p-2">

            <div class="px-4 mb-2 relative z-10 flex items-center justify-between">
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
            <div class="grid grid-cols-5 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10 gap-3 p-4 relative z-10 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                <For
                    each=move || (0..total_slots)
                    key=|i| *i
                    children=move |i| {
                        let maybe_item = inventory().get(i).cloned();
                        view! {
                            <div class="group relative w-full aspect-[2/3]">
                                {maybe_item
                                    .map(|specs| view! { <ItemCard item_specs=specs /> }.into_any())
                                    .unwrap_or_else(|| view! { <EmptySlot /> }.into_any())}
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component]
fn EmptySlot() -> impl IntoView {
    view! {
        <div class="w-full h-full rounded-md border-2 border-zinc-700 bg-gradient-to-br from-zinc-800 to-zinc-900 opacity-70" />
    }
}

#[component]
fn ItemCard(item_specs: ItemSpecs) -> impl IntoView {
    let inventory_context = expect_context::<InventoryContext>();

    let (border_color, ring_color, shadow_color, gradient) = match item_specs.rarity {
        ItemRarity::Normal => (
            "border-gray-600/70",
            "ring-gray-600/20",
            "shadow-gray-800/20",
            "from-gray-800 to-gray-950",
        ),
        ItemRarity::Magic => (
            "border-blue-500/70",
            "ring-blue-400/20",
            "shadow-blue-700/20",
            "from-blue-900/50 to-gray-950",
        ),
        ItemRarity::Rare => (
            "border-yellow-400/70",
            "ring-yellow-300/20",
            "shadow-yellow-600/20",
            "from-yellow-900/50 to-gray-950",
        ),
        ItemRarity::Unique => (
            "border-amber-700/70",
            "ring-amber-600/30",
            "shadow-amber-700/30",
            "from-amber-900/50 to-gray-950",
        ),
    };

    view! {
        <div
            class=format!(
                "relative group rounded-md p-1 bg-gradient-to-br {} border-4 {} ring-2 {} shadow-md {}",
                gradient,
                border_color,
                ring_color,
                shadow_color,
            )
            on:mouseenter=move |_| { inventory_context.hovered_item.set(Some(item_specs.clone())) }
            on:mouseleave=move |_| inventory_context.hovered_item.set(None)
        >
            <img
                src=img_asset(&item_specs.icon)
                class="rounded-md object-cover transition-all duration-150 ease-in-out group-hover:scale-105 group-hover:brightness-110"
            />
        </div>
    }
}

fn magic_affix_li(text: String) -> impl IntoView {
    view! { <li class="text-blue-400 text-sm leading-snug">{text}</li> }
}

#[component]
fn ItemTooltip(item_specs: ItemSpecs) -> impl IntoView {
    let extra_info = match &item_specs.item_category {
        ItemCategory::Trinket => {
            view! { <li class="text-gray-400 text-sm leading-snug">"Trinket"</li> }.into_any()
        }
        ItemCategory::Weapon(ws) => view! {
            <li class="text-gray-400 text-sm leading-snug">"Weapon"</li>
            <li class="text-gray-400 text-sm leading-snug">
                "Physical Damage: "
                <span class="text-white font-semibold">
                    {ws.min_damages} {" - "} {ws.max_damages}
                </span>
            </li>
            <li class="text-gray-400 text-sm leading-snug">
                "Attacks per Second: "
                <span class="text-white font-semibold">{format!("{:.2}", ws.cooldown)}</span>
            </li>
        }
        .into_any(),
    };

    let (prefixes, suffixes): (Vec<_>, Vec<_>) = match &item_specs.item_category {
        ItemCategory::Weapon(ws) => (
            ws.magic_prefixes
                .iter()
                .map(|prefix| match prefix {
                    WeaponMagicPrefix::AttackSpeed(v) => {
                        magic_affix_li(format!("Increased Attack Speed: +{:.0}%", v * 100.0))
                    }
                    WeaponMagicPrefix::AttackDamages(v) => {
                        magic_affix_li(format!("Increased Attack Damage: +{:.0}%", v * 100.0))
                    }
                })
                .collect(),
            ws.magic_suffixes
                .iter()
                .map(|suffix| match suffix {
                    WeaponMagicSuffix::GoldFind(v) => {
                        magic_affix_li(format!("Increased Gold Find: +{:.0}%", v * 100.0))
                    }
                })
                .collect(),
        ),
        _ => (vec![], vec![]),
    };

    let has_affixes = !prefixes.is_empty() || !suffixes.is_empty();

    // Color setups
    let (name_color, border_color, ring_color, shadow_color) = match item_specs.rarity {
        ItemRarity::Normal => (
            "text-white",
            "border-gray-600",
            "ring-gray-700",
            "shadow-gray-800",
        ),
        ItemRarity::Magic => (
            "text-blue-400",
            "border-blue-500",
            "ring-blue-400",
            "shadow-blue-700",
        ),
        ItemRarity::Rare => (
            "text-yellow-400",
            "border-yellow-400",
            "ring-yellow-300",
            "shadow-yellow-600",
        ),
        ItemRarity::Unique => (
            "text-amber-700",
            "border-amber-700",
            "ring-amber-600",
            "shadow-amber-700",
        ),
    };

    view! {
        <div class=format!(
            "max-w-xs p-4 rounded-xl border {} ring-2 {} shadow-md {} bg-gradient-to-br from-gray-800 via-gray-900 to-black space-y-2",
            border_color,
            ring_color,
            shadow_color,
        )>
            <strong class=format!(
                "text-lg font-bold {}",
                name_color,
            )>{item_specs.name.clone()}</strong>
            <hr class="border-t border-gray-700" />
            <p class="text-sm text-gray-400 leading-snug">
                "Item Level: " <span class="text-white">{item_specs.item_level}</span>
            </p>
            <hr class="border-t border-gray-700" />
            <ul class="list-none space-y-1">{extra_info}</ul>
            {has_affixes.then(|| view! { <hr class="border-t border-gray-700 my-1" /> })}
            <ul class="list-none space-y-1">{prefixes}{suffixes}</ul>
            <hr class="border-t border-gray-700" />
            <p class="text-sm italic text-gray-300 leading-snug">
                {item_specs.description.clone()}
            </p>
        </div>
    }
}
