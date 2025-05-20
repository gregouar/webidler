use std::sync::Arc;

use leptos::html::*;
use leptos::prelude::*;

use shared::data::item::Range;
use shared::data::item::Shape;
use shared::data::item::{ItemRarity, ItemSlot, ItemSpecs};
use shared::data::skill::DamageType;

use crate::components::game::effects_tooltip;

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
        ItemSlot::Weapon => {
            if item_specs.base.extra_slots.contains(&ItemSlot::Shield) {
                "Two-Handed Weapon"
            } else {
                "One-Handed Weapon"
            }
        }
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

            let block_color = if specs.block != base_specs.block {
                "text-blue-400"
            } else {
                "text-white"
            };

            view! {
                {if specs.armor > 0.0 {
                    Some(
                        view! {
                            <li class="text-gray-400 text-sm leading-snug">
                                "Armor: "
                                <span class=format!(
                                    "{} font-semibold",
                                    armor_color,
                                )>{format!("{:.0}", specs.armor)}</span>
                            </li>
                        },
                    )
                } else {
                    None
                }}
                {if specs.block > 0.0 {
                    Some(
                        view! {
                            <li class="text-gray-400 text-sm leading-snug">
                                "Block chances: "
                                <span class=format!(
                                    "{} font-semibold",
                                    block_color,
                                )>{format!("{:.0}%", specs.block * 100.0)}</span>
                            </li>
                        },
                    )
                } else {
                    None
                }}
            }
        });

    let weapon_info = item_specs
        .weapon_specs
        .as_ref()
        .zip(item_specs.base.weapon_specs.as_ref())
        .map(|(specs, base_specs)| {
            let mut damage_lines = vec![];

            for damage_type in DamageType::iter() {
                let (spec_min, spec_max) =
                    specs.damage.get(&damage_type).copied().unwrap_or_default();
                let (base_min, base_max) = base_specs
                    .damage
                    .get(&damage_type)
                    .copied()
                    .unwrap_or_default();

                let damage_color = match damage_type {
                    DamageType::Physical => {
                        if spec_min != base_min || spec_max != base_max {
                            "text-blue-400"
                        } else {
                            "text-white"
                        }
                    }
                    DamageType::Fire => "text-red-400",
                    DamageType::Poison => "text-lime-400",
                };

                if spec_min > 0.0 || spec_max > 0.0 {
                    damage_lines.push(view! {
                        <li class="text-gray-400 text-sm leading-snug">
                            {effects_tooltip::damage_type_str(damage_type)} " Damage: "
                            <span class=format!(
                                "{} font-semibold",
                                damage_color,
                            )>{format!("{:.0} - {:.0}", spec_min, spec_max)}</span>
                        </li>
                    });
                }
            }

            let shape = match specs.shape {
                Shape::Single => "",
                Shape::Vertical2 => ", 2x1 area",
                Shape::Horizontal2 => ", 1x2 area",
                Shape::Horizontal3 => ", 1x3 area",
                Shape::Square4 => ", 2x2 area",
                Shape::All => ", all",
            };

            let range = match specs.range {
                Range::Melee => "Melee",
                Range::Distance => "Distance",
                Range::Any => "Any",
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
                <li class="text-gray-400 text-sm leading-snug">{range} {shape}</li>
                {damage_lines}
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
                <li class="text-gray-400 text-sm leading-snug">
                    "Cooldown: "
                    <span class=format!(
                        "{} font-semibold",
                        cooldown_color,
                    )>{format!("{:.2}s", specs.cooldown)}</span>
                </li>
            }
        });

    let affixes = effects_tooltip::formatted_effects_list((&item_specs.aggregate_effects()).into());

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
                    <li class="leading-snug whitespace-pre-line">{item_specs.name.clone()}</li>
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
