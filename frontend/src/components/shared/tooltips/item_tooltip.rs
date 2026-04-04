use std::sync::Arc;
use strum::IntoEnumIterator;

use leptos::{html::*, prelude::*};

use shared::data::{
    area::AreaLevel,
    item::{ItemCategory, ItemRarity, ItemSlot, ItemSpecs, SkillRange, SkillShape},
    item_affix::{AffixEffectScope, AffixTag, AffixType, ItemAffix},
    skill::DamageType,
};

use crate::components::{
    data_context::DataContext,
    shared::tooltips::{effects_tooltip::scope_str, trigger_tooltip::format_trigger},
    ui::{Separator, number},
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
    #[prop(default = Signal::derive(|| AreaLevel::MAX))] max_item_level: Signal<AreaLevel>,
    // #[prop(default = Signal::derive(|| AreaLevel::MAX))] max_item_level: RwSignal<AreaLevel>,
) -> impl IntoView {
    let (border_color, inner_border, shadow_color, rarity_wash, rarity_core, frame_shine) =
        tooltip_rarity_palette(item_specs.modifiers.rarity);

    view! {
        <div class="relative isolate max-w-xs text-center">
            <div
                class="pointer-events-none absolute inset-0"
                aria-hidden="true"
                style=format!(
                    "filter: drop-shadow(0 10px 20px {}) drop-shadow(0 3px 5px rgba(0,0,0,0.45));",
                    shadow_color,
                )
            >
                <div
                    class="absolute inset-0 bg-black/90"
                    style="clip-path: polygon(10px 0, calc(100% - 10px) 0, 100% 10px, 100% calc(100% - 10px), calc(100% - 10px) 100%, 10px 100%, 0 calc(100% - 10px), 0 10px);"
                ></div>
            </div>

            <div
                class=format!(
                    "relative overflow-hidden border {} shadow-[inset_0_1px_0_rgba(240,215,159,0.18),inset_0_-1px_0_rgba(0,0,0,0.5)]",
                    border_color,
                )
                style=format!(
                    "clip-path: polygon(10px 0, calc(100% - 10px) 0, 100% 10px, 100% calc(100% - 10px), calc(100% - 10px) 100%, 10px 100%, 0 calc(100% - 10px), 0 10px);
                    background-image:
                        linear-gradient(180deg, rgba(214,177,102,0.06), rgba(0,0,0,0.22)),
                        radial-gradient(circle at 50% 22%, {}, transparent 62%),
                        linear-gradient(180deg, {}, transparent 36%),
                        linear-gradient(135deg, rgba(30,29,34,0.985), rgba(9,9,12,1));
                    background-blend-mode: screen, soft-light, soft-light, normal;",
                    rarity_core,
                    rarity_wash,
                )
            >
                <div
                    class=format!(
                        "pointer-events-none absolute inset-[1px] border {}",
                        inner_border,
                    )
                    style="clip-path: polygon(9px 0, calc(100% - 9px) 0, 100% 9px, 100% calc(100% - 9px), calc(100% - 9px) 100%, 9px 100%, 0 calc(100% - 9px), 0 9px);"
                ></div>
                <span
                    class="pointer-events-none absolute inset-x-[6px] top-[2px] h-[2px]"
                    style=format!(
                        "background: linear-gradient(90deg, transparent, {}, transparent);",
                        frame_shine,
                    )
                ></span>
                <span
                    class="pointer-events-none absolute inset-y-[5px] left-[1px] w-[2px]"
                    style=format!(
                        "background: linear-gradient(180deg, transparent, {}, transparent);",
                        frame_shine,
                    )
                ></span>
                <span
                    class="pointer-events-none absolute inset-y-[5px] right-[1px] w-[2px]"
                    style=format!(
                        "background: linear-gradient(180deg, transparent, {}, transparent);",
                        frame_shine,
                    )
                ></span>
                <span class="pointer-events-none absolute inset-x-4 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/45 to-transparent"></span>
                <div class="relative p-2 xl:p-4">
                    <ItemTooltipContent
                        item_specs=item_specs.clone()
                        show_affixes
                        comparable
                        max_item_level
                    />
                </div>
            </div>
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
        let upgrade_affixes =
            formatted_affixes_list(&item_specs.modifiers.affixes, AffixType::Upgrade);
        // let effects = formatted_affixes_list(&item_specs.modifiers.affixes);
        (
            (!base_affixes.is_empty() || !prefixes.is_empty() || !suffixes.is_empty()),
            view! {
                {(!base_affixes.is_empty())
                    .then(|| {
                        view! {
                            // <li class="text-gray-400 text-xs xl:text-sm ">
                            // "Base affixes:"
                            // </li>
                            {base_affixes}
                        }
                    })}
                {(!prefixes.is_empty())
                    .then(|| {
                        view! {
                            // <li class="text-gray-400 text-xs xl:text-sm ">
                            // "Prefixes:"
                            // </li>
                            {prefixes}
                        }
                    })}
                {(!suffixes.is_empty())
                    .then(|| {
                        view! {
                            // <li class="text-gray-400 text-xs xl:text-sm ">
                            // "Suffixes:"
                            // </li>
                            {suffixes}
                        }
                    })}
                {(!upgrade_affixes.is_empty())
                    .then(|| {
                        view! {
                            // <li class="text-gray-400 text-xs xl:text-sm ">
                            // "Base affixes:"
                            // </li>
                            {upgrade_affixes}
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
                                <li class="text-gray-400 text-xs ">"Triggers:"</li>
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

    let (has_upgrades, upgrades) = {
        if show_affixes && !item_specs.base.upgrade_effects.is_empty() {
            let upgrade_effects = item_specs
                .base
                .upgrade_effects
                .iter()
                .map(|effect| effect.stat_effect.clone())
                .collect::<Vec<_>>();

            (
                item_specs
                    .base
                    .upgrade_levels
                    .get(item_specs.modifiers.upgrade_level as usize)
                    .map(|next_level| *next_level <= item_specs.modifiers.level)
                    .unwrap_or_default(),
                Some(effects_tooltip::formatted_effects_list(upgrade_effects)),
            )
        } else {
            (false, None)
        }
    };

    let max_upgrade_level = item_specs
        .base
        .upgrade_levels
        .iter()
        .filter(|upgrade_level| **upgrade_level <= item_specs.modifiers.level)
        .count();

    let name_color = name_color_rarity(item_specs.modifiers.rarity);

    let required_level = item_specs.required_level;

    view! {
        <div class="space-y-2">
            {match comparable {
                ComparableType::Compared | ComparableType::Equipped => {
                    Some(
                        view! {
                            <p class="text-xs xl:text-sm italic text-gray-400  whitespace-pre-line">
                                {match comparable {
                                    ComparableType::Compared => "Selected",
                                    ComparableType::Equipped => "Equipped",
                                    _ => "",
                                }}
                            </p>
                            <Separator />
                        },
                    )
                }
                _ => None,
            }} <strong class=format!("text-sm xl:text-base font-bold font-display tracking-[0.03em] text-shadow-md/80 {}", name_color)>
                <ul class="list-none xl:space-y-1 mb-2">
                    <li class=" whitespace-pre-line">{item_specs.modifiers.name.clone()}</li>
                    {match item_specs.modifiers.rarity {
                        ItemRarity::Rare | ItemRarity::Masterwork => {
                            Some(view! { <li class="">{item_specs.base.name.clone()}</li> })
                        }
                        _ => None,
                    }}

                </ul>
            </strong> <Separator /> <ul class="list-none xl:space-y-1">
                <ItemSlotTooltip item_specs=item_specs.clone() show_level=show_affixes />
                {(max_upgrade_level > 0)
                    .then(|| {
                        view! {
                            <li class="text-xs xl:text-sm text-gray-400">
                                "Empower level: "
                                <span class="font-bold text-[#f0b36b]">
                                    {item_specs.modifiers.upgrade_level}
                                </span>"/"{max_upgrade_level}
                            </li>
                        }
                    })}
                <QualityTooltip item_specs=item_specs.clone() />
                <ArmorTooltip item_specs=item_specs.clone() />
                <WeaponTooltip item_specs=item_specs.clone() />
                <RuneTooltip item_specs=item_specs.clone() />
                <MapTooltip item_specs=item_specs.clone() />
            </ul>
            {(has_triggers || has_effects)
                .then(|| {
                    view! {
                        <Separator />
                        <ul class="list-none xl:space-y-1 text-xs xl:text-sm">
                            {effects}{triggers}
                        </ul>
                    }
                })}
            {(has_upgrades)
                .then(|| {
                    view! {
                        <Separator />
                        <span class="text-xs xl:text-sm text-gray-400">"Empower effects:"</span>
                        <ul class="list-none xl:space-y-1 text-xs xl:text-sm">{upgrades}</ul>
                    }
                })} <Separator /> <ul class="list-none xl:space-y-1">
                <li class="text-xs xl:text-sm text-gray-400">
                    "Required Power Level: "
                    <span class=move || {
                        if max_item_level.get() < required_level {
                            "font-bold text-red-500"
                        } else {
                            "text-stone-100"
                        }
                    }>{required_level}</span>
                </li>
                <li class="text-xs xl:text-sm text-gray-400">
                    "Item Level: " <span class="text-stone-100">{item_specs.modifiers.level}</span>
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
                                <Separator />
                                <p class="text-xs xl:text-sm italic text-gray-400  whitespace-pre-line">
                                    {description}
                                </p>
                            }
                        })
                })}
            {match comparable {
                ComparableType::Comparable => {
                    Some(
                        view! {
                            <Separator />
                            <p class="text-xs xl:text-sm italic text-gray-400  whitespace-pre-line">
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
        ItemRarity::Normal => "text-stone-100",
        ItemRarity::Magic => "text-blue-300",
        ItemRarity::Rare => "text-[#f0cd78]",
        ItemRarity::Masterwork => "text-fuchsia-300",
        ItemRarity::Unique => "text-[#ff9d67]",
    }
}

fn tooltip_rarity_palette(
    item_rarity: ItemRarity,
) -> (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
) {
    match item_rarity {
        ItemRarity::Normal => (
            "border-[#6c5329]/85",
            "border-white/8",
            "rgba(0,0,0,0.5)",
            "rgba(210,215,224,0.05)",
            "rgba(255,255,255,0.02)",
            "rgba(230,230,236,0.2)",
        ),
        ItemRarity::Magic => (
            "border-[#6c5329]/85",
            "border-blue-200/18",
            "rgba(23,44,89,0.52)",
            "rgba(75,126,235,0.14)",
            "rgba(48,86,196,0.1)",
            "rgba(144,205,255,0.44)",
        ),
        ItemRarity::Rare => (
            "border-[#7a5d29]/90",
            "border-[#f3db9d]/18",
            "rgba(67,47,11,0.55)",
            "rgba(173,124,26,0.16)",
            "rgba(108,76,8,0.12)",
            "rgba(255,226,145,0.5)",
        ),
        ItemRarity::Masterwork => (
            "border-[#74528a]/88",
            "border-fuchsia-200/18",
            "rgba(61,24,99,0.52)",
            "rgba(143,78,220,0.15)",
            "rgba(90,44,150,0.11)",
            "rgba(228,183,255,0.46)",
        ),
        ItemRarity::Unique => (
            "border-[#8c5628]/92",
            "border-[#ffd1af]/18",
            "rgba(104,34,8,0.62)",
            "rgba(188,72,28,0.2)",
            "rgba(114,18,8,0.16)",
            "rgba(255,170,116,0.54)",
        ),
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
                {if *specs.armor > 0.0 {
                    Some(
                        view! {
                            <li class="text-gray-400 text-xs xl:text-sm ">
                                "Armor: "
                                <span class=format!(
                                    "{} font-semibold",
                                    armor_color,
                                )>{format!("{:.0}", *specs.armor)}</span>
                            </li>
                        },
                    )
                } else {
                    None
                }}
                {if specs.block.get() > 0.0 {
                    Some(
                        view! {
                            <li class="text-gray-400 text-xs xl:text-sm ">
                                "Block chance: "
                                <span class=format!(
                                    "{} font-semibold",
                                    block_color,
                                )>{format!("{:.0}%", specs.block.get())}</span>
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
                        if specs_value.min.get().round() != base_value.min.get().round()
                            || specs_value.max.get().round() != base_value.max.get().round()
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

                if specs_value.min.get() > 0.0 || specs_value.max.get() > 0.0 {
                    damage_lines.push(view! {
                        <li class="text-gray-400 text-xs xl:text-sm ">
                            {effects_tooltip::damage_type_str(Some(damage_type))} "Damage: "
                            <span class=format!(
                                "{} font-semibold",
                                damage_color,
                            )>
                                {format!(
                                    "{:.0} - {:.0}",
                                    specs_value.min.get(),
                                    specs_value.max.get(),
                                )}
                            </span>
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
                <li class="text-gray-400 text-xs xl:text-sm ">{range} {shape}</li>
                {damage_lines}
                <li class="text-gray-400 text-xs xl:text-sm ">
                    "Critical hit chance: "
                    <span class=format!(
                        "{} font-semibold",
                        crit_chance_color,
                    )>{format!("{:.2}%", specs.crit_chance.value.get())}</span>
                </li>
                <li class="text-gray-400 text-xs xl:text-sm ">
                    "Critical hit damage: "
                    <span class=format!(
                        "{} font-semibold",
                        crit_damage_color,
                    )>{format!("+{}%", number::format_number(*specs.crit_damage))}</span>
                </li>
                <li class="text-gray-400 text-xs xl:text-sm ">
                    "Cooldown: "
                    <span class=format!(
                        "{} font-semibold",
                        cooldown_color,
                    )>{format!("{:.2}s", specs.cooldown.get())}</span>
                </li>
            }
        })
}

#[component]
pub fn RuneTooltip(item_specs: Arc<ItemSpecs>) -> impl IntoView {
    item_specs.base.rune_specs.as_ref().map(|specs| {
        view! {
            <li class="text-gray-400 text-xs xl:text-sm ">"Rune"</li>
            {(specs.root_node)
                .then(|| {
                    view! {
                        <li class="text-white text-xs xl:text-sm ">
                            "Transform Node into Root Node"
                        </li>
                    }
                })}
            <li class="text-gray-400 text-xs xl:text-sm  italic">
                "Socket into an empty Passive Node to give the following effects:"
            </li>
        }
    })
}

#[component]
pub fn MapTooltip(item_specs: Arc<ItemSpecs>) -> impl IntoView {
    let data_context: DataContext = expect_context();

    item_specs.base.map_specs.as_ref().map(|specs| {
        view! {
            <li class="text-gray-400 text-xs xl:text-sm ">"Edict"</li>

            {specs
                .area_id
                .as_ref()
                .map(|area_id| {
                    view! {
                        <li class="text-gray-400 text-xs xl:text-sm ">
                            "Only for: "
                            <span class="text-white font-semibold">
                                {data_context
                                    .areas_specs
                                    .read_untracked()
                                    .get(area_id)
                                    .map(|area| area.name.clone())
                                    .unwrap_or(area_id.clone())}
                            </span>
                        </li>
                    }
                })}

            {(specs.reward_slots > 0)
                .then(|| {
                    view! {
                        <li class="text-gray-400 text-xs xl:text-sm ">
                            "Rare Reward Slots: "
                            <span class="text-white font-semibold">
                                {format!("+{:.0}", specs.reward_slots)}
                            </span>
                        </li>
                    }
                })}

            {(specs.reward_picks > 0)
                .then(|| {
                    view! {
                        <li class="text-gray-400 text-xs xl:text-sm ">
                            "Reward Picks: "
                            <span class="text-white font-semibold">
                                {format!("+{:.0}", specs.reward_picks)}
                            </span>
                        </li>
                    }
                })}

            <li class="text-gray-400 text-xs xl:text-sm  italic">
                "Apply to a Grind to give all Enemies the following effects:"
            </li>
        }
    })
}

#[component]
pub fn ItemSlotTooltip(item_specs: Arc<ItemSpecs>, show_level: bool) -> impl IntoView {
    view! {
        {item_specs
            .base
            .slot
            .map(|slot| {
                let item_slot = match slot {
                    ItemSlot::Amulet => "Amulet",
                    ItemSlot::Body => "Body Armor",
                    ItemSlot::Boots => "Boots",
                    ItemSlot::Gloves => "Gloves",
                    ItemSlot::Helmet => "Helmet",
                    ItemSlot::Ring => "Ring",
                    ItemSlot::Shield => "Off-hand",
                    ItemSlot::Accessory => "Accessory",
                    ItemSlot::Weapon => {
                        if item_specs.base.extra_slots.contains(&ItemSlot::Shield) {
                            "Two-Handed Weapon"
                        } else {
                            "One-Handed Weapon"
                        }
                    }
                };

                view! {
                    <li class="text-gray-400 text-xs xl:text-sm ">
                        {item_slot}
                        {(show_level)
                            .then(|| format!(" - Level {}", item_specs.base.min_area_level))}
                    </li>
                }
            })}
    }
}

#[component]
pub fn QualityTooltip(item_specs: Arc<ItemSpecs>) -> impl IntoView {
    view! {
        {(item_specs.modifiers.quality > 0.0)
            .then(|| {
                view! {
                    <li class="text-gray-400 text-xs xl:text-sm ">
                        "Quality: "
                        <span class="text-white font-semibold">
                            {format!("+{:.0}%", item_specs.modifiers.quality)}
                        </span>
                    </li>
                }
            })}
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
                .first()
                .map(|e| e.scope)
                .unwrap_or(AffixEffectScope::Global);
            let affix_meta = match affix_type {
                AffixType::Unique | AffixType::Upgrade => view! {
                    <li class="text-gray-400 text-xs -mb-1">
                        {affix_type_str(affix.affix_type)}" – "{scope_str(scope)}
                    </li>
                }
                .into_any(),
                _ => {
                    let mut tags: Vec<_> = affix.tags.iter().collect();
                    tags.sort();
                    let tags: Vec<_> = tags
                        .into_iter()
                        .map(|tag| affix_tag_str(*tag).to_string())
                        .collect();

                    view! {
                        <li class="text-gray-400 text-xs -mb-1">
                            {affix_type_str(affix.affix_type)}" "
                            <span class="italic">"‘"{affix.name.clone()}"’"</span> "  (Level: "
                            {affix.item_level}") – " {scope_str(scope)} " – "{tags.join(", ")}
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
        AffixType::Upgrade => "Empowering affix",
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

pub fn item_category_str(item_category: ItemCategory) -> &'static str {
    match item_category {
        ItemCategory::Armor => "Armor Piece",
        ItemCategory::AttackWeapon => "Attack Weapon",
        ItemCategory::SpellWeapon => "Spell Weapon",
        ItemCategory::MeleeWeapon => "Melee Weapon",
        ItemCategory::MeleeWeapon1H => "One-Handed Melee Weapon",
        ItemCategory::MeleeWeapon2H => "Two-Handed Melee Weapon",
        ItemCategory::RangedWeapon => "Ranged Weapon",
        ItemCategory::Shield => "Shield",
        ItemCategory::Focus => "Magic Focus",
        ItemCategory::Jewelry => "Jewelry",
        ItemCategory::Accessory => "Accessory",
        ItemCategory::Amulet => "Amulet",
        ItemCategory::Body => "Body Armor",
        ItemCategory::Boots => "Boots",
        ItemCategory::Cloak => "Cloak",
        ItemCategory::Gloves => "Gloves",
        ItemCategory::Helmet => "Helmet",
        ItemCategory::Ring => "Ring",
        ItemCategory::Map => "Edict",
        ItemCategory::Rune => "Rune",
    }
}
