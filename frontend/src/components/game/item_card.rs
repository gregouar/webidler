use std::sync::Arc;

use leptos::html::*;
use leptos::prelude::*;

use shared::data::{
    item::{ItemRarity, ItemSlot, ItemSpecs},
    item_affix::{AffixEffect, AffixEffectType, ItemStat},
};

use crate::assets::img_asset;
use crate::components::ui::tooltip::{DynamicTooltipContext, DynamicTooltipPosition};

#[component]
pub fn ItemCard(item_specs: ItemSpecs, tooltip_position: DynamicTooltipPosition) -> impl IntoView {
    let (border_color, ring_color, shadow_color, gradient) = match item_specs.rarity {
        ItemRarity::Normal => (
            "border-gray-600/70",
            "ring-gray-600/20",
            "shadow-gray-800/20",
            "from-gray-900/80 to-gray-950",
        ),
        ItemRarity::Magic => (
            "border-blue-500/70",
            "ring-blue-400/20",
            "shadow-blue-700/20",
            "from-blue-900/80 to-gray-950",
        ),
        ItemRarity::Rare => (
            "border-yellow-400/70",
            "ring-yellow-300/20",
            "shadow-yellow-600/20",
            "from-yellow-900/80 to-gray-950",
        ),
        ItemRarity::Unique => (
            "border-amber-700/70",
            "ring-amber-600/30",
            "shadow-amber-700/30",
            "from-amber-900/80 to-gray-950",
        ),
    };

    let icon_asset = img_asset(&item_specs.base.icon);

    let tooltip_context = expect_context::<DynamicTooltipContext>();
    let rc_item_specs = Arc::new(item_specs.clone());
    // let rc_item_specs2 = rc_item_specs.clone();
    let show_tooltip = move |_| {
        let item_specs = rc_item_specs.clone();
        tooltip_context.set_content(
            move || {
                let item_specs = item_specs.clone();
                view! { <ItemTooltip item_specs=item_specs /> }.into_any()
            },
            tooltip_position,
        );
    };

    // let tooltip_context = expect_context::<DynamicTooltipContext>();
    // let show_tooltip2 = move |_| {
    //     let item_specs = rc_item_specs2.clone();
    //     tooltip_context.set_content(
    //         move || {
    //             let item_specs = item_specs.clone();
    //             view! { <ItemTooltip item_specs=item_specs /> }.into_any()
    //         },
    //         tooltip_position,
    //     );
    // };

    let hide_tooltip = {
        let tooltip_context = expect_context::<DynamicTooltipContext>();
        move |_| tooltip_context.hide()
    };

    // let hide_tooltip2 = {
    //     let tooltip_context = expect_context::<DynamicTooltipContext>();
    //     move |_| tooltip_context.hide()
    // };

    // let el_ref = NodeRef::new();

    // let mouse = use_mouse();
    // let mouse_ref = mouse.x.get();

    // let mouse = use_mouse(); // TODO: How to retrieve last known position without being registered for tracking?
    // let on_mount = move |el: NodeRef<Div>| {
    //     let el = el.get().unwrap();
    //     let rect = el.get_bounding_client_rect();
    //     let (x, y) = (mouse.x.get_untracked(), mouse.y.get_untracked());

    //     logging::error!("{},{}", mouse_ref, mouse_ref);
    //     logging::error!("{},{}", x, y);
    //     logging::error!(
    //         "{},{},{},{}",
    //         rect.left(),
    //         rect.right(),
    //         rect.top(),
    //         rect.bottom()
    //     );

    //     if x >= rect.left() && x <= rect.right() && y >= rect.top() && y <= rect.bottom() {
    //         show_tooltip2(());
    //     };
    // };

    // Effect::new(move || {
    //     on_mount(el_ref);
    // });

    view! {
        <div
            // node_ref=el_ref
            class=format!(
                "relative group flex items-center justify-center w-full aspect-[2/3]
                rounded-md p-1 bg-gradient-to-br {} border-4 {} ring-2 {} shadow-md {}
                ",
                gradient,
                border_color,
                ring_color,
                shadow_color,
            )
            on:mouseenter=show_tooltip
            on:mouseleave=hide_tooltip
        >
            // on:click=hide_tooltip2
            <img
                src=icon_asset
                class="object-contain max-w-full max-h-full transition-all duration-50 ease-in-out
                group-hover:scale-105 group-hover:brightness-110
                group-active:scale-90 group-active:brightness-90
                "
            />
        </div>
    }
}

#[component]
pub fn ItemTooltip(item_specs: Arc<ItemSpecs>) -> impl IntoView {
    let item_slot = match &item_specs.base.item_slot {
        ItemSlot::Amulet => "Amulet",
        ItemSlot::Body => "Body Armor",
        ItemSlot::Boots => "Boots",
        ItemSlot::Gloves => "Gloves",
        ItemSlot::Helmet => "Helmet",
        ItemSlot::Relic => "Relic",
        ItemSlot::Ring => "Ring",
        ItemSlot::Shield => "Shield",
        ItemSlot::Weapon => "Weapon",
    };
    let armor_info = item_specs
        .armor_specs
        .as_ref()
        .zip(item_specs.base.armor_specs.as_ref())
        .map(|(specs, base_specs)| {
            let armor_color = if specs.armor != base_specs.armor {
                "text-blue-400"
            } else {
                "text-white"
            };

            view! {
                <li class="text-gray-400 text-sm leading-snug">
                    "Armor: "
                    <span class=format!(
                        "{} font-semibold",
                        armor_color,
                    )>{format!("{:.0}", specs.armor)}</span>
                </li>
            }
        });

    let weapon_info = item_specs
        .weapon_specs
        .as_ref()
        .zip(item_specs.base.weapon_specs.as_ref())
        .map(|(specs, base_specs)| {
            let cooldown_color = if specs.cooldown != base_specs.cooldown {
                "text-blue-400"
            } else {
                "text-white"
            };

            let damage_color = if specs.min_damage != base_specs.min_damage
                || specs.max_damage != base_specs.max_damage
            {
                "text-blue-400"
            } else {
                "text-white"
            };

            view! {
                <li class="text-gray-400 text-sm leading-snug">"Weapon"</li>
                <li class="text-gray-400 text-sm leading-snug">
                    "Physical Damage: "
                    <span class=format!(
                        "{} font-semibold",
                        damage_color,
                    )>
                        {format!("{:.0}", specs.min_damage)} " - "
                        {format!("{:.0}", specs.max_damage)}
                    </span>
                </li>
                <li class="text-gray-400 text-sm leading-snug">
                    "Attacks per Second: "
                    <span class=format!(
                        "{} font-semibold",
                        cooldown_color,
                    )>{format!("{:.2}", specs.cooldown)}</span>
                </li>
            }
        });

    let affixes = formatted_affix_list(item_specs.aggregate_effects());

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
            )>{item_specs.base.name.clone()}</strong>
            <hr class="border-t border-gray-700" />
            <p class="text-sm text-gray-400 leading-snug">
                "Item Level: " <span class="text-white">{item_specs.level}</span>
            </p>
            <hr class="border-t border-gray-700" />
            <ul class="list-none space-y-1">{item_slot} {armor_info} {weapon_info}</ul>
            {(!affixes.is_empty()).then(|| view! { <hr class="border-t border-gray-700 my-1" /> })}
            <ul class="list-none space-y-1">{affixes}</ul>
            <hr class="border-t border-gray-700" />
            <p class="text-sm italic text-gray-300 leading-snug">
                {item_specs.base.description.clone()}
            </p>
        </div>
    }
}

fn magic_affix_li(text: String) -> impl IntoView {
    view! { <li class="text-blue-400 text-sm leading-snug">{text}</li> }
}

pub fn formatted_affix_list(mut affix_effects: Vec<AffixEffect>) -> Vec<impl IntoView> {
    use AffixEffectType::*;
    use ItemStat::*;

    affix_effects.sort_by_key(|effect| {
        (
            -match effect.stat {
                LocalAttackDamage => 0,
                LocalMinAttackDamage => 1,
                LocalMaxAttackDamage => 2,
                LocalAttackSpeed => 3,
                LocalArmor => 4,
                GoldFind => 5,
            },
            -match effect.effect_type {
                Flat => 0,
                Multiplier => 1,
            },
        )
    });

    let mut merged: Vec<String> = Vec::new();

    // This will be used to merge added min and added max damage together
    let mut min_flat: Option<f64> = None;
    let mut max_flat: Option<f64> = None;

    for effect in affix_effects {
        match (effect.stat, effect.effect_type) {
            (LocalMinAttackDamage, Flat) => min_flat = Some(effect.value),
            (LocalMaxAttackDamage, Flat) => max_flat = Some(effect.value),
            (LocalMinAttackDamage, Multiplier) => merged.push(format!(
                "{:.0}% Increased Minimum Attack Damage",
                effect.value * 100.0
            )),
            (LocalMaxAttackDamage, Multiplier) => merged.push(format!(
                "{:.0}% Increased Maximum Attack Damage",
                effect.value * 100.0
            )),
            // If it's not part of a min/max pair, process normally
            (LocalAttackSpeed, Flat) => merged.push(format!("-{:.2}s Attack Speed", effect.value)),
            (LocalAttackSpeed, Multiplier) => merged.push(format!(
                "{:.0}% Increased Attack Speed",
                effect.value * 100.0
            )),
            (LocalAttackDamage, Flat) => {
                merged.push(format!("{:.0} Added Attack Damage", effect.value))
            }
            (LocalAttackDamage, Multiplier) => merged.push(format!(
                "{:.0}% Increased Attack Damage",
                effect.value * 100.0
            )),
            (LocalArmor, Flat) => merged.push(format!("+{:.0} Added Armor", effect.value)),
            (LocalArmor, Multiplier) => {
                merged.push(format!("{:.0}% Increased Armor", effect.value))
            }
            (GoldFind, Flat) => {
                merged.push(format!("+{:.0} Gold per Kill", effect.value));
            }
            (GoldFind, Multiplier) => {
                merged.push(format!("{:.0}% Increased Gold Find", effect.value * 100.0))
            }
        }
    }

    match (min_flat, max_flat) {
        (Some(min_flat), Some(max_flat)) => merged.push(format!(
            "Adds {:.0} to {:.0} Attack Damage",
            min_flat, max_flat
        )),
        (Some(min_flat), None) => {
            merged.push(format!("Adds {:.0} to Minimum Attack Damage", min_flat))
        }
        (None, Some(max_flat)) => {
            merged.push(format!("Adds {:.0} to Maximum Attack Damage", max_flat))
        }
        _ => {}
    }

    merged.into_iter().rev().map(magic_affix_li).collect()
}
