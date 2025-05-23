use std::collections::HashMap;

use leptos::html::*;
use leptos::prelude::*;

use shared::data::item_affix::AffixEffectScope;
use shared::data::skill::SkillType;
use shared::data::{
    item_affix::{EffectModifier, StatEffect, StatType},
    skill::DamageType,
};

pub fn damage_type_str(damage_type: DamageType) -> &'static str {
    match damage_type {
        DamageType::Physical => "Physical",
        DamageType::Fire => "Fire",
        DamageType::Poison => "Poison",
    }
}

pub fn optional_damage_type_str(damage_type: Option<DamageType>) -> &'static str {
    match damage_type {
        Some(DamageType::Physical) => " Physical",
        Some(DamageType::Fire) => " Fire",
        Some(DamageType::Poison) => " Poison",
        None => "",
    }
}

fn skill_type_str(skill_type: Option<SkillType>) -> &'static str {
    match skill_type {
        Some(SkillType::Attack) => " Attack",
        Some(SkillType::Spell) => " Spell",
        None => "",
    }
}

fn to_skill_type_str(skill_type: Option<SkillType>) -> &'static str {
    match skill_type {
        Some(SkillType::Attack) => " to Attacks",
        Some(SkillType::Spell) => " to Spells",
        None => "",
    }
}

fn effect_li(text: String) -> impl IntoView {
    view! { <li class="text-blue-400 text-sm leading-snug">{text}</li> }
}

pub fn formatted_effects_list(
    mut affix_effects: Vec<StatEffect>,
    scope: AffixEffectScope,
) -> Vec<impl IntoView> {
    use EffectModifier::*;
    use StatType::*;

    // TODO: scope

    affix_effects.sort_by_key(|effect| (effect.stat, effect.modifier));

    let mut merged: Vec<String> = Vec::new();

    // This will be used to merge added min and added max damage together
    let mut min_damage: HashMap<Option<DamageType>, f64> = HashMap::new();
    let mut max_damage: HashMap<Option<DamageType>, f64> = HashMap::new();

    for effect in affix_effects.iter().rev() {
        match (effect.stat, effect.modifier) {
            // TODO skill_type
            (MinDamage((skill_type, damage_type)), Flat) => {
                min_damage.insert(damage_type, effect.value);
            }
            (MaxDamage((skill_type, damage_type)), Flat) => {
                max_damage.insert(damage_type, effect.value);
            }
            (MinDamage((skill_type, damage_type)), Multiplier) => merged.push(format!(
                "{:.0}% Increased{} Minimum{} Damage",
                effect.value * 100.0,
                skill_type_str(skill_type),
                optional_damage_type_str(damage_type)
            )),
            (MaxDamage((skill_type, damage_type)), Multiplier) => merged.push(format!(
                "{:.0}% Increased{} Maximum{} Damage",
                effect.value * 100.0,
                skill_type_str(skill_type),
                optional_damage_type_str(damage_type)
            )),
            (GoldFind, Flat) => {
                merged.push(format!("Adds {:.0} Gold per Kill", effect.value));
            }
            (GoldFind, Multiplier) => {
                merged.push(format!("{:.0}% Increased Gold Find", effect.value * 100.0))
            }
            (Life, Flat) => merged.push(format!("Adds {:.0} Maximum Life", effect.value)),
            (Life, Multiplier) => merged.push(format!(
                "{:.0}% Increased Maximum Life",
                effect.value * 100.0
            )),
            (LifeRegen, Flat) => merged.push(format!(
                "Adds {:.2}% Life Regeneration per second",
                effect.value
            )),
            (LifeRegen, Multiplier) => merged.push(format!(
                "{:.0}% Increased Life Regeneration",
                effect.value * 100.0
            )),
            (Mana, Flat) => merged.push(format!("Adds {:.0} Maximum Mana", effect.value)),
            (Mana, Multiplier) => merged.push(format!(
                "{:.0}% Increased Maximum Mana",
                effect.value * 100.0
            )),
            (ManaRegen, Flat) => merged.push(format!(
                "Adds {:.2}% Mana Regeneration per second",
                effect.value
            )),
            (ManaRegen, Multiplier) => merged.push(format!(
                "{:.0}% Increased Mana Regeneration",
                effect.value * 100.0
            )),
            (Armor(armor_type), Flat) => merged.push(format!(
                "Adds {:.0} {} {}",
                effect.value,
                damage_type_str(armor_type),
                match armor_type {
                    DamageType::Physical => "Armor",
                    _ => "Resistance",
                }
            )),
            (Armor(armor_type), Multiplier) => merged.push(format!(
                "{:.0}% Increased {} {}",
                effect.value * 100.0,
                damage_type_str(armor_type),
                match armor_type {
                    DamageType::Physical => "Armor",
                    _ => "Resistance",
                }
            )),
            (Block, Flat) => {
                merged.push(format!("Adds {:.0}% Block Chances", effect.value * 100.0))
            }
            (Block, Multiplier) => merged.push(format!(
                "{:.0}% Increased Block Chances",
                effect.value * 100.0
            )),
            (Damage((skill_type, damage_type)), Flat) => merged.push(format!(
                "Adds {:.0}{} Damage{}",
                effect.value,
                optional_damage_type_str(damage_type),
                to_skill_type_str(skill_type)
            )),
            (Damage((skill_type, damage_type)), Multiplier) => merged.push(format!(
                "{:.0}% Increased{}{} Damage",
                effect.value * 100.0,
                optional_damage_type_str(damage_type),
                skill_type_str(skill_type)
            )),
            (CritChances(skill_type), Flat) => merged.push(format!(
                "Adds {:.2}% Critical Strike Chances{}",
                effect.value * 100.0,
                to_skill_type_str(skill_type)
            )),
            (CritChances(skill_type), Multiplier) => merged.push(format!(
                "{:.0}% Increased{} Critical Strike Chances",
                effect.value * 100.0,
                skill_type_str(skill_type)
            )),
            (CritDamage(skill_type), Flat) => merged.push(format!(
                "Adds {:.0}% Critical Strike Damage{}",
                effect.value * 100.0,
                to_skill_type_str(skill_type)
            )),
            (CritDamage(skill_type), Multiplier) => merged.push(format!(
                "{:.0}% Increased{} Critical Damage",
                effect.value * 100.0,
                skill_type_str(skill_type)
            )),
            (Speed(skill_type), Flat) => merged.push(format!(
                "-{:.2}s Cooldown{}",
                effect.value,
                to_skill_type_str(skill_type)
            )),
            (Speed(skill_type), Multiplier) => merged.push(format!(
                "{:.0}% Increased{} Speed",
                effect.value * 100.0,
                skill_type_str(skill_type)
            )),
            (MovementSpeed, Flat) => {
                merged.push(format!("-{:.2}s Movement Cooldown", effect.value))
            }
            (MovementSpeed, Multiplier) => merged.push(format!(
                "{:.0}% Increased Movement Speed",
                effect.value * 100.0
            )),
            (SpellPower, Flat) => merged.push(format!("Adds {:.0} Spell Power", effect.value)),
            (SpellPower, Multiplier) => merged.push(format!(
                "{:.0}% Increased Spell Power",
                effect.value * 100.0
            )),
        }
    }

    for damage_type in DamageType::iter() {
        // TODO: get None
        match (
            min_damage.get(&Some(damage_type)),
            max_damage.get(&Some(damage_type)),
        ) {
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
