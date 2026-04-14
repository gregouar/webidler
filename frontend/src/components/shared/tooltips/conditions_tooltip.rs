use shared::data::{
    conditional_modifier::Condition,
    skill::{DamageType, SkillType},
    stat_effect::{StatStatusType, StatType},
};

use crate::components::shared::tooltips::effects_tooltip;

pub fn format_skill_modifier_conditions_pre(
    conditions: &[Condition],
    prefix: &'static str,
) -> String {
    // TODO: sort?
    conditions
        .iter()
        .map(|condition| match condition {
            Condition::HasStatus {
                status_type,
                skill_type,
                not,
            } => format!(
                " {}{}{} ",
                prefix,
                if *not { "Non-" } else { "" },
                format_under_status_type_condition(status_type.as_ref(), *skill_type),
            ),
            Condition::StatusStacks { .. } => "".into(),
            Condition::MaximumLife => "".into(),
            Condition::MaximumMana => "".into(),
            Condition::LowLife => "".into(),
            Condition::LowMana => "".into(),
            Condition::ThreatLevel => "".into(),
        })
        .collect::<Vec<_>>()
        .join("")
}

pub fn format_skill_modifier_conditions_post(conditions: &[Condition]) -> String {
    // TODO: sort?
    conditions
        .iter()
        .map(|condition| match condition {
            Condition::HasStatus { .. } => "".into(),
            Condition::StatusStacks {
                status_type,
                skill_type,
            } => format!(
                " per {}",
                effects_tooltip::skill_status_type_str(*skill_type, status_type.as_ref(), false),
            ),
            Condition::MaximumLife => " on Maximum Life".into(),
            Condition::MaximumMana => " on Maximum Mana".into(),
            Condition::LowLife => " on Low Life".into(),
            Condition::LowMana => " on Low Mana".into(),
            Condition::ThreatLevel => " per Threat Level".into(),
        })
        .collect::<Vec<_>>()
        .join("")
}

pub fn format_under_status_type_condition(
    status_type: Option<&StatStatusType>,
    skill_type: Option<SkillType>,
) -> String {
    let status_type_str = match status_type {
        Some(status_type) => match status_type {
            StatStatusType::Stun => stunned_str(Some(true)).to_string(),
            StatStatusType::DamageOverTime { damage_type } => {
                damaged_over_time_str(*damage_type).to_string()
            }
            StatStatusType::StatModifier { debuff, stat } => match (stat.as_deref(), debuff) {
                (Some(StatType::Speed(_)), Some(true)) => "Slowed".into(),
                (_, Some(true)) => debuffed_str(Some(true)).to_string(),
                (_, Some(false)) => buffed_str(Some(true)).to_string(),
                _ => "Under Stats Effects".to_string(),
            },
            StatStatusType::Trigger {
                trigger_id: Some(trigger_id),
                trigger_description,
            } => trigger_description.clone().unwrap_or(trigger_id.clone()),
            StatStatusType::Trigger {
                trigger_id: _,
                trigger_description: _,
            } => "Under Trigger Effects".to_string(),
        },
        None => "".to_string(),
    };

    format!("{}{}", skilled_type_str(skill_type), status_type_str)
}

pub fn skilled_type_str(skill_type: Option<SkillType>) -> &'static str {
    match skill_type {
        Some(SkillType::Attack) => "Attacked ",
        Some(SkillType::Spell) => "Spelled ",
        Some(SkillType::Curse) => "Cursed ",
        Some(SkillType::Blessing) => "Blessed ",
        Some(SkillType::Other) => "??? ",
        None => "",
    }
}

pub fn stunned_str(value: Option<bool>) -> &'static str {
    match value {
        Some(value) => match value {
            true => "Stunned ",
            false => "Non-Stunned ",
        },
        None => "",
    }
}

pub fn buffed_str(value: Option<bool>) -> &'static str {
    match value {
        Some(value) => match value {
            true => "Buffed ",
            false => "Non-Buffed ",
        },
        None => "",
    }
}

pub fn debuffed_str(value: Option<bool>) -> &'static str {
    match value {
        Some(value) => match value {
            true => "Debuffed ",
            false => "Non-Debuffed ",
        },
        None => "",
    }
}

pub fn damaged_over_time_str(value: Option<DamageType>) -> &'static str {
    match value {
        Some(value) => match value {
            DamageType::Physical => "Bleeding ",
            DamageType::Fire => "Burning ",
            DamageType::Poison => "Poisoned ",
            DamageType::Storm => "Chilled ",
        },
        None => "",
    }
}

pub fn format_conditions_duration(conditions_duration: u32) -> String {
    if conditions_duration > 0 {
        format!(
            " during the last {:1} seconds",
            conditions_duration as f64 * 0.1
        )
    } else {
        "".into()
    }
}
