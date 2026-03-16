use std::collections::HashMap;
use strum::IntoEnumIterator;

use leptos::{html::*, prelude::*};

use shared::data::{
    chance::ChanceRange,
    item_affix::AffixEffectScope,
    modifier::Modifier,
    skill::{DamageType, SkillRepeat, SkillType},
    stat_effect::{
        LuckyRollType, MinMax, StatConverterSource, StatEffect, StatSkillEffectType,
        StatStatusType, StatType,
    },
};

use crate::components::{
    data_context::DataContext,
    shared::tooltips::{
        conditions_tooltip,
        skill_tooltip::{self, restore_type_str, skill_type_str},
    },
    ui::number::format_number,
};

pub fn format_effect_value(effect: &StatEffect) -> String {
    match effect.modifier {
        Modifier::Flat => format_number(effect.value),
        Modifier::Increased => {
            let (number_value, word) = if effect.value >= 0.0 {
                (effect.value, "Increased")
            } else {
                let div = (1.0 - effect.value * 0.01).max(0.0);
                (
                    -(if div != 0.0 { effect.value / div } else { 0.0 }),
                    "Reduced",
                )
            };
            if effect.value < 0.0 && number_value.round() >= 100.0 {
                "Removes All".into()
            } else {
                format!("{}% {word}", format_number(number_value))
            }
        }
        Modifier::More => {
            let (number_value, word) = if effect.value >= 0.0 {
                (effect.value, "More")
            } else {
                let div = (1.0 - effect.value * 0.01).max(0.0);
                (-(if div != 0.0 { effect.value / div } else { 0.0 }), "Less")
            };
            if effect.value < 0.0 && number_value.round() >= 100.0 {
                "Removes All".into()
            } else {
                format!("{}% {word}", format_number(number_value))
            }
        }
    }
}

pub fn min_max_str(min_max: Option<MinMax>) -> &'static str {
    match min_max {
        Some(MinMax::Min) => "Minimum ",
        Some(MinMax::Max) => "Maximum ",
        None => "",
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

pub fn damage_over_time_type_str(damage_type: Option<DamageType>) -> &'static str {
    match damage_type {
        Some(damage_type) => match damage_type {
            DamageType::Physical => "Bleed ",
            DamageType::Fire => "Burn ",
            DamageType::Poison => "Poison ",
            DamageType::Storm => "Weather ",
        },
        None => "Damage over Time",
    }
}
pub fn damage_over_time_type_value_str(damage_type: Option<DamageType>) -> &'static str {
    match damage_type {
        Some(damage_type) => match damage_type {
            DamageType::Physical => "Bleed Damage ",
            DamageType::Fire => "Burn Damage ",
            DamageType::Poison => "Poison Damage ",
            DamageType::Storm => "Weather Damage ",
        },
        None => "Damage over Time",
    }
}

pub fn lucky_roll_str(roll_type: &LuckyRollType) -> String {
    match roll_type {
        LuckyRollType::Damage { damage_type } => {
            format!("{}Damage", damage_type_str(*damage_type))
        }
        LuckyRollType::Block => "Chance to Block".into(),
        LuckyRollType::Evade(damage_type) => format!(
            "Chance to Evade {}",
            damage_over_time_type_str(*damage_type)
        ),
        LuckyRollType::CritChance => "Chance to Critical Hit".into(),
        LuckyRollType::SuccessChance { effect_type } => {
            format!(
                "Success Chance to {}",
                stat_skill_effect_type_str(effect_type.as_ref())
            )
        }
    }
}

pub fn stat_converter_source_str(stat_converter_source: StatConverterSource) -> String {
    match stat_converter_source {
        StatConverterSource::CritDamage => "Critical Hit Damage".into(),
        StatConverterSource::Damage {
            damage_type,
            min_max,
        } => {
            format!(
                "{}Base {}Hit Damage",
                min_max_str(min_max),
                damage_type_str(damage_type)
            )
        }
        // StatConverterSource::DamageOverTime {
        //     damage_type,
        //     min_max,
        // } => {
        //     format!(
        //         "{}Base {}",
        //         min_max_str(min_max),
        //         damage_over_time_type_str(damage_type)
        //     )
        // }
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
        Some(SkillType::Other) => " to ???",
        None => "",
    }
}

fn with_skill_type_str(skill_type: Option<SkillType>) -> &'static str {
    match skill_type {
        Some(SkillType::Attack) => " with Attacks",
        Some(SkillType::Spell) => " with Spells",
        Some(SkillType::Curse) => " with Curses",
        Some(SkillType::Blessing) => " with Blessings",
        Some(SkillType::Other) => " with ???",
        None => "",
    }
}

pub fn skill_status_type_str(
    skill_type: Option<SkillType>,
    status_type: Option<&StatStatusType>,
) -> String {
    match (skill_type, status_type) {
        (None, None) => "Effects over Time".to_string(),
        (Some(SkillType::Blessing | SkillType::Curse), None) => {
            skill_type_str(skill_type).to_string()
        }
        (skill_type, status_type) => format!(
            "{}{}",
            skill_type_str(skill_type),
            opt_status_type_str(status_type)
        ),
    }
}

pub fn status_type_str(status_type: &StatStatusType) -> String {
    match status_type {
        StatStatusType::Stun => "Stun".to_string(),
        StatStatusType::DamageOverTime { damage_type } => {
            damage_over_time_type_str(*damage_type).into()
        }
        StatStatusType::StatModifier { debuff, stat } => match (stat.as_deref(), debuff) {
            (Some(StatType::Speed(_)), Some(true)) => "Slowed".to_string(),
            (_, Some(true)) => "Negative Statuses".to_string(),
            (_, Some(false)) => "Positive Statuses".to_string(),
            _ => "Statuses".to_string(),
        },
        StatStatusType::Trigger {
            trigger_id: Some(trigger_id),
            trigger_description,
        } => trigger_description.clone().unwrap_or(trigger_id.clone()),
        StatStatusType::Trigger {
            trigger_id: _,
            trigger_description: _,
        } => "Triggered Effects".to_string(),
    }
}

pub fn opt_status_type_str(status_type: Option<&StatStatusType>) -> String {
    match status_type {
        Some(status_type) => status_type_str(status_type),
        None => "Effects over Time".to_string(),
    }
}

pub fn status_type_value_str(status_type: Option<&StatStatusType>) -> String {
    match status_type {
        Some(status_type) => match status_type {
            StatStatusType::Stun => "Stun Effects".to_string(),
            StatStatusType::DamageOverTime { damage_type } => {
                // format!("{}Damage per second", damage_type_str(*damage_type))
                damage_over_time_type_value_str(*damage_type).into()
            }
            StatStatusType::StatModifier { debuff, stat } => match (stat.as_deref(), debuff) {
                (Some(StatType::Speed(_)), Some(true)) => "Slow Effects".to_string(),
                (Some(stat), Some(false)) => {
                    format!(
                        "Increased {} Status Effects",
                        format_multiplier_stat_name(stat)
                    )
                }
                (Some(stat), Some(true)) => {
                    format!(
                        "Decreased {} Status Effects",
                        format_multiplier_stat_name(stat)
                    )
                }
                (_, Some(true)) => "Negative Status Effects".to_string(),
                (_, Some(false)) => "Positive Status Effects".to_string(),
                _ => "Status Effects".to_string(),
            },
            StatStatusType::Trigger {
                trigger_id: Some(trigger_id),
                trigger_description,
            } => format!(
                "{} Effects",
                trigger_description.clone().unwrap_or(trigger_id.clone())
            ),
            StatStatusType::Trigger {
                trigger_id: _,
                trigger_description: _,
            } => "Triggered Effects".to_string(),
        },
        None => "Effects over Time".to_string(),
    }
}

pub fn stat_skill_effect_type_str(effect_type: Option<&StatSkillEffectType>) -> String {
    match effect_type {
        Some(skill_effect_type) => match skill_effect_type {
            StatSkillEffectType::FlatDamage {} => "Hit".into(),
            StatSkillEffectType::ApplyStatus { status_type } => {
                format!("Apply {}", opt_status_type_str(status_type.as_ref()))
            }
            StatSkillEffectType::Restore { restore_type } => {
                format!("Restore{}", restore_type_str(*restore_type))
            }
            StatSkillEffectType::Resurrect => "Resurrect".into(),
        },
        None => "All Skill Effects".into(),
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
                Modifier::Flat,
                Damage {
                    skill_type,
                    damage_type,
                    min_max: Some(MinMax::Min),
                },
            ) => {
                min_damage.insert(
                    (*skill_type, *damage_type, effect.value >= 0.0),
                    effect.value,
                );
            }
            (
                Modifier::Flat,
                Damage {
                    skill_type,
                    damage_type,
                    min_max: Some(MinMax::Max),
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
    } else if let StatType::StatConverter(stat_converter_specs) = &effect.stat {
        let extra_str = match stat_converter_specs.is_extra {
            true => "gained as",
            false => "converted to",
        };
        format!(
            "{}% of {} {extra_str} {}{}",
            format_flat_number(Some(effect.value), false),
            stat_converter_source_str(stat_converter_specs.source),
            modifier_str(effect.modifier),
            format_multiplier_stat_name(&stat_converter_specs.stat)
        )
    } else {
        match effect.modifier {
            Modifier::Increased | Modifier::More => format_multiplier_stat(effect),
            Modifier::Flat => format_flat_stat(&effect.stat, Some(effect.value)),
        }
    }
}

pub fn modifier_str(modifier: Modifier) -> &'static str {
    match modifier {
        Modifier::Increased => "Increased ",
        Modifier::More => "More ",
        Modifier::Flat => "",
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
        StatType::TakeFromLifeBeforeMana => "Life spent instead of Mana".to_string(),
        StatType::Block(skill_type) => format!("{}Block Chance", skill_type_str(*skill_type)),
        StatType::BlockDamageTaken => "Blocked Damage Taken".to_string(),
        StatType::EvadeDamageTaken => "Evaded Damage over Time Taken".to_string(),
        StatType::Evade(damage_type) => {
            format!(
                "Chance to Evade {}",
                damage_over_time_type_str(*damage_type)
            )
        }
        StatType::Damage {
            skill_type,
            damage_type,
            min_max,
        } => format!(
            "{}{}{}Damage",
            min_max_str(*min_max),
            damage_type_str(*damage_type),
            skill_type_str(*skill_type),
        ),
        StatType::Restore {
            restore_type,
            skill_type,
        } => {
            format!(
                "Restore{} Effects{}",
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
            min_max,
        } => {
            format!(
                "{}{}{}",
                min_max_str(*min_max),
                skill_type_str(*skill_type),
                status_type_value_str(status_type.as_ref())
            )
        }
        StatType::StatusDuration {
            status_type,
            skill_type,
        } => {
            format!(
                "{} Duration",
                skill_status_type_str(*skill_type, status_type.as_ref())
            )
        }
        StatType::StatusResistance {
            skill_type,
            status_type,
        } => {
            format!(
                "{} Resilience",
                skill_status_type_str(*skill_type, status_type.as_ref())
            )
        }
        StatType::Speed(skill_type) => format!("{}Speed", skill_type_str(*skill_type)),
        StatType::MovementSpeed => "Movement Speed".to_string(),
        StatType::GoldFind => "Gold Find".to_string(),
        StatType::ItemRarity => "Items Rarity".to_string(),
        StatType::ItemLevel => "Items Power Level".to_string(),
        StatType::GemsFind => "Gems Find".to_string(),
        StatType::RestoreOnHit {
            restore_type,
            skill_type,
        } => {
            format!(
                "{} gained on {}Hit",
                restore_type_str(Some(*restore_type)),
                skill_type_str(*skill_type)
            )
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
        } => format!(
            "Luck {}{}",
            &lucky_roll_str(roll_type),
            to_skill_type_str(*skill_type),
        ),
        StatType::SuccessChance {
            skill_type,
            effect_type,
        } => format!(
            "Chance to {}{}",
            skill_type_str(*skill_type),
            stat_skill_effect_type_str(effect_type.as_ref())
        ),
        StatType::SkillLevel(skill_type) => format!("{} Skill Level", skill_type_str(*skill_type)),
        StatType::SkillConditionalModifier {
            stat,
            skill_type,
            conditions,
        } => format!(
            "{}{} against {}Enemies{}",
            format_multiplier_stat_name(stat),
            with_skill_type_str(*skill_type),
            conditions_tooltip::format_skill_modifier_conditions_pre(conditions, ""),
            conditions_tooltip::format_skill_modifier_conditions_post(conditions)
        ),
        StatType::SkillTargetModifier { .. } => "TODO?".into(),
        StatType::StatConditionalModifier {
            stat,
            conditions,
            conditions_duration,
        } => format!(
            "{} {}{}{}",
            format_multiplier_stat_name(stat),
            conditions_tooltip::format_skill_modifier_conditions_pre(conditions, "when "),
            conditions_tooltip::format_skill_modifier_conditions_post(conditions),
            conditions_tooltip::format_conditions_duration(*conditions_duration),
        ),
        StatType::StatConverter(stat_converter_specs) => {
            if stat_converter_specs.is_extra {
                format!(
                    "Gain {} as {}",
                    stat_converter_source_str(stat_converter_specs.source),
                    format_multiplier_stat_name(&stat_converter_specs.stat)
                )
            } else {
                format!(
                    "Convert {} to {}",
                    stat_converter_source_str(stat_converter_specs.source),
                    format_multiplier_stat_name(&stat_converter_specs.stat)
                )
            }
        }
        StatType::Description(description) | StatType::Description2(description) => {
            description.clone()
        }
    }
}

pub fn format_flat_stat(stat: &StatType, value: Option<f64>) -> String {
    let data_context: DataContext = expect_context();
    match stat {
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
        StatType::TakeFromLifeBeforeMana => {
            format!(
                "{} Life spent instead of Mana",
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
        StatType::Evade(damage_type) => format!(
            "{} Chance to Evade {}",
            format_adds_removes(value, false, "%"),
            damage_over_time_type_str(*damage_type)
        ),
        StatType::EvadeDamageTaken => {
            format!(
                "Takes {}% of Evaded Damage over Time",
                format_flat_number(value, false)
            )
        }
        StatType::Damage {
            min_max: Some(_), ..
        } => "".to_string(),
        StatType::Damage {
            skill_type,
            damage_type,
            min_max: None,
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
        StatType::CritChance(skill_type) => {
            let unwrap_value = value.unwrap_or_default();
            if unwrap_value >= 100.0 {
                format!(
                    "Guaranteed Critical Hit Chance{}",
                    with_skill_type_str(*skill_type)
                )
            } else {
                format!(
                    "{} Critical Hit Chance{}",
                    format_adds_removes(value, false, "%"),
                    to_skill_type_str(*skill_type)
                )
            }
        }
        StatType::CritDamage(skill_type) => format!(
            "{} Critical Hit Damage{}",
            format_adds_removes(value, false, "%"),
            to_skill_type_str(*skill_type)
        ),
        StatType::StatusPower {
            status_type,
            skill_type,
            min_max,
        } => {
            format!(
                "{} {}{}{}",
                format_adds_removes(
                    value,
                    false,
                    if matches!(status_type, Some(StatStatusType::StatModifier { .. })) {
                        "% to"
                    } else {
                        " to"
                    }
                ),
                min_max_str(*min_max),
                skill_type_str(*skill_type),
                status_type_value_str(status_type.as_ref())
            )
        }
        StatType::StatusDuration {
            status_type,
            skill_type,
        } => {
            if value.unwrap_or_default() >= 99999.0 {
                format!(
                    "{} never expire",
                    skill_status_type_str(*skill_type, status_type.as_ref())
                )
            } else {
                format!(
                    "{} seconds duration to {}",
                    format_adds_removes(value, true, ""),
                    skill_status_type_str(*skill_type, status_type.as_ref())
                )
            }
        }
        StatType::StatusResistance {
            status_type,
            skill_type,
        } => {
            format!(
                "{}% Resilience to {}{}",
                format_adds_removes(value, false, ""),
                skill_type_str(*skill_type),
                status_type_value_str(status_type.as_ref())
            )
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
        StatType::GemsFind => format!(
            "Adds {} Gems per Champion Kill",
            format_flat_number(value, false)
        ),
        StatType::ItemRarity => format!("Adds {}% Items Rarity", format_flat_number(value, false)),
        StatType::ItemLevel => {
            format!(
                "+{} Levels to Items Power",
                format_flat_number(value, false)
            )
        }
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
        StatType::RestoreOnHit {
            restore_type,
            skill_type,
        } => format!(
            "Gain {} {} on {}Hit",
            format_flat_number(value, false),
            restore_type_str(Some(*restore_type)),
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
            let luck_type = skill_type_str(*skill_type).to_string() + &lucky_roll_str(roll_type);
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
        StatType::StatConverter(stat_converter_specs) => {
            let extra_str = match stat_converter_specs.is_extra {
                true => "gained as",
                false => "converted to",
            };
            format!(
                "{}% of {} {extra_str} {}",
                format_flat_number(value, false),
                stat_converter_source_str(stat_converter_specs.source),
                format_multiplier_stat_name(&stat_converter_specs.stat)
            )
        }
        StatType::SuccessChance {
            skill_type,
            effect_type,
        } => {
            let unwrap_value = value.unwrap_or_default();
            if unwrap_value >= 100.0 {
                format!(
                    "Guaranteed to {}{}",
                    skill_type_str(*skill_type),
                    stat_skill_effect_type_str(effect_type.as_ref())
                )
            } else if unwrap_value <= -100.0 {
                format!(
                    "Impossible to {}{}",
                    skill_type_str(*skill_type),
                    stat_skill_effect_type_str(effect_type.as_ref())
                )
            } else {
                format!(
                    "{} Chance to {}{}",
                    format_adds_removes(value, false, "%"),
                    skill_type_str(*skill_type),
                    stat_skill_effect_type_str(effect_type.as_ref())
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

        StatType::SkillTargetModifier {
            skill_id,
            skill_type,
            range,
            shape,
            repeat,
        } => {
            let skill_name = match skill_id {
                Some(skill_id) => data_context
                    .skill_specs
                    .read()
                    .get(skill_id)
                    .map(|skill| skill.base.name.clone())
                    .unwrap_or(skill_id.clone()),
                None => "".to_string(),
            };

            let range_str = match range {
                Some(range) => match range {
                    shared::data::item::SkillRange::Melee => "Melee",
                    shared::data::item::SkillRange::Distance => "Distance",
                    shared::data::item::SkillRange::Any => "Any",
                },
                None => "",
            };

            let shape_str = match shape {
                Some(shape) => skill_tooltip::shape_str(*shape),
                None => "",
            };

            let repeat_str = repeat
                .as_ref()
                .map(|repeat| {
                    skill_tooltip::repeat_str(&SkillRepeat {
                        value: ChanceRange {
                            min: repeat.min_value,
                            max: repeat.max_value,
                            lucky_chance: Default::default(),
                        },
                        target: repeat.target,
                    })
                })
                .unwrap_or_default();

            format!(
                "{skill_name}{} becomes {range_str}{shape_str}{repeat_str}",
                skill_type_str(*skill_type),
            )
        }
        StatType::SkillConditionalModifier {
            stat,
            skill_type,
            conditions,
        } => format!(
            "{}{} against {}Enemies{}",
            format_flat_stat(stat, value),
            with_skill_type_str(*skill_type),
            conditions_tooltip::format_skill_modifier_conditions_pre(conditions, ""),
            conditions_tooltip::format_skill_modifier_conditions_post(conditions)
        ),
        StatType::StatConditionalModifier {
            stat,
            conditions,
            conditions_duration,
        } => format!(
            "{} {}{}{}",
            format_flat_stat(stat, value),
            conditions_tooltip::format_skill_modifier_conditions_pre(conditions, "when"),
            conditions_tooltip::format_skill_modifier_conditions_post(conditions),
            conditions_tooltip::format_conditions_duration(*conditions_duration),
        ),
        StatType::Description(description) | StatType::Description2(description) => {
            description.clone()
        }
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
