use std::collections::HashMap;

use leptos::html::*;
use leptos::prelude::*;

use shared::data::{
    item_affix::{EffectModifier, EffectTarget, StatEffect},
    skill::DamageType,
};

pub fn damage_type_str(damage_type: DamageType) -> &'static str {
    match damage_type {
        DamageType::Physical => "Physical",
        DamageType::Fire => "Fire",
        DamageType::Poison => "Poison",
    }
}

fn effect_li(text: String) -> impl IntoView {
    view! { <li class="text-blue-400 text-sm leading-snug">{text}</li> }
}

pub fn formatted_effects_list(mut affix_effects: Vec<StatEffect>) -> Vec<impl IntoView> {
    use EffectModifier::*;
    use EffectTarget::*;

    affix_effects.sort_by_key(|effect| (effect.stat, effect.modifier));

    let mut merged: Vec<String> = Vec::new();

    // This will be used to merge added min and added max damage together
    let mut min_damage: HashMap<DamageType, f64> = HashMap::new();
    let mut max_damage: HashMap<DamageType, f64> = HashMap::new();

    for effect in affix_effects.iter().rev() {
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
                merged.push(format!("Adds {:.0} Attack Damage", effect.value))
            }
            (LocalAttackDamage, Multiplier) => merged.push(format!(
                "{:.0}% Increased Attack Damage",
                effect.value * 100.0
            )),
            (LocalArmor, Flat) => merged.push(format!("Adds {:.0} Armor", effect.value)),
            (LocalArmor, Multiplier) => {
                merged.push(format!("{:.0}% Increased Armor", effect.value * 100.0))
            }
            (LocalBlock, Flat) => {
                merged.push(format!("Adds {:.0}% Block Chances", effect.value * 100.0))
            }
            (LocalBlock, Multiplier) => merged.push(format!(
                "{:.0}% Increased Block Chances",
                effect.value * 100.0
            )),
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
                "Adds {:.2}% Life Regeneration per second",
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
                "Adds {:.2}% Mana Regeneration per second",
                effect.value
            )),
            (GlobalManaRegen, Multiplier) => merged.push(format!(
                "{:.0}% Increased Mana Regeneration",
                effect.value * 100.0
            )),
            (GlobalArmor(armor_type), Flat) => merged.push(format!(
                "Adds {:.0} {} {}",
                effect.value,
                damage_type_str(armor_type),
                match armor_type {
                    DamageType::Physical => "Armor",
                    _ => "Resistance",
                }
            )),
            (GlobalArmor(armor_type), Multiplier) => merged.push(format!(
                "{:.0}% Increased {} {}",
                effect.value * 100.0,
                damage_type_str(armor_type),
                match armor_type {
                    DamageType::Physical => "Armor",
                    _ => "Resistance",
                }
            )),
            (GlobalBlock, Flat) => {
                merged.push(format!("Adds {:.0}% Block Chances", effect.value * 100.0))
            }
            (GlobalBlock, Multiplier) => merged.push(format!(
                "{:.0}% Increased Global Block Chances",
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

    for damage_type in DamageType::iter() {
        match (min_damage.get(&damage_type), max_damage.get(&damage_type)) {
            (Some(min_flat), Some(max_flat)) => merged.push(format!(
                "Adds {:.0} to {:.0} {} Damage",
                min_flat,
                max_flat,
                damage_type_str(damage_type)
            )),
            (Some(min_flat), None) => merged.push(format!(
                "Adds {:.0} to Minimum {} Damage",
                min_flat,
                damage_type_str(damage_type)
            )),
            (None, Some(max_flat)) => merged.push(format!(
                "Adds {:.0} to Maximum {} Damage",
                max_flat,
                damage_type_str(damage_type)
            )),
            _ => {}
        }
    }

    merged.into_iter().rev().map(effect_li).collect()
}
