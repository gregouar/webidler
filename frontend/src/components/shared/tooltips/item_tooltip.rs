use std::sync::Arc;
use strum::IntoEnumIterator;

use leptos::{html::*, prelude::*};

use shared::data::{
    area::AreaLevel,
    item::{ItemRarity, ItemSlot, ItemSpecs, SkillRange, SkillShape},
    item_affix::{AffixEffectScope, AffixTag, AffixType, ItemAffix},
    skill::DamageType,
};

use crate::components::shared::tooltips::{
    effects_tooltip::scope_str, trigger_tooltip::format_trigger,
};

use super::effects_tooltip;

pub enum ComparableType {
    NotComparable,
    Comparable,
    Compared,
    Equipped,
}

#[component]
pub fn ItemTooltip(
    item_specs: Arc<ItemSpecs>,
    #[prop(default = false)] show_affixes: bool,
    #[prop(default = ComparableType::NotComparable)] comparable: ComparableType,
    max_item_level: Signal<AreaLevel>,
    // #[prop(default = Signal::derive(|| AreaLevel::MAX))] max_item_level: RwSignal<AreaLevel>,
) -> impl IntoView {
    let (border_color, ring_color, shadow_color) = match item_specs.modifiers.rarity {
        ItemRarity::Normal => ("border-gray-600", "ring-gray-700", "shadow-gray-800"),
        ItemRarity::Magic => ("border-blue-500", "ring-blue-400", "shadow-blue-700"),
        ItemRarity::Rare => ("border-yellow-400", "ring-yellow-300", "shadow-yellow-600"),
        ItemRarity::Masterwork => (
            "border-fuchsia-400",
            "ring-fuchsia-300",
            "shadow-fuchsia-600",
        ),
        ItemRarity::Unique => ("border-orange-700", "ring-orange-600", "shadow-orange-700"),
    };

    view! {
        <div class=format!(
            "max-w-xs p-2 xl:p-4 rounded-xl border {} ring-2 {} shadow-md {} bg-gradient-to-br from-gray-800 via-gray-900 to-black",
            border_color,
            ring_color,
            shadow_color,
        )>
            <ItemTooltipContent
                item_specs=item_specs.clone()
                show_affixes
                comparable
                max_item_level
            />
        </div>
    }
}

#[component]
pub fn ItemTooltipContent(
    item_specs: Arc<ItemSpecs>,
    #[prop(default = false)] show_affixes: bool,
    #[prop(default = false)] hide_description: bool,
    #[prop(default = ComparableType::NotComparable)] comparable: ComparableType,
    max_item_level: Signal<AreaLevel>,
    // #[prop(default = Signal::derive(|| AreaLevel::MAX))] max_item_level: RwSignal<AreaLevel>,
) -> impl IntoView {
    let (has_effects, effects) = if show_affixes {
        let base_affixes = formatted_affixes_list(&item_specs.modifiers.affixes, AffixType::Unique);
        let prefixes = formatted_affixes_list(&item_specs.modifiers.affixes, AffixType::Prefix);
        let suffixes = formatted_affixes_list(&item_specs.modifiers.affixes, AffixType::Suffix);
        // let effects = formatted_affixes_list(&item_specs.modifiers.affixes);
        (
            (!base_affixes.is_empty() || !prefixes.is_empty() || !suffixes.is_empty()),
            view! {
                {(!base_affixes.is_empty())
                    .then(|| {
                        view! {
                            // <li class="text-gray-400 text-xs xl:text-sm leading-snug">
                            // "Base affixes:"
                            // </li>
                            {base_affixes}
                        }
                    })}
                {(!prefixes.is_empty())
                    .then(|| {
                        view! {
                            // <li class="text-gray-400 text-xs xl:text-sm leading-snug">
                            // "Prefixes:"
                            // </li>
                            {prefixes}
                        }
                    })}
                {(!suffixes.is_empty())
                    .then(|| {
                        view! {
                            // <li class="text-gray-400 text-xs xl:text-sm leading-snug">
                            // "Suffixes:"
                            // </li>
                            {suffixes}
                        }
                    })}
            }
            .into_any(),
        )
    } else {
        let mut effects = effects_tooltip::formatted_effects_list(
            (&item_specs
                .modifiers
                .aggregate_effects(AffixEffectScope::Local))
                .into(),
        );
        effects.extend(effects_tooltip::formatted_effects_list(
            (&item_specs
                .modifiers
                .aggregate_effects(AffixEffectScope::Global))
                .into(),
        ));
        (!effects.is_empty(), view! { {effects} }.into_any())
    };

    let (has_triggers, triggers) = {
        let trigger_lines = item_specs
            .base
            .triggers
            .clone()
            .into_iter()
            .map(format_trigger)
            .collect::<Vec<_>>();

        (
            !trigger_lines.is_empty(),
            if show_affixes {
                view! {
                    {(!trigger_lines.is_empty())
                        .then(|| {
                            view! {
                                <li class="text-gray-400 text-xs leading-snug">"Triggers:"</li>
                                {trigger_lines}
                            }
                        })}
                }
                .into_any()
            } else {
                view! { {trigger_lines} }.into_any()
            },
        )
    };

    let name_color = name_color_rarity(item_specs.modifiers.rarity);

    let required_level = item_specs.required_level;

    view! {
        <div class="space-y-2">
            {match comparable {
                ComparableType::Compared | ComparableType::Equipped => {
                    Some(
                        view! {
                            <p class="text-xs xl:text-sm italic text-gray-400 leading-snug whitespace-pre-line">
                                {match comparable {
                                    ComparableType::Compared => "Selected",
                                    ComparableType::Equipped => "Equipped",
                                    _ => "",
                                }}
                            </p>
                            <hr class="border-t border-gray-700" />
                        },
                    )
                }
                _ => None,
            }} <strong class=format!("text-base xl:text-lg font-bold {}", name_color)>
                <ul class="list-none space-y-1 mb-2">
                    <li class="leading-snug whitespace-pre-line">
                        {item_specs.modifiers.name.clone()}
                    </li>
                    {match item_specs.modifiers.rarity {
                        ItemRarity::Rare | ItemRarity::Masterwork => {
                            Some(
                                view! {
                                    <li class="leading-snug">{item_specs.base.name.clone()}</li>
                                },
                            )
                        }
                        _ => None,
                    }}

                </ul>
            </strong> <hr class="border-t border-gray-700" /> <ul class="list-none space-y-1">
                <ItemSlotTooltip item_specs=item_specs.clone() />
                <QualityTooltip item_specs=item_specs.clone() />
                <ArmorTooltip item_specs=item_specs.clone() />
                <WeaponTooltip item_specs=item_specs.clone() />
            </ul>
            {(has_triggers || has_effects)
                .then(|| {
                    view! {
                        <hr class="border-t border-gray-700 my-1" />
                        <ul class="list-none space-y-1 text-xs xl:text-sm">{effects}{triggers}</ul>
                    }
                })} <hr class="border-t border-gray-700" /> <ul class="list-none space-y-1">
                <li class="text-blue-400 text-xs xl:text-sm text-gray-400 leading-snug">
                    "Required Power Level: "
                    <span class=move || {
                        if max_item_level.get() < required_level {
                            "font-bold text-red-500"
                        } else {
                            "text-white"
                        }
                    }>{required_level}</span>
                </li>
                <li class="text-blue-400 text-xs xl:text-sm text-gray-400 leading-snug">
                    "Item Level: " <span class="text-white">{item_specs.modifiers.level}</span>
                </li>
            </ul>
            {(!hide_description)
                .then(|| {
                    item_specs
                        .base
                        .description
                        .clone()
                        .map(|description| {
                            view! {
                                <hr class="border-t border-gray-700" />
                                <p class="text-xs xl:text-sm italic text-gray-400 leading-snug whitespace-pre-line">
                                    {description}
                                </p>
                            }
                        })
                })}
            {match comparable {
                ComparableType::Comparable => {
                    Some(
                        view! {
                            <hr class="border-t border-gray-700" />
                            <p class="text-xs xl:text-sm italic text-gray-400 leading-snug whitespace-pre-line">
                                "Hold CTRL to compare."
                            </p>
                        },
                    )
                }
                _ => None,
            }}
        </div>
    }
}

pub fn name_color_rarity(item_rarity: ItemRarity) -> &'static str {
    match item_rarity {
        ItemRarity::Normal => "text-white",
        ItemRarity::Magic => "text-blue-400",
        ItemRarity::Rare => "text-yellow-400",
        ItemRarity::Masterwork => "text-fuchsia-400",
        ItemRarity::Unique => "text-orange-700",
    }
}

#[component]
pub fn ArmorTooltip(item_specs: Arc<ItemSpecs>) -> impl IntoView {
    item_specs
        .armor_specs
        .as_ref()
        .zip(item_specs.base.armor_specs.as_ref())
        .map(|(specs, base_specs)| {
            let armor_color = if specs.armor.round() != base_specs.armor.round() {
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
                            <li class="text-gray-400 text-xs xl:text-sm leading-snug">
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
                            <li class="text-gray-400 text-xs xl:text-sm leading-snug">
                                "Block chance: "
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
                let specs_value = specs.damage.get(&damage_type).copied().unwrap_or_default();
                let base_value = base_specs
                    .damage
                    .get(&damage_type)
                    .copied()
                    .unwrap_or_default();

                let damage_color = match damage_type {
                    DamageType::Physical => {
                        if specs_value.min.round() != base_value.min.round()
                            || specs_value.max.round() != base_value.max.round()
                        {
                            "text-blue-400"
                        } else {
                            "text-white"
                        }
                    }
                    DamageType::Fire => "text-red-400",
                    DamageType::Poison => "text-lime-400",
                    DamageType::Storm => "text-amber-400",
                };

                if specs_value.min > 0.0 || specs_value.max > 0.0 {
                    damage_lines.push(view! {
                        <li class="text-gray-400 text-xs xl:text-sm leading-snug">
                            {effects_tooltip::damage_type_str(Some(damage_type))} "Damage: "
                            <span class=format!(
                                "{} font-semibold",
                                damage_color,
                            )>{format!("{:.0} - {:.0}", specs_value.min, specs_value.max)}</span>
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
                SkillShape::Contact => ", contact",
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

            let crit_chance_color = if specs.crit_chance != base_specs.crit_chance {
                "text-blue-400"
            } else {
                "text-white"
            };

            let crit_damage_color = if specs.crit_damage.round() != base_specs.crit_damage.round() {
                "text-blue-400"
            } else {
                "text-white"
            };

            view! {
                <li class="text-gray-400 text-xs xl:text-sm leading-snug">{range} {shape}</li>
                {damage_lines}
                <li class="text-gray-400 text-xs xl:text-sm leading-snug">
                    "Critical hit chance: "
                    <span class=format!(
                        "{} font-semibold",
                        crit_chance_color,
                    )>{format!("{:.2}%", specs.crit_chance.value)}</span>
                </li>
                <li class="text-gray-400 text-xs xl:text-sm leading-snug">
                    "Critical hit damage: "
                    <span class=format!(
                        "{} font-semibold",
                        crit_damage_color,
                    )>{format!("+{:.0}%", specs.crit_damage)}</span>
                </li>
                <li class="text-gray-400 text-xs xl:text-sm leading-snug">
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

    view! { <li class="text-gray-400 text-xs xl:text-sm leading-snug">{item_slot}</li> }
}

#[component]
pub fn QualityTooltip(item_specs: Arc<ItemSpecs>) -> impl IntoView {
    view! {
        <li class="text-gray-400 text-xs xl:text-sm leading-snug">
            "Quality: "
            <span class="text-white font-semibold">
                {format!("+{:.0}%", item_specs.modifiers.quality)}
            </span>
        </li>
    }
}

pub fn formatted_affixes_list(
    affixes: &[ItemAffix],
    affix_type: AffixType,
) -> Vec<impl IntoView + use<>> {
    affixes
        .iter()
        .filter(|affix| affix.affix_type == affix_type)
        .map(|affix| {
            let scope = affix
                .effects
                .iter()
                .next()
                .map(|e| e.scope)
                .unwrap_or(AffixEffectScope::Global);
            let affix_meta = match affix_type {
                AffixType::Unique => {
                    view! { <li class="text-gray-400 text-xs leading-snug">"Implicit affix"</li> }
                        .into_any()
                }
                _ => {
                    let mut tags: Vec<_> = affix.tags.iter().collect();
                    tags.sort();
                    let tags: Vec<_> = tags
                        .into_iter()
                        .map(|tag| affix_tag_str(*tag).to_string())
                        .collect();

                    view! {
                        <li class="text-gray-400 text-xs leading-snug">
                            {affix_type_str(affix.affix_type)}" "
                            <span class="italic">"‘"{affix.name.clone()}"’"</span> "  (Tier: "
                            {affix.tier}") – " {scope_str(scope)} " – "{tags.join(", ")}
                        </li>
                    }
                    .into_any()
                }
            };
            view! {
                {affix_meta}
                {effects_tooltip::formatted_effects_list(
                    affix
                        .effects
                        .iter()
                        .map(|affix_effect| affix_effect.stat_effect.clone())
                        .collect(),
                )}
            }
        })
        .collect()
}

fn affix_type_str(affix_type: AffixType) -> &'static str {
    match affix_type {
        AffixType::Prefix => "Prefix",
        AffixType::Suffix => "Suffix",
        AffixType::Unique => "Base affix",
    }
}

fn affix_tag_str(affix_tag: AffixTag) -> &'static str {
    match affix_tag {
        AffixTag::Attack => "Attack",
        AffixTag::Armor => "Defense",
        AffixTag::Critical => "Critical",
        AffixTag::Fire => "Fire",
        AffixTag::Gold => "Gold",
        AffixTag::Life => "Life",
        AffixTag::Mana => "Mana",
        AffixTag::Physical => "Physical",
        AffixTag::Poison => "Poison",
        AffixTag::Skill => "Skill",
        AffixTag::Speed => "Speed",
        AffixTag::Spell => "Spell",
        AffixTag::Status => "Status",
        AffixTag::Stealth => "Stealth",
        AffixTag::Storm => "Storm",
        AffixTag::Threat => "Threat",
    }
}
