use std::sync::Arc;
use strum::IntoEnumIterator;

use leptos::{html::*, prelude::*};

use shared::data::{
    item::{ItemRarity, ItemSlot, ItemSpecs, SkillRange, SkillShape},
    item_affix::AffixEffectScope,
    skill::DamageType,
};

use crate::components::game::tooltips::skill_tooltip::format_trigger;

use super::effects_tooltip;

#[component]
pub fn ItemTooltip(item_specs: Arc<ItemSpecs>) -> impl IntoView {
    let (border_color, ring_color, shadow_color) = match item_specs.modifiers.rarity {
        ItemRarity::Normal => ("border-gray-600", "ring-gray-700", "shadow-gray-800"),
        ItemRarity::Magic => ("border-blue-500", "ring-blue-400", "shadow-blue-700"),
        ItemRarity::Rare => ("border-yellow-400", "ring-yellow-300", "shadow-yellow-600"),
        ItemRarity::Unique => ("border-amber-700", "ring-amber-600", "shadow-amber-700"),
    };

    view! {
        <div class=format!(
            "max-w-xs p-2 lg:p-4 rounded-xl border {} ring-2 {} shadow-md {} bg-gradient-to-br from-gray-800 via-gray-900 to-black",
            border_color,
            ring_color,
            shadow_color,
        )>
            <ItemTooltipContent item_specs />
        </div>
    }
}

#[component]
pub fn ItemTooltipContent(
    item_specs: Arc<ItemSpecs>,
    #[prop(default = false)] hide_description: bool,
) -> impl IntoView {
    let local_affixes = effects_tooltip::formatted_effects_list(
        (&item_specs
            .modifiers
            .aggregate_effects(AffixEffectScope::Local))
            .into(),
        AffixEffectScope::Local,
    );
    let global_affixes = effects_tooltip::formatted_effects_list(
        (&item_specs
            .modifiers
            .aggregate_effects(AffixEffectScope::Global))
            .into(),
        AffixEffectScope::Global,
    );

    let trigger_lines = item_specs
        .base
        .triggers
        .clone()
        .into_iter()
        .map(format_trigger)
        .collect::<Vec<_>>();

    let name_color = name_color_rarity(item_specs.modifiers.rarity);

    view! {
        <div class="space-y-2">
            <strong class=format!("text-base lg:text-lg font-bold {}", name_color)>
                <ul class="list-none space-y-1 md-2">
                    <li class="leading-snug whitespace-pre-line">
                        {item_specs.modifiers.name.clone()}
                    </li>
                    {match item_specs.modifiers.rarity {
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
                <ItemSlotTooltip item_specs=item_specs.clone() />
                <ArmorTooltip item_specs=item_specs.clone() />
                <WeaponTooltip item_specs=item_specs.clone() />
            </ul>
            {(!trigger_lines.is_empty() || !local_affixes.is_empty() || !global_affixes.is_empty())
                .then(|| {
                    view! {
                        <hr class="border-t border-gray-700 my-1" />
                        <ul class="list-none space-y-1">
                            {local_affixes}{global_affixes}{trigger_lines}
                        </ul>
                    }
                })}
            <hr class="border-t border-gray-700" />
            <p class="text-xs lg:text-sm text-gray-400 leading-snug">
                "Item Level: " <span class="text-white">{item_specs.modifiers.level}</span>
            </p>
            {(!hide_description)
                .then(|| {
                    item_specs
                        .base
                        .description
                        .clone()
                        .map(|description| {
                            view! {
                                <hr class="border-t border-gray-700" />
                                <p class="text-xs lg:text-sm italic text-gray-400 leading-snug whitespace-pre-line">
                                    {description}
                                </p>
                            }
                        })
                })}
        </div>
    }
}

pub fn name_color_rarity(item_rarity: ItemRarity) -> &'static str {
    match item_rarity {
        ItemRarity::Normal => "text-white",
        ItemRarity::Magic => "text-blue-400",
        ItemRarity::Rare => "text-yellow-400",
        ItemRarity::Unique => "text-amber-700",
    }
}

#[component]
pub fn ArmorTooltip(item_specs: Arc<ItemSpecs>) -> impl IntoView {
    item_specs
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
                            <li class="text-gray-400 text-xs lg:text-sm leading-snug">
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
                            <li class="text-gray-400 text-xs lg:text-sm leading-snug">
                                "Block chances: "
                                <span class=format!(
                                    "{} font-semibold",
                                    block_color,
                                )>{format!("{:.0}%", specs.block)}</span>
                            </li>
                        },
                    )
                } else {
                    None
                }}
            }
        })
}

#[component]
pub fn WeaponTooltip(item_specs: Arc<ItemSpecs>) -> impl IntoView {
    item_specs
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
                    DamageType::Storm => "text-amber-400",
                };

                if spec_min > 0.0 || spec_max > 0.0 {
                    damage_lines.push(view! {
                        <li class="text-gray-400 text-xs lg:text-sm leading-snug">
                            {effects_tooltip::optional_damage_type_str(Some(damage_type))}
                            "Damage: "
                            <span class=format!(
                                "{} font-semibold",
                                damage_color,
                            )>{format!("{spec_min:.0} - {spec_max:.0}")}</span>
                        </li>
                    });
                }
            }

            let shape = match specs.shape {
                SkillShape::Single => "",
                SkillShape::Vertical2 => ", 2x1 area",
                SkillShape::Horizontal2 => ", 1x2 area",
                SkillShape::Horizontal3 => ", 1x3 area",
                SkillShape::Square4 => ", 2x2 area",
                SkillShape::All => ", all",
            };

            let range = match specs.range {
                SkillRange::Melee => "Melee",
                SkillRange::Distance => "Distance",
                SkillRange::Any => "Any",
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
                <li class="text-gray-400 text-xs lg:text-sm leading-snug">{range} {shape}</li>
                {damage_lines}
                <li class="text-gray-400 text-xs lg:text-sm leading-snug">
                    "Critical chances: "
                    <span class=format!(
                        "{} font-semibold",
                        crit_chances_color,
                    )>{format!("{:.2}%", specs.crit_chances)}</span>
                </li>
                <li class="text-gray-400 text-xs lg:text-sm leading-snug">
                    "Critical damage: "
                    <span class=format!(
                        "{} font-semibold",
                        crit_damage_color,
                    )>{format!("+{:.0}%", specs.crit_damage)}</span>
                </li>
                <li class="text-gray-400 text-xs lg:text-sm leading-snug">
                    "Cooldown: "
                    <span class=format!(
                        "{} font-semibold",
                        cooldown_color,
                    )>{format!("{:.2}s", specs.cooldown)}</span>
                </li>
            }
        })
}

#[component]
pub fn ItemSlotTooltip(item_specs: Arc<ItemSpecs>) -> impl IntoView {
    let item_slot = match &item_specs.base.slot {
        ItemSlot::Amulet => "Amulet",
        ItemSlot::Body => "Body Armor",
        ItemSlot::Boots => "Boots",
        ItemSlot::Gloves => "Gloves",
        ItemSlot::Helmet => "Helmet",
        ItemSlot::Ring => "Ring",
        ItemSlot::Shield => "Shield",
        ItemSlot::Accessory => "Accessory",
        ItemSlot::Weapon => {
            if item_specs.base.extra_slots.contains(&ItemSlot::Shield) {
                "Two-Handed Weapon"
            } else {
                "One-Handed Weapon"
            }
        }
    };

    view! { <li class="text-gray-400 text-xs lg:text-sm leading-snug">{item_slot}</li> }
}
