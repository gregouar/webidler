use std::collections::HashMap;
use strum::IntoEnumIterator;

use leptos::{html::*, prelude::*};

use shared::data::{
    item_affix::AffixEffectScope,
    skill::{DamageType, SkillType},
    stat_effect::{Modifier, StatEffect, StatType},
};

use crate::components::ui::number::format_number;

pub fn format_effect_value(effect: &StatEffect) -> String {
    match effect.modifier {
        Modifier::Flat => format_number(effect.value),
        Modifier::Multiplier => {
            if effect.value >= 0.0 {
                format!("{}% Increased", format_number(effect.value * 100.0))
            } else {
                let div = (1.0 - effect.value).max(0.0);
                format!(
                    "{}% Decreased",
                    format_number(-(if div != 0.0 { effect.value / div } else { 0.0 }) * 100.0)
                )
            }
        }
    }
}

pub fn optional_damage_type_str(damage_type: Option<DamageType>) -> &'static str {
    match damage_type {
        Some(DamageType::Physical) => "Physical ",
        Some(DamageType::Fire) => "Fire ",
        Some(DamageType::Poison) => "Poison ",
        None => "",
    }
}

fn skill_type_str(skill_type: Option<SkillType>) -> &'static str {
    match skill_type {
        Some(SkillType::Attack) => "Attack ",
        Some(SkillType::Spell) => "Spell ",
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
        AffixEffectScope::Global => "Global ",
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
        match effect.modifier {
            Multiplier => merged.push(format!(
                "{} {}{}",
                format_effect_value(effect),
                scope_str(scope),
                format_multiplier_stat_name(effect.stat),
            )),
            Flat => match effect.stat {
                // Save to aggregate after
                MinDamage {
                    skill_type,
                    damage_type,
                } => {
                    min_damage.insert((skill_type, damage_type), effect.value);
                }
                MaxDamage {
                    skill_type,
                    damage_type,
                } => {
                    max_damage.insert((skill_type, damage_type), effect.value);
                }
                //
                stat => merged.push(format_flat_stat(stat, effect.value)),
            },
        }
    }

    // Merge min and max added damages if possible
    for skill_type in SkillType::iter().map(Some).chain([None]) {
        for damage_type in DamageType::iter().map(Some).chain([None]) {
            match (
                min_damage.get(&(skill_type, damage_type)),
                max_damage.get(&(skill_type, damage_type)),
            ) {
                (Some(min_flat), Some(max_flat)) => merged.push(format!(
                    "Adds {} - {} {}Damage{}",
                    format_number(*min_flat),
                    format_number(*max_flat),
                    optional_damage_type_str(damage_type),
                    to_skill_type_str(skill_type)
                )),
                (Some(min_flat), None) => merged.push(format!(
                    "Adds {} Minimum {}Damage{}",
                    format_number(*min_flat),
                    optional_damage_type_str(damage_type),
                    to_skill_type_str(skill_type)
                )),
                (None, Some(max_flat)) => merged.push(format!(
                    "Adds {} Maximum {}Damage{}",
                    format_number(*max_flat),
                    optional_damage_type_str(damage_type),
                    to_skill_type_str(skill_type)
                )),
                _ => {}
            }
        }
    }

    merged.into_iter().rev().map(effect_li).collect()
}

fn format_multiplier_stat_name(stat: StatType) -> String {
    match stat {
        StatType::Life => "Maximum Life".to_string(),
        StatType::LifeRegen => "Life Regeneration".to_string(),
        StatType::Mana => "Maximum Mana".to_string(),
        StatType::ManaRegen => "Mana Regeneration".to_string(),
        StatType::Armor(armor_type) => match armor_type {
            DamageType::Physical => "Armor".to_string(),
            _ => format!("{}Resistance", optional_damage_type_str(Some(armor_type))),
        },
        StatType::TakeFromManaBeforeLife => "Damage taken from Mana before Life".to_string(),
        StatType::Block => "Block Chances".to_string(),
        StatType::Damage {
            skill_type,
            damage_type,
        } => format!(
            "{}{}Damage",
            optional_damage_type_str(damage_type),
            skill_type_str(skill_type),
        ),
        StatType::MinDamage {
            skill_type,
            damage_type,
        } => format!(
            "{}{}Minimum Damage",
            optional_damage_type_str(damage_type),
            skill_type_str(skill_type),
        ),
        StatType::MaxDamage {
            skill_type,
            damage_type,
        } => format!(
            "{}{}Maximum Damage",
            optional_damage_type_str(damage_type),
            skill_type_str(skill_type),
        ),
        StatType::SpellPower => "Spell Power".to_string(),
        StatType::CritChances(skill_type) => {
            format!("{}Critical Hit Chances", skill_type_str(skill_type))
        }
        StatType::CritDamage(skill_type) => {
            format!("{}Critical Hit Damages", skill_type_str(skill_type))
        }
        StatType::Speed(skill_type) => format!("{}Speed", skill_type_str(skill_type)),
        StatType::MovementSpeed => "Movement Speed".to_string(),
        StatType::GoldFind => "Gold Find".to_string(),
        StatType::LifeOnHit(hit_trigger) => format!(
            "Life gained on {}Hit",
            skill_type_str(hit_trigger.skill_type)
        ),
        StatType::ManaOnHit(hit_trigger) => format!(
            "Mana gained on {}Hit",
            skill_type_str(hit_trigger.skill_type)
        ),
        StatType::DamageResistance {
            skill_type,
            damage_type,
        } => format!(
            "{}{}Damage Resistance",
            optional_damage_type_str(damage_type),
            skill_type_str(skill_type)
        ),
    }
}

fn format_flat_stat(stat: StatType, value: f64) -> String {
    match stat {
        StatType::MinDamage { .. } | StatType::MaxDamage { .. } => "".to_string(),
        StatType::Life => format!("Adds {:.0} Maximum Life", value),
        StatType::LifeRegen => format!("Adds {:.2}% Life Regeneration per second", value),
        StatType::Mana => format!("Adds {:.0} Maximum Mana", value),
        StatType::ManaRegen => format!("Adds {:.2}% Mana Regeneration per second", value),
        StatType::Armor(armor_type) => format!(
            "Adds {:.0} {}",
            value,
            match armor_type {
                DamageType::Physical => "Armor".to_string(),
                _ => format!("{}Resistance", optional_damage_type_str(Some(armor_type))),
            }
        ),
        StatType::TakeFromManaBeforeLife => format!(
            "{:.0}% of Damage taken from Mana before Life",
            value * 100.0
        ),
        StatType::Block => format!("Adds {:.0}% Block Chances", value * 100.0),
        StatType::Damage {
            skill_type,
            damage_type,
        } => format!(
            "Adds {:.0} {}Damage{}",
            value,
            optional_damage_type_str(damage_type),
            to_skill_type_str(skill_type)
        ),
        StatType::SpellPower => format!("Adds {:.0} Power to Spells", value),
        StatType::CritChances(skill_type) => format!(
            "Adds {:.2}% Critical Hit Chances{}",
            value * 100.0,
            to_skill_type_str(skill_type)
        ),
        StatType::CritDamage(skill_type) => format!(
            "Adds {:.0}% Critical Hit Damage{}",
            value * 100.0,
            to_skill_type_str(skill_type)
        ),
        StatType::Speed(skill_type) => {
            format!("-{:.2}s Cooldown{}", value, to_skill_type_str(skill_type))
        }
        StatType::MovementSpeed => format!("-{:.2}s Movement Cooldown", value),
        StatType::GoldFind => format!("Adds {:.0} Gold per Kill", value),
        StatType::LifeOnHit(hit_trigger) => format!(
            "Gain {:.0} Life on{} Hit",
            value,
            skill_type_str(hit_trigger.skill_type)
        ),
        StatType::ManaOnHit(hit_trigger) => format!(
            "Gain {:.0} Mana on {}Hit",
            value,
            skill_type_str(hit_trigger.skill_type)
        ),
        StatType::DamageResistance {
            skill_type,
            damage_type,
        } => {
            if value > 0.0 {
                format!(
                    "Resist {:.0}% of {}{}Damage",
                    value * 100.0,
                    optional_damage_type_str(damage_type),
                    skill_type_str(skill_type)
                )
            } else {
                format!(
                    "Takes {:.0}% Increased {}{}Damage",
                    -value * 100.0,
                    optional_damage_type_str(damage_type),
                    skill_type_str(skill_type)
                )
            }
        }
    }
}
