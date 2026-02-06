use std::collections::HashMap;
use strum::IntoEnumIterator;

use leptos::{html::*, prelude::*};

use shared::data::{
    item_affix::AffixEffectScope,
    skill::{DamageType, SkillType},
    stat_effect::{
        LuckyRollType, Modifier, StatConverterSource, StatEffect, StatSkillEffectType,
        StatStatusType, StatType,
    },
};

use crate::components::{
    shared::tooltips::{
        conditions_tooltip,
        skill_tooltip::{restore_type_str, skill_type_str},
    },
    ui::number::format_number,
};

pub fn format_effect_value(effect: &StatEffect) -> String {
    match effect.modifier {
        Modifier::Flat => format_number(effect.value),
        Modifier::Multiplier => {
            let (number_value, word) = if effect.value >= 0.0 {
                (
                    effect.value,
                    if effect.stat.is_multiplicative() {
                        "More"
                    } else {
                        "Increased"
                    },
                )
            } else {
                let div = (1.0 - effect.value * 0.01).max(0.0);
                (
                    -(if div != 0.0 { effect.value / div } else { 0.0 }),
                    if effect.stat.is_multiplicative() {
                        "Less"
                    } else {
                        "Reduced"
                    },
                )
            };
            if effect.value < 0.0 && number_value.round() >= 100.0 {
                "Removes All".into()
            } else {
                format!("{}% {word}", format_number(number_value))
            }
        }
    }
}

pub fn damage_type_str(damage_type: Option<DamageType>) -> &'static str {
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

pub fn lucky_roll_str(roll_type: LuckyRollType) -> String {
    match roll_type {
        LuckyRollType::Damage { damage_type } => {
            format!("{}Damage", damage_type_str(damage_type))
        }
        LuckyRollType::Block => "Block Chance".into(),
        LuckyRollType::CritChance => "Critical Hit Chance".into(),
        LuckyRollType::SuccessChance => "Success Chance".into(),
    }
}

fn stat_converter_source_str(stat_converter_source: StatConverterSource) -> String {
    match stat_converter_source {
        StatConverterSource::CritDamage => "Critical Hit Damage".into(),
        StatConverterSource::Damage { damage_type } => {
            format!("Base {}Damage", damage_type_str(damage_type))
        }
        StatConverterSource::MinDamage { damage_type } => {
            format!("Minimum Base {}Damage", damage_type_str(damage_type))
        }
        StatConverterSource::MaxDamage { damage_type } => {
            format!("Maximum Base {}Damage", damage_type_str(damage_type))
        }
        StatConverterSource::ThreatLevel => "Threat Level".into(),
        StatConverterSource::MaxLife => "Maximum Life".into(),
        StatConverterSource::MaxMana => "Maximum Mana".into(),
        StatConverterSource::ManaRegen => "Mana Regeneration".into(),
        StatConverterSource::LifeRegen => "Life Regeneration".into(),
        StatConverterSource::Block(skill_type) => {
            format!("{}Block Chance", skill_type_str(Some(skill_type)))
        }
    }
}

fn to_skill_type_str(skill_type: Option<SkillType>) -> &'static str {
    match skill_type {
        Some(SkillType::Attack) => " to Attacks",
        Some(SkillType::Spell) => " to Spells",
        Some(SkillType::Curse) => " to Curses",
        Some(SkillType::Blessing) => " to Blessings",
        None => "",
    }
}

fn with_skill_type_str(skill_type: Option<SkillType>) -> &'static str {
    match skill_type {
        Some(SkillType::Attack) => " with Attacks",
        Some(SkillType::Spell) => " with Spells",
        Some(SkillType::Curse) => " with Curses",
        Some(SkillType::Blessing) => " with Blessings",
        None => "",
    }
}

pub fn status_type_str(status_type: Option<StatStatusType>) -> String {
    match status_type {
        Some(status_type) => match status_type {
            StatStatusType::Stun => "Stun".to_string(),
            StatStatusType::DamageOverTime { damage_type } => {
                format!("{}Damage over Time", damage_type_str(damage_type))
            }
            StatStatusType::StatModifier { debuff } => match debuff {
                Some(true) => "Negative Statuses".to_string(),
                Some(false) => "Positive Statuses".to_string(),
                None => "Statuses".to_string(),
            },
            StatStatusType::Trigger => "Triggered Effects".to_string(),
        },
        None => "".to_string(),
    }
}

fn stat_skill_effect_type_str(effect_type: Option<StatSkillEffectType>) -> String {
    match effect_type {
        Some(skill_effect_type) => match skill_effect_type {
            StatSkillEffectType::FlatDamage {} => "Hit".into(),
            StatSkillEffectType::ApplyStatus {} => "Apply Status".into(),
            StatSkillEffectType::Restore { restore_type } => {
                format!("Restore{}", restore_type_str(restore_type))
            }
            StatSkillEffectType::Resurrect => "Resurrect".into(),
        },
        None => "All Effects".into(),
    }
}

pub fn scope_str(scope: AffixEffectScope) -> &'static str {
    match scope {
        AffixEffectScope::Local => "Local",
        AffixEffectScope::Global => "Global",
    }
}

pub fn effect_li(text: String) -> impl IntoView {
    view! { <li class="text-blue-400 leading-snug">{text}</li> }
}

pub fn formatted_effects_list(
    mut affix_effects: Vec<StatEffect>,
    // scope: AffixEffectScope,
) -> Vec<impl IntoView> {
    use Modifier::*;
    use StatType::*;

    // let _ = scope; // TODO: maybe later display scope for some effects like armor

    affix_effects.sort_by_key(|effect| (effect.stat.clone(), effect.modifier));

    let mut merged: Vec<String> = Vec::with_capacity(affix_effects.len());

    // This will be used to merge added min and added max damage together
    let mut min_damage: HashMap<(Option<SkillType>, Option<DamageType>, bool), f64> =
        HashMap::new();
    let mut max_damage: HashMap<(Option<SkillType>, Option<DamageType>, bool), f64> =
        HashMap::new();

    for effect in affix_effects.iter().rev() {
        match (effect.modifier, &effect.stat) {
            (
                Flat,
                MinDamage {
                    skill_type,
                    damage_type,
                },
            ) => {
                min_damage.insert(
                    (*skill_type, *damage_type, effect.value >= 0.0),
                    effect.value,
                );
            }
            (
                Flat,
                MaxDamage {
                    skill_type,
                    damage_type,
                },
            ) => {
                max_damage.insert(
                    (*skill_type, *damage_type, effect.value >= 0.0),
                    effect.value,
                );
            }
            _ => merged.push(format_stat(effect)),
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
                    damage_type_str(damage_type),
                    to_skill_type_str(skill_type)
                )),
                (Some(min_flat), None) => merged.push(format!(
                    "Adds {} Minimum {}Damage{}",
                    format_number(*min_flat),
                    damage_type_str(damage_type),
                    to_skill_type_str(skill_type)
                )),
                (None, Some(max_flat)) => merged.push(format!(
                    "Adds {} Maximum {}Damage{}",
                    format_number(*max_flat),
                    damage_type_str(damage_type),
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
                    damage_type_str(damage_type),
                    to_skill_type_str(skill_type)
                )),
                (Some(min_flat), None) => merged.push(format!(
                    "Removes {} Minimum {}Damage{}",
                    format_number(-*min_flat),
                    damage_type_str(damage_type),
                    to_skill_type_str(skill_type)
                )),
                (None, Some(max_flat)) => merged.push(format!(
                    "Removes {} Maximum {}Damage{}",
                    format_number(-*max_flat),
                    damage_type_str(damage_type),
                    to_skill_type_str(skill_type)
                )),
                _ => {}
            }
        }
    }

    merged.into_iter().rev().map(effect_li).collect()
}

pub fn format_stat(effect: &StatEffect) -> String {
    if effect.value == 0.0 {
        "No Effect".to_string()
    } else {
        match effect.modifier {
            Modifier::Multiplier => format_multiplier_stat(effect),
            Modifier::Flat => format_flat_stat(&effect.stat, Some(effect.value)),
        }
    }
}

pub fn format_multiplier_stat(effect: &StatEffect) -> String {
    format!(
        "{} {}",
        format_effect_value(effect),
        // scope_str(scope),
        format_multiplier_stat_name(&effect.stat),
    )
}

pub fn format_multiplier_stat_name(stat: &StatType) -> String {
    match stat {
        StatType::Life => "Maximum Life".to_string(),
        StatType::LifeRegen => "Life Regeneration".to_string(),
        StatType::Mana => "Maximum Mana".to_string(),
        StatType::ManaRegen => "Mana Regeneration".to_string(),
        StatType::ManaCost { skill_type } => {
            format!("{}Mana cost", skill_type_str(*skill_type))
        }
        StatType::Armor(armor_type) => match armor_type {
            Some(DamageType::Physical) => "Armor".to_string(),
            None => "Resistances and Armor".to_string(),
            _ => format!("{}Resistance", damage_type_str(*armor_type)),
        },
        StatType::TakeFromManaBeforeLife => "Damage taken from Mana before Life".to_string(),
        StatType::Block(skill_type) => format!("{}Block CHance", skill_type_str(*skill_type)),
        StatType::BlockDamageTaken => "Blocked Damage Taken".to_string(),
        StatType::Damage {
            skill_type,
            damage_type,
        } => format!(
            "{}{}Damage",
            damage_type_str(*damage_type),
            skill_type_str(*skill_type),
        ),
        StatType::MinDamage {
            skill_type,
            damage_type,
        } => format!(
            "Minimum {}{}Damage",
            damage_type_str(*damage_type),
            skill_type_str(*skill_type),
        ),
        StatType::MaxDamage {
            skill_type,
            damage_type,
        } => format!(
            "Maximum {}{}Damage",
            damage_type_str(*damage_type),
            skill_type_str(*skill_type),
        ),
        StatType::Restore {
            restore_type,
            skill_type,
        } => {
            format!(
                "Restore{} Effect{}",
                restore_type_str(*restore_type),
                with_skill_type_str(*skill_type)
            )
        }
        StatType::CritChance(skill_type) => {
            format!("{}Critical Hit Chance", skill_type_str(*skill_type))
        }
        StatType::CritDamage(skill_type) => {
            format!("{}Critical Hit Damage", skill_type_str(*skill_type))
        }
        StatType::StatusPower {
            status_type,
            skill_type,
        } => {
            format!(
                "{}{} Effect",
                skill_type_str(*skill_type),
                status_type_str(*status_type)
            )
        }
        StatType::StatusDuration {
            status_type,
            skill_type,
        } => {
            format!(
                "{}{} Duration",
                skill_type_str(*skill_type),
                status_type_str(*status_type)
            )
        }
        StatType::Speed(skill_type) => format!("{}Speed", skill_type_str(*skill_type)),
        StatType::MovementSpeed => "Movement Speed".to_string(),
        StatType::GoldFind => "Gold Find".to_string(),
        StatType::ItemRarity => "Item Rarity".to_string(),
        StatType::LifeOnHit { skill_type } => {
            format!("Life gained on {}Hit", skill_type_str(*skill_type))
        }
        StatType::ManaOnHit { skill_type } => {
            format!("Mana gained on {}Hit", skill_type_str(*skill_type))
        }
        StatType::DamageResistance {
            skill_type,
            damage_type,
        } => format!(
            "{}{}Damage Resistance",
            damage_type_str(*damage_type),
            skill_type_str(*skill_type)
        ),
        StatType::ThreatGain => "Threat Gain".into(),
        StatType::Lucky {
            skill_type,
            roll_type,
        } => skill_type_str(*skill_type).to_string() + &lucky_roll_str(*roll_type),
        StatType::StatConverter(stat_converter_specs) => {
            if stat_converter_specs.is_extra {
                format!(
                    "Gain {} as {}",
                    stat_converter_source_str(stat_converter_specs.source),
                    format_multiplier_stat_name(&stat_converter_specs.target_stat)
                )
            } else {
                format!(
                    "Convert {} to {}",
                    stat_converter_source_str(stat_converter_specs.source),
                    format_multiplier_stat_name(&stat_converter_specs.target_stat)
                )
            }
        }
        StatType::SuccessChance {
            skill_type,
            effect_type,
        } => format!(
            "Success Chance to {}{}",
            skill_type_str(*skill_type),
            stat_skill_effect_type_str(*effect_type)
        ),
        StatType::SkillLevel(skill_type) => format!("{} Skill Level", skill_type_str(*skill_type)),
        StatType::SkillConditionalModifier {
            stat,
            skill_type,
            conditions,
        } => format!(
            "{}{} against {} Enemies",
            format_multiplier_stat_name(stat),
            with_skill_type_str(*skill_type),
            conditions_tooltip::format_skill_modifier_conditions(conditions)
        ),
        StatType::StatConditionalModifier { stat, conditions } => format!(
            "{} when {}",
            format_multiplier_stat_name(stat),
            conditions_tooltip::format_skill_modifier_conditions(conditions)
        ),
    }
}

pub fn format_flat_stat(stat: &StatType, value: Option<f64>) -> String {
    match stat {
        StatType::MinDamage { .. } | StatType::MaxDamage { .. } => "".to_string(),
        StatType::Life => format!("{} Maximum Life", format_adds_removes(value, false, "")),
        StatType::LifeRegen => format!(
            "{} Life Regeneration per second",
            format_adds_removes(value.map(|value| value * 0.1), true, "%")
        ),
        StatType::Mana => format!("{} Maximum Mana", format_adds_removes(value, false, "")),
        StatType::ManaRegen => format!(
            "{} Mana Regeneration per second",
            format_adds_removes(value.map(|value| value * 0.1), true, "%")
        ),
        StatType::ManaCost { skill_type } => format!(
            "{} Mana Cost{}",
            format_adds_removes(value, false, ""),
            to_skill_type_str(*skill_type)
        ),
        StatType::Armor(armor_type) => format!(
            "{} {}",
            format_adds_removes(value, false, ""),
            match armor_type {
                Some(DamageType::Physical) => "Armor".to_string(),
                None => "Resistances and Armor".to_string(),
                _ => format!("{}Resistance", damage_type_str(*armor_type)),
            }
        ),
        StatType::TakeFromManaBeforeLife => {
            format!(
                "{} Damage taken from Mana before Life",
                format_adds_removes(value, false, "% of")
            )
        }
        StatType::Block(skill_type) => format!(
            "{} {}Block Chance",
            format_adds_removes(value, false, "%"),
            skill_type_str(*skill_type)
        ),
        StatType::BlockDamageTaken => {
            format!(
                "Takes {}% of Blocked Damage",
                format_flat_number(value, false)
            )
        }
        StatType::Damage {
            skill_type,
            damage_type,
        } => format!(
            "{} {}Damage{}",
            format_adds_removes(value, false, ""),
            damage_type_str(*damage_type),
            to_skill_type_str(*skill_type)
        ),
        StatType::Restore {
            restore_type,
            skill_type,
        } => {
            format!(
                "Restore {} more{}{}",
                format_flat_number(value, false),
                restore_type_str(*restore_type),
                with_skill_type_str(*skill_type)
            )
        }
        StatType::CritChance(skill_type) => format!(
            "{} Critical Hit Chance{}",
            format_adds_removes(value, false, "%"),
            to_skill_type_str(*skill_type)
        ),
        StatType::CritDamage(skill_type) => format!(
            "{} Critical Hit Damage{}",
            format_adds_removes(value, false, "%"),
            to_skill_type_str(*skill_type)
        ),
        StatType::StatusPower {
            status_type,
            skill_type,
        } => {
            format!(
                "{} {}{}",
                format_adds_removes(value, false, " to"),
                skill_type_str(*skill_type),
                status_type_str(*status_type)
            )
        }
        StatType::StatusDuration {
            status_type,
            skill_type,
        } => {
            if value.unwrap_or_default() >= 99999.0 {
                format!(
                    "{}{} never expire",
                    skill_type_str(*skill_type),
                    status_type_str(*status_type)
                )
            } else {
                format!(
                    "{}{} seconds duration to {}",
                    format_adds_removes(value, true, ""),
                    skill_type_str(*skill_type),
                    status_type_str(*status_type)
                )
            }
        }
        StatType::Speed(skill_type) => {
            if value.unwrap_or_default() >= 0.0 {
                format!(
                    "Removes {}s Cooldown{}",
                    format_flat_number(value, true),
                    to_skill_type_str(*skill_type)
                )
            } else {
                format!(
                    "Adds {}s Cooldown{}",
                    format_flat_number(value.map(|v| -v), true),
                    to_skill_type_str(*skill_type)
                )
            }
        }
        StatType::MovementSpeed => {
            format!("-{}s Movement Cooldown", format_flat_number(value, true))
        }
        StatType::GoldFind => format!("Adds {} Gold per Kill", format_flat_number(value, false)),
        StatType::ItemRarity => format!("Adds {}% Item Rarity", format_flat_number(value, false)),
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
        StatType::LifeOnHit { skill_type } => format!(
            "Gain {} Life on {}Hit",
            format_flat_number(value, false),
            skill_type_str(*skill_type)
        ),
        StatType::ManaOnHit { skill_type } => format!(
            "Gain {} Mana on {}Hit",
            format_flat_number(value, false),
            skill_type_str(*skill_type)
        ),
        StatType::DamageResistance {
            skill_type,
            damage_type,
        } => {
            if value.unwrap_or_default() >= 0.0 {
                format!(
                    "Resist {}% of {}{}Damage",
                    format_flat_number(value, false),
                    damage_type_str(*damage_type),
                    skill_type_str(*skill_type)
                )
            } else {
                format!(
                    "Take {}% Increased {}{}Damage",
                    format_flat_number(value.map(|v| -v), false),
                    damage_type_str(*damage_type),
                    skill_type_str(*skill_type)
                )
            }
        }
        StatType::Lucky {
            skill_type,
            roll_type,
        } => {
            let luck_type = skill_type_str(*skill_type).to_string() + &lucky_roll_str(*roll_type);
            let unwrap_value = value.unwrap_or_default();
            if unwrap_value >= 100.0 {
                format!("{luck_type} is Lucky",)
            } else if unwrap_value <= -100.0 {
                format!("{luck_type} is Unlucky",)
            } else {
                format!(
                    "{} Luck Chance to {luck_type}",
                    format_adds_removes(value, false, "%")
                )
            }
        }
        StatType::StatConverter(stat_converter_specs) => match stat_converter_specs.source {
            StatConverterSource::ThreatLevel => {
                let target_stat_effect = StatEffect {
                    stat: (*stat_converter_specs.target_stat).clone(),
                    modifier: stat_converter_specs.target_modifier,
                    value: value.unwrap_or_default(),
                    bypass_ignore: false,
                };
                format!(
                    "{} {} per Threat Level",
                    format_effect_value(&target_stat_effect),
                    format_multiplier_stat_name(&target_stat_effect.stat),
                )
            }
            _ => {
                let extra_str = match stat_converter_specs.is_extra {
                    true => "gained as",
                    false => "converted to",
                };
                format!(
                    "{}% of {} {extra_str} {}{}",
                    format_flat_number(value, false),
                    stat_converter_source_str(stat_converter_specs.source),
                    match stat_converter_specs.target_modifier {
                        Modifier::Multiplier => "Increased ",
                        Modifier::Flat => "",
                    },
                    format_multiplier_stat_name(&stat_converter_specs.target_stat)
                )
            }
        },
        StatType::SuccessChance {
            skill_type,
            effect_type,
        } => {
            let unwrap_value = value.unwrap_or_default();
            if unwrap_value >= 100.0 {
                format!(
                    "Guaranteed to {}{}",
                    skill_type_str(*skill_type),
                    stat_skill_effect_type_str(*effect_type)
                )
            } else if unwrap_value <= -100.0 {
                format!(
                    "Impossible to {}{}",
                    skill_type_str(*skill_type),
                    stat_skill_effect_type_str(*effect_type)
                )
            } else {
                format!(
                    "{} Success Chance to {}{}",
                    format_adds_removes(value, false, "%"),
                    skill_type_str(*skill_type),
                    stat_skill_effect_type_str(*effect_type)
                )
            }
        }
        StatType::SkillLevel(skill_type) => {
            format!(
                "{} Level(s) to {}Skills",
                format_adds_removes(value, false, ""),
                skill_type_str(*skill_type),
            )
        }
        StatType::SkillConditionalModifier {
            stat,
            skill_type,
            conditions,
        } => format!(
            "{}{} against {} Enemies",
            format_flat_stat(stat, value),
            with_skill_type_str(*skill_type),
            conditions_tooltip::format_skill_modifier_conditions(conditions)
        ),
        StatType::StatConditionalModifier { stat, conditions } => format!(
            "{} when {}",
            format_flat_stat(stat, value),
            conditions_tooltip::format_skill_modifier_conditions(conditions)
        ),
    }
}

fn format_adds_removes(value: Option<f64>, precise: bool, separator: &str) -> String {
    if value.unwrap_or_default() >= 0.0 {
        // format!("Adds {}", format_flat_number(value, precise),)
        format!("+{}{}", format_flat_number(value, precise), separator)
    } else if value.unwrap_or_default() < -1e300 {
        "Removes".to_string()
    } else {
        // format!("Removes {}", format_flat_number(value.map(|v| -v), precise),)
        format!(
            "-{}{}",
            format_flat_number(value.map(|v| -v), precise),
            separator
        )
    }
}

fn format_flat_number(value: Option<f64>, precise: bool) -> String {
    match value {
        Some(value) => {
            if precise {
                format!("{:.1}", value)
            } else {
                format!("{:.0}", value)
            }
        }
        None => if precise { ".#" } else { "#" }.to_string(),
    }
}
