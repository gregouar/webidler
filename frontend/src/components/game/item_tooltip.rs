use std::collections::HashMap;
use std::sync::Arc;

use leptos::html::*;
use leptos::prelude::*;

use shared::data::skill::DamageType;
use shared::data::{
    item::{ItemRarity, ItemSlot, ItemSpecs},
    item_affix::{AffixEffect, EffectModifier, EffectStat},
};

pub fn damage_type_str(damage_type: DamageType) -> &'static str {
    match damage_type {
        DamageType::Physical => "Physical",
        DamageType::Fire => "Fire",
    }
}

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
            let mut damage_lines = vec![];

            for damage_type in DamageType::iter() {
                let (spec_min, spec_max) =
                    specs.damage.get(&damage_type).copied().unwrap_or_default();
                let (base_min, base_max) = base_specs
                    .damage
                    .get(&damage_type)
                    .copied()
                    .unwrap_or_default();

                let damage_color = if spec_min != base_min || spec_max != base_max {
                    "text-blue-400"
                } else {
                    match damage_type {
                        DamageType::Physical => "text-white",
                        DamageType::Fire => "text-red-400",
                    }
                };

                if spec_min > 0.0 || spec_max > 0.0 {
                    damage_lines.push(view! {
                        <li class="text-gray-400 text-sm leading-snug">
                            {damage_type_str(damage_type)} " Damage: "
                            <span class=format!(
                                "{} font-semibold",
                                damage_color,
                            )>{format!("{:.0} - {:.0}", spec_min, spec_max)}</span>
                        </li>
                    });
                }
            }

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
                {damage_lines}
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
    use EffectModifier::*;
    use EffectStat::*;

    affix_effects.sort_by_key(|effect| {
        (
            // TODO: Macro? crate?
            -match effect.stat {
                LocalAttackDamage => 0,
                LocalMinDamage(DamageType::Physical) => 10,
                LocalMaxDamage(DamageType::Physical) => 20,
                LocalMinDamage(DamageType::Fire) => 30,
                LocalMaxDamage(DamageType::Fire) => 40,
                LocalCritChances => 50,
                LocalCritDamage => 60,
                LocalAttackSpeed => 70,
                LocalArmor => 80,
                GlobalLife => 90,
                GlobalLifeRegen => 100,
                GlobalMana => 110,
                GlobalManaRegen => 120,
                GlobalArmor => 130,
                GlobalAttackDamage => 140,
                GlobalDamage(DamageType::Physical) => 150,
                GlobalDamage(DamageType::Fire) => 160,
                GlobalSpellPower => 165,
                GlobalSpellDamage => 170,
                GlobalCritChances => 180,
                GlobalCritDamage => 190,
                GlobalAttackSpeed => 200,
                GlobalSpellSpeed => 210,
                GlobalSpeed => 220,
                GlobalMovementSpeed => 230,
                GlobalGoldFind => 240,
            },
            -match effect.modifier {
                Flat => 0,
                Multiplier => 1,
            },
        )
    });

    let mut merged: Vec<String> = Vec::new();

    // This will be used to merge added min and added max damage together
    let mut min_damage: HashMap<DamageType, f64> = HashMap::new();
    let mut max_damage: HashMap<DamageType, f64> = HashMap::new();

    for effect in affix_effects {
        match (effect.stat, effect.modifier) {
            (LocalMinDamage(damage_type), Flat) => {
                min_damage.insert(damage_type, effect.value);
            }
            (LocalMaxDamage(damage_type), Flat) => {
                max_damage.insert(damage_type, effect.value);
            }
            (LocalMinDamage(damage_type), Multiplier) => merged.push(format!(
                "{:.0}% Increased Minimum {} Damage",
                effect.value * 100.0,
                damage_type_str(damage_type)
            )),
            (LocalMaxDamage(damage_type), Multiplier) => merged.push(format!(
                "{:.0}% Increased Maximum {} Damage",
                effect.value * 100.0,
                damage_type_str(damage_type)
            )),
            (LocalAttackSpeed, Flat) => {
                merged.push(format!("-{:.2}s Attack Cooldown", effect.value))
            }
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
            (LocalCritChances, Flat) => merged.push(format!(
                "Adds {:.2}% Critical Strike Chances",
                effect.value * 100.0
            )),
            (LocalCritChances, Multiplier) => merged.push(format!(
                "{:.0}% Increased Critical Strike Chances",
                effect.value * 100.0
            )),
            (LocalCritDamage, Flat) => merged.push(format!(
                "Adds {:.0}% Critical Strike Damage",
                effect.value * 100.0
            )),
            (LocalCritDamage, Multiplier) => merged.push(format!(
                "{:.0}% Increased Critical Strike Damage",
                effect.value * 100.0
            )),
            (GlobalLife, Flat) => merged.push(format!("Adds {:.0} Maximum Life", effect.value)),
            (GlobalLife, Multiplier) => merged.push(format!(
                "{:.0}% Increased Maximum Life",
                effect.value * 100.0
            )),
            (GlobalLifeRegen, Flat) => merged.push(format!(
                "Adds {:.2} Life Regeneration per second",
                effect.value
            )),
            (GlobalLifeRegen, Multiplier) => merged.push(format!(
                "{:.0}% Increased Life Regeneration",
                effect.value * 100.0
            )),
            (GlobalMana, Flat) => merged.push(format!("Adds {:.0} Maximum Mana", effect.value)),
            (GlobalMana, Multiplier) => merged.push(format!(
                "{:.0}% Increased Maximum Mana",
                effect.value * 100.0
            )),
            (GlobalManaRegen, Flat) => merged.push(format!(
                "Adds {:.0} Mana Regeneration per second",
                effect.value
            )),
            (GlobalManaRegen, Multiplier) => merged.push(format!(
                "{:.0}% Increased Mana Regeneration",
                effect.value * 100.0
            )),
            (GlobalArmor, Flat) => merged.push(format!("Adds {:.0} Armor", effect.value)),
            (GlobalArmor, Multiplier) => merged.push(format!(
                "{:.0}% Increased Global Armor",
                effect.value * 100.0
            )),
            (GlobalDamage(damage_type), Flat) => merged.push(format!(
                "Adds {:.0} {} Damage",
                effect.value,
                damage_type_str(damage_type)
            )),
            (GlobalDamage(damage_type), Multiplier) => merged.push(format!(
                "{:.0}% Increased Global {} Damage",
                effect.value * 100.0,
                damage_type_str(damage_type)
            )),
            (GlobalAttackDamage, Flat) => {
                merged.push(format!("Adds {:.0} Attack Damage", effect.value))
            }
            (GlobalAttackDamage, Multiplier) => merged.push(format!(
                "{:.0}% Increased Global Attack Damage",
                effect.value * 100.0
            )),
            (GlobalSpellDamage, Flat) => {
                merged.push(format!("Adds {:.0} Spell Damage", effect.value))
            }
            (GlobalSpellDamage, Multiplier) => merged.push(format!(
                "{:.0}% Increased Spell Damage",
                effect.value * 100.0
            )),
            (GlobalCritChances, Flat) => merged.push(format!(
                "Adds {:.2}% Global Critical Strike Chances",
                effect.value * 100.0
            )),
            (GlobalCritChances, Multiplier) => merged.push(format!(
                "{:.0}% Increased Global Critical Strike Chances",
                effect.value * 100.0
            )),
            (GlobalCritDamage, Flat) => merged.push(format!(
                "Adds {:.0}% Global Critical Strike Damage",
                effect.value * 100.0
            )),
            (GlobalCritDamage, Multiplier) => merged.push(format!(
                "{:.0}% Increased Global Critical Damage",
                effect.value * 100.0
            )),
            (GlobalAttackSpeed, Flat) => {
                merged.push(format!("-{:.2}s Global Attack Cooldown", effect.value))
            }
            (GlobalAttackSpeed, Multiplier) => merged.push(format!(
                "{:.0}% Increased Attack Speed",
                effect.value * 100.0
            )),
            (GlobalSpellSpeed, Flat) => {
                merged.push(format!("-{:.2}s Spell Spell Cooldown", effect.value))
            }
            (GlobalSpellSpeed, Multiplier) => merged.push(format!(
                "{:.0}% Increased Casting Speed",
                effect.value * 100.0
            )),
            (GlobalSpeed, Flat) => merged.push(format!("-{:.2}s Global Cooldown", effect.value)),
            (GlobalSpeed, Multiplier) => merged.push(format!(
                "{:.0}% Increased Action Speed",
                effect.value * 100.0
            )),
            (GlobalMovementSpeed, Flat) => {
                merged.push(format!("-{:.2}s Movement Cooldown", effect.value))
            }
            (GlobalMovementSpeed, Multiplier) => merged.push(format!(
                "{:.0}% Increased Movement Speed",
                effect.value * 100.0
            )),
            (GlobalSpellPower, Flat) => {
                merged.push(format!("Adds {:.0} Spell Power", effect.value))
            }
            (GlobalSpellPower, Multiplier) => merged.push(format!(
                "{:.0}% Increased Spell Power",
                effect.value * 100.0
            )),
        }
    }

    for damage_type in [DamageType::Physical, DamageType::Fire] {
        match (min_damage.get(&damage_type), max_damage.get(&damage_type)) {
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
    }

    merged.into_iter().rev().map(magic_affix_li).collect()
}
