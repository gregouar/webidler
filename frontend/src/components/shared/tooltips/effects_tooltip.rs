use std::collections::HashMap;
use strum::IntoEnumIterator;

use leptos::{html::*, prelude::*};

use shared::data::{
    skill::{DamageType, SkillType},
    stat_effect::{Modifier, StatEffect, StatStatusType, StatType},
};

use crate::components::{
    shared::tooltips::skill_tooltip::{restore_type_str, skill_type_str},
    ui::number::format_number,
};

pub fn format_effect_value(effect: &StatEffect) -> String {
    match effect.modifier {
        Modifier::Flat => format_number(effect.value),
        Modifier::Multiplier => {
            let (number, word) = if effect.value >= 0.0 {
                (
                    format_number(effect.value),
                    if effect.stat.is_multiplicative() {
                        "More"
                    } else {
                        "Increased"
                    },
                )
            } else {
                let div = (1.0 - effect.value * 0.01).max(0.0);
                (
                    format_number(-(if div != 0.0 { effect.value / div } else { 0.0 })),
                    if effect.stat.is_multiplicative() {
                        "Less"
                    } else {
                        "Reduced"
                    },
                )
            };
            format!("{number}% {word}")
        }
    }
}

pub fn optional_damage_type_str(damage_type: Option<DamageType>) -> &'static str {
    match damage_type {
        Some(damage_type) => match damage_type {
            DamageType::Physical => "Physical ",
            DamageType::Fire => "Fire ",
            DamageType::Poison => "Poison ",
            DamageType::Storm => "Storm ",
        },
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

fn status_type_str(status_type: Option<StatStatusType>) -> String {
    match status_type {
        Some(status_type) => match status_type {
            StatStatusType::Stun => "Stun".to_string(),
            StatStatusType::DamageOverTime { damage_type } => {
                format!("{}Damage over Time", optional_damage_type_str(damage_type))
            }
            StatStatusType::StatModifier { debuff } => match debuff {
                Some(true) => "Curses".to_string(),
                Some(false) => "Blessings".to_string(),
                None => "Curses and Blessings".to_string(),
            },
        },
        None => "Effects over Time".to_string(),
    }
}

// fn scope_str(scope: AffixEffectScope) -> &'static str {
//     match scope {
//         AffixEffectScope::Local => "",
//         AffixEffectScope::Global => "Global ",
//     }
// }

fn effect_li(text: String) -> impl IntoView {
    view! { <li class="text-blue-400 text-xs xl:text-sm leading-snug">{text}</li> }
}

pub fn formatted_effects_list(
    mut affix_effects: Vec<StatEffect>,
    // scope: AffixEffectScope,
) -> Vec<impl IntoView> {
    use Modifier::*;
    use StatType::*;

    // let _ = scope; // TODO: maybe later display scope for some effects like armor

    affix_effects.sort_by_key(|effect| (effect.stat, effect.modifier));

    let mut merged: Vec<String> = Vec::with_capacity(affix_effects.len());

    // This will be used to merge added min and added max damage together
    let mut min_damage: HashMap<(Option<SkillType>, Option<DamageType>, bool), f64> =
        HashMap::new();
    let mut max_damage: HashMap<(Option<SkillType>, Option<DamageType>, bool), f64> =
        HashMap::new();

    for effect in affix_effects.iter().rev() {
        match effect.modifier {
            Multiplier => merged.push(format!(
                "{} {}",
                format_effect_value(effect),
                // scope_str(scope),
                format_multiplier_stat_name(effect.stat),
            )),
            Flat => match effect.stat {
                // Save to aggregate after
                MinDamage {
                    skill_type,
                    damage_type,
                } => {
                    min_damage.insert((skill_type, damage_type, effect.value >= 0.0), effect.value);
                }
                MaxDamage {
                    skill_type,
                    damage_type,
                } => {
                    max_damage.insert((skill_type, damage_type, effect.value >= 0.0), effect.value);
                }
                //
                stat => merged.push(format_flat_stat(stat, Some(effect.value))),
            },
        }
    }

    // Merge min and max added damages if possible
    for skill_type in SkillType::iter().map(Some).chain([None]) {
        for damage_type in DamageType::iter().map(Some).chain([None]) {
            match (
                min_damage.get(&(skill_type, damage_type, true)),
                max_damage.get(&(skill_type, damage_type, true)),
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

            match (
                min_damage.get(&(skill_type, damage_type, false)),
                max_damage.get(&(skill_type, damage_type, false)),
            ) {
                (Some(min_flat), Some(max_flat)) => merged.push(format!(
                    "Removes {} - {} {}Damage{}",
                    format_number(-*min_flat),
                    format_number(-*max_flat),
                    optional_damage_type_str(damage_type),
                    to_skill_type_str(skill_type)
                )),
                (Some(min_flat), None) => merged.push(format!(
                    "Removes {} Minimum {}Damage{}",
                    format_number(-*min_flat),
                    optional_damage_type_str(damage_type),
                    to_skill_type_str(skill_type)
                )),
                (None, Some(max_flat)) => merged.push(format!(
                    "Removes {} Maximum {}Damage{}",
                    format_number(-*max_flat),
                    optional_damage_type_str(damage_type),
                    to_skill_type_str(skill_type)
                )),
                _ => {}
            }
        }
    }

    merged.into_iter().rev().map(effect_li).collect()
}

pub fn format_multiplier_stat_name(stat: StatType) -> String {
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
        StatType::Block => "Block Chance".to_string(),
        StatType::BlockSpell => "Block Chance applied to Spells".to_string(),
        StatType::BlockDamageTaken => "Blocked Damage Taken".to_string(),
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
        StatType::Restore(restore_type) => {
            format!("Restore{} Power", restore_type_str(restore_type))
        }
        StatType::SpellPower => "Spell Power".to_string(),
        StatType::CritChance(skill_type) => {
            format!("{}Critical Hit Chance", skill_type_str(skill_type))
        }
        StatType::CritDamage(skill_type) => {
            format!("{}Critical Hit Damages", skill_type_str(skill_type))
        }
        StatType::StatusPower(status_type) => {
            format!("{} Power", status_type_str(status_type))
        }
        StatType::StatusDuration(status_type) => {
            format!("{} Duration", status_type_str(status_type))
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
        StatType::ThreatGain => "Threat Gain".into(),
    }
}

pub fn format_flat_stat(stat: StatType, value: Option<f64>) -> String {
    match stat {
        StatType::MinDamage { .. } | StatType::MaxDamage { .. } => "".to_string(),
        StatType::Life => format!("Adds {} Maximum Life", format_flat_number(value, false)),
        StatType::LifeRegen => format!(
            "Adds {}% Life Regeneration per second",
            format_flat_number(value, true)
        ),
        StatType::Mana => format!("Adds {} Maximum Mana", format_flat_number(value, false)),
        StatType::ManaRegen => format!(
            "Adds {}% Mana Regeneration per second",
            format_flat_number(value, true)
        ),
        StatType::Armor(armor_type) => format!(
            "Adds {} {}",
            format_flat_number(value, false),
            match armor_type {
                DamageType::Physical => "Armor".to_string(),
                _ => format!("{}Resistance", optional_damage_type_str(Some(armor_type))),
            }
        ),
        StatType::TakeFromManaBeforeLife => {
            format!(
                "{}% of Damage taken from Mana before Life",
                format_flat_number(value, false)
            )
        }
        StatType::Block => format!("Adds {}% Block Chance", format_flat_number(value, false)),
        StatType::BlockSpell => format!(
            "Adds {}% of Block Chance to Spells",
            format_flat_number(value, false)
        ),
        StatType::BlockDamageTaken => {
            format!(
                "Takes {}% of Blocked Damages",
                format_flat_number(value, false)
            )
        }
        StatType::Damage {
            skill_type,
            damage_type,
        } => format!(
            "Adds {} {}Damage{}",
            format_flat_number(value, false),
            optional_damage_type_str(damage_type),
            to_skill_type_str(skill_type)
        ),
        StatType::Restore(restore_type) => {
            format!(
                "Restore {} more{}",
                format_flat_number(value, false),
                restore_type_str(restore_type)
            )
        }
        StatType::SpellPower => {
            format!("Adds {} Power to Spells", format_flat_number(value, false))
        }
        StatType::CritChance(skill_type) => format!(
            "Adds {}% Critical Hit Chance{}",
            format_flat_number(value, false),
            to_skill_type_str(skill_type)
        ),
        StatType::CritDamage(skill_type) => format!(
            "Adds {}% Critical Hit Damage{}",
            format_flat_number(value, false),
            to_skill_type_str(skill_type)
        ),
        StatType::StatusPower(status_type) => format!(
            "Adds {} Power to {}",
            format_flat_number(value, false),
            status_type_str(status_type)
        ),
        StatType::StatusDuration(status_type) => format!(
            "Adds {} seconds duration to {}",
            format_flat_number(value, true),
            status_type_str(status_type)
        ),
        StatType::Speed(skill_type) => {
            if value.unwrap_or_default() >= 0.0 {
                format!(
                    "Removes {}s Cooldown{}",
                    format_flat_number(value, true),
                    to_skill_type_str(skill_type)
                )
            } else {
                format!(
                    "Adds {}s Cooldown{}",
                    format_flat_number(value.map(|v| -v), true),
                    to_skill_type_str(skill_type)
                )
            }
        }
        StatType::MovementSpeed => {
            format!("-{}s Movement Cooldown", format_flat_number(value, true))
        }
        StatType::GoldFind => format!("Adds {} Gold per Kill", format_flat_number(value, false)),
        StatType::ThreatGain => {
            if value.unwrap_or_default() >= 0.0 {
                format!("Gain {}% Extra Threat ", format_flat_number(value, false))
            } else {
                format!(
                    "Gain {}% Less Threat",
                    format_flat_number(value.map(|v| -v), false)
                )
            }
        }
        StatType::LifeOnHit(hit_trigger) => format!(
            "Gain {} Life on {}Hit",
            format_flat_number(value, false),
            skill_type_str(hit_trigger.skill_type)
        ),
        StatType::ManaOnHit(hit_trigger) => format!(
            "Gain {} Mana on {}Hit",
            format_flat_number(value, false),
            skill_type_str(hit_trigger.skill_type)
        ),
        StatType::DamageResistance {
            skill_type,
            damage_type,
        } => {
            if value.unwrap_or_default() >= 0.0 {
                format!(
                    "Resist {}% of {}{}Damage",
                    format_flat_number(value, false),
                    optional_damage_type_str(damage_type),
                    skill_type_str(skill_type)
                )
            } else {
                format!(
                    "Take {}% Increased {}{}Damage",
                    format_flat_number(value.map(|v| -v), false),
                    optional_damage_type_str(damage_type),
                    skill_type_str(skill_type)
                )
            }
        }
    }
}

fn format_flat_number(value: Option<f64>, precise: bool) -> String {
    match value {
        Some(value) => {
            if precise {
                format!("{:.1}", value * 0.1)
            } else {
                format!("{:.0}", value)
            }
        }
        None => if precise { ".#" } else { "#" }.to_string(),
    }
}
