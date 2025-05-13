use std::sync::Arc;

use leptos::html::*;
use leptos::prelude::*;

use shared::data::{
    item::{ItemRarity, ItemSlot, ItemSpecs},
    item_affix::{AffixEffect, AffixEffectModifier, ItemStat},
};

#[component]
pub fn ItemTooltip(item_specs: Arc<ItemSpecs>) -> impl IntoView {
    let item_slot = match &item_specs.base.slot {
        ItemSlot::Amulet => "Amulet",
        ItemSlot::Body => "Body Armor",
        ItemSlot::Boots => "Boots",
        ItemSlot::Gloves => "Gloves",
        ItemSlot::Helmet => "Helmet",
        ItemSlot::Ring => "Ring",
        ItemSlot::Shield => "Shield",
        ItemSlot::Trinket => "Trinket",
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
            let damage_color = if specs.min_damage != base_specs.min_damage
                || specs.max_damage != base_specs.max_damage
            {
                "text-blue-400"
            } else {
                "text-white"
            };

            let cooldown_color = if specs.cooldown != base_specs.cooldown {
                "text-blue-400"
            } else {
                "text-white"
            };

            let crit_chances_color = if specs.crit_chances != base_specs.crit_chances {
                "text-blue-400"
            } else {
                "text-white"
            };

            let crit_damage_color = if specs.crit_damage != base_specs.crit_damage {
                "text-blue-400"
            } else {
                "text-white"
            };

            view! {
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
                    "Cooldown: "
                    <span class=format!(
                        "{} font-semibold",
                        cooldown_color,
                    )>{format!("{:.2}s", specs.cooldown)}</span>
                </li>
                <li class="text-gray-400 text-sm leading-snug">
                    "Critical chances: "
                    <span class=format!(
                        "{} font-semibold",
                        crit_chances_color,
                    )>{format!("{:.2}%", specs.crit_chances * 100.0)}</span>
                </li>
                <li class="text-gray-400 text-sm leading-snug">
                    "Critical damage: "
                    <span class=format!(
                        "{} font-semibold",
                        crit_damage_color,
                    )>{format!("+{:.0}%", specs.crit_damage * 100.0)}</span>
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
            <strong class=format!("text-lg font-bold {}", name_color)>
                <ul class="list-none space-y-1">
                    <li class="leading-snug">{item_specs.name.clone()}</li>
                    {match item_specs.rarity {
                        ItemRarity::Rare => {
                            Some(
                                view! {
                                    <li class="leading-snug">{item_specs.base.name.clone()}</li>
                                },
                            )
                        }
                        _ => None,
                    }}

                </ul>
            </strong>
            <hr class="border-t border-gray-700" />
            <ul class="list-none space-y-1">
                <li class="text-gray-400 text-sm leading-snug">{item_slot}</li>
                {armor_info}
                {weapon_info}
            </ul>
            {(!affixes.is_empty())
                .then(|| {
                    view! {
                        <hr class="border-t border-gray-700 my-1" />
                        <ul class="list-none space-y-1">{affixes}</ul>
                    }
                })}
            <hr class="border-t border-gray-700" />
            <p class="text-sm text-gray-400 leading-snug">
                "Item Level: " <span class="text-white">{item_specs.level}</span>
            </p>
            {item_specs
                .base
                .description
                .clone()
                .map(|description| {
                    view! {
                        <hr class="border-t border-gray-700" />
                        <p class="text-sm italic text-gray-300 leading-snug whitespace-pre-line">
                            {description}
                        </p>
                    }
                })}
        </div>
    }
}

fn magic_affix_li(text: String) -> impl IntoView {
    view! { <li class="text-blue-400 text-sm leading-snug">{text}</li> }
}

pub fn formatted_affix_list(mut affix_effects: Vec<AffixEffect>) -> Vec<impl IntoView> {
    use AffixEffectModifier::*;
    use ItemStat::*;

    affix_effects.sort_by_key(|effect| {
        (
            -match effect.stat {
                LocalAttackDamage => 0,
                LocalMinAttackDamage => 1,
                LocalMaxAttackDamage => 2,
                LocalAttackSpeed => 3,
                LocalArmor => 4,
                GlobalGoldFind => 5,
            },
            -match effect.modifier {
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
        match (effect.stat, effect.modifier) {
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
                merged.push(format!("{:.0}% Increased Armor", effect.value * 100.0))
            }
            (GlobalGoldFind, Flat) => {
                merged.push(format!("Adds {:.0} Gold per Kill", effect.value));
            }
            (GlobalGoldFind, Multiplier) => {
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
