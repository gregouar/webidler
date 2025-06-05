use std::collections::HashMap;

use leptos::html::*;
use leptos::prelude::*;
use strum::IntoEnumIterator;

use shared::data::{
    item_affix::AffixEffectScope,
    skill::{DamageType, SkillType},
    stat_effect::{Modifier, StatEffect, StatType},
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

fn scope_str(scope: AffixEffectScope) -> &'static str {
    match scope {
        AffixEffectScope::Local => "",
        AffixEffectScope::Global => " Global",
    }
}

fn effect_li(text: String) -> impl IntoView {
    view! { <li class="text-blue-400 text-sm leading-snug">{text}</li> }
}

pub fn formatted_effects_list(
    mut affix_effects: Vec<StatEffect>,
    scope: AffixEffectScope,
) -> Vec<impl IntoView> {
    use Modifier::*;
    use StatType::*;

    affix_effects.sort_by_key(|effect| (effect.stat, effect.modifier));

    let mut merged: Vec<String> = Vec::with_capacity(affix_effects.len());

    // This will be used to merge added min and added max damage together
    let mut min_damage: HashMap<(Option<SkillType>, Option<DamageType>), f64> = HashMap::new();
    let mut max_damage: HashMap<(Option<SkillType>, Option<DamageType>), f64> = HashMap::new();

    for effect in affix_effects.iter().rev() {
        match (effect.stat, effect.modifier) {
            (
                MinDamage {
                    skill_type,
                    damage_type,
                },
                Flat,
            ) => {
                min_damage.insert((skill_type, damage_type), effect.value);
            }
            (
                MaxDamage {
                    skill_type,
                    damage_type,
                },
                Flat,
            ) => {
                max_damage.insert((skill_type, damage_type), effect.value);
            }
            (
                MinDamage {
                    skill_type,
                    damage_type,
                },
                Multiplier,
            ) => merged.push(format!(
                "{:.0}% Increased Minimum{}{} Damage",
                effect.value * 100.0,
                optional_damage_type_str(damage_type),
                skill_type_str(skill_type),
            )),
            (
                MaxDamage {
                    skill_type,
                    damage_type,
                },
                Multiplier,
            ) => merged.push(format!(
                "{:.0}% Increased Maximum{}{} Damage",
                effect.value * 100.0,
                optional_damage_type_str(damage_type),
                skill_type_str(skill_type),
            )),
            (
                Damage {
                    skill_type,
                    damage_type,
                },
                Flat,
            ) => merged.push(format!(
                "Adds {:.0}{} Damage{}",
                effect.value,
                optional_damage_type_str(damage_type),
                to_skill_type_str(skill_type)
            )),
            (
                Damage {
                    skill_type,
                    damage_type,
                },
                Multiplier,
            ) => merged.push(format!(
                "{:.0}% Increased{}{} Damage",
                effect.value * 100.0,
                optional_damage_type_str(damage_type),
                skill_type_str(skill_type),
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
                "Adds {:.0} {}",
                effect.value,
                match armor_type {
                    DamageType::Physical => "Armor".to_string(),
                    _ => format!("{} Resistance", damage_type_str(armor_type)),
                }
            )),
            (Armor(armor_type), Multiplier) => merged.push(format!(
                "{:.0}% Increased{} {}",
                effect.value * 100.0,
                scope_str(scope),
                match armor_type {
                    DamageType::Physical => "Armor".to_string(),
                    _ => format!("{} Resistance", damage_type_str(armor_type)),
                }
            )),
            (Block, Flat) => {
                merged.push(format!("Adds {:.0}% Block Chances", effect.value * 100.0))
            }
            (Block, Multiplier) => merged.push(format!(
                "{:.0}% Increased Block Chances",
                effect.value * 100.0
            )),
            (CritChances(skill_type), Flat) => merged.push(format!(
                "Adds {:.2}% Critical Hit Chances{}",
                effect.value * 100.0,
                to_skill_type_str(skill_type)
            )),
            (CritChances(skill_type), Multiplier) => merged.push(format!(
                "{:.0}% Increased{} Critical Hit Chances",
                effect.value * 100.0,
                skill_type_str(skill_type)
            )),
            (CritDamage(skill_type), Flat) => merged.push(format!(
                "Adds {:.0}% Critical Hit Damage{}",
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
            (SpellPower, Flat) => merged.push(format!("Adds {:.0} Power to Spells", effect.value)),
            (SpellPower, Multiplier) => merged.push(format!(
                "{:.0}% Increased Spell Power",
                effect.value * 100.0
            )),
            (TakeFromManaBeforeLife, Multiplier) => merged.push(format!(
                "{:.0}% Increased Damage taken from Mana before Life",
                effect.value * 100.0
            )),
            (TakeFromManaBeforeLife, Flat) => merged.push(format!(
                "{:.0}% of Damage taken from Mana before Life",
                effect.value * 100.0
            )),
            (LifeOnHit(hit_trigger), Flat) => merged.push(format!(
                "Gain {:.0} Life on{} Hit",
                effect.value,
                skill_type_str(hit_trigger.skill_type)
            )),
            (LifeOnHit(hit_trigger), Multiplier) => merged.push(format!(
                "{:.0}% Increased Life gained on{} Hit",
                effect.value * 100.0,
                skill_type_str(hit_trigger.skill_type)
            )),
            (ManaOnHit(hit_trigger), Flat) => merged.push(format!(
                "Gain {:.0} Mana on {}Hit",
                effect.value,
                skill_type_str(hit_trigger.skill_type)
            )),
            (ManaOnHit(hit_trigger), Multiplier) => merged.push(format!(
                "{:.0}% Increased Mana gained on{} Hit",
                effect.value * 100.0,
                skill_type_str(hit_trigger.skill_type)
            )),
        }
    }

    for skill_type in SkillType::iter().map(Some).chain([None]) {
        for damage_type in DamageType::iter().map(Some).chain([None]) {
            match (
                min_damage.get(&(skill_type, damage_type)),
                max_damage.get(&(skill_type, damage_type)),
            ) {
                (Some(min_flat), Some(max_flat)) => merged.push(format!(
                    "Adds {:.0} - {:.0}{} Damage{}",
                    min_flat,
                    max_flat,
                    optional_damage_type_str(damage_type),
                    to_skill_type_str(skill_type)
                )),
                (Some(min_flat), None) => merged.push(format!(
                    "Adds {:.0} Minimum{} Damage{}",
                    min_flat,
                    optional_damage_type_str(damage_type),
                    to_skill_type_str(skill_type)
                )),
                (None, Some(max_flat)) => merged.push(format!(
                    "Adds {:.0} Maximum{} Damage{}",
                    max_flat,
                    optional_damage_type_str(damage_type),
                    to_skill_type_str(skill_type)
                )),
                _ => {}
            }
        }
    }

    merged.into_iter().rev().map(effect_li).collect()
}
