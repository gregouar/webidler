use shared::data::{
    conditional_modifier::Condition,
    skill::{DamageType, SkillType},
    stat_effect::StatStatusType,
};

use crate::components::shared::tooltips::skill_tooltip::skill_type_str;

pub fn format_skill_modifier_conditions(conditions: &[Condition]) -> String {
    // TODO: sort?
    conditions
        .iter()
        .map(|condition| match condition {
            Condition::HasStatus {
                status_type,
                skill_type,
            } => format_status_type_condition(*status_type, *skill_type),
            Condition::MaximumLife => "On Maximum Life".into(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_status_type_condition(
    status_type: Option<StatStatusType>,
    skill_type: Option<SkillType>,
) -> String {
    let status_type_str = match status_type {
        Some(status_type) => match status_type {
            StatStatusType::Stun => stunned_str(Some(true)),
            StatStatusType::DamageOverTime { damage_type } => damaged_over_time_str(damage_type),
            StatStatusType::StatModifier { debuff } => match debuff {
                Some(true) => debuffed_str(Some(true)),
                Some(false) => buffed_str(Some(true)),
                None => "Under Effects",
            },
            StatStatusType::Trigger => "Under Effects",
        },
        None => "",
    };

    format!("{}{}", skilled_type_str(skill_type), status_type_str)
}

pub fn skilled_type_str(skill_type: Option<SkillType>) -> &'static str {
    match skill_type {
        Some(SkillType::Attack) => "Attacked ",
        Some(SkillType::Spell) => "Spelled ",
        Some(SkillType::Curse) => "Cursed ",
        Some(SkillType::Blessing) => "Blessed ",
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
            true => "Positive ",
            false => "Non-Positive ",
        },
        None => "",
    }
}

pub fn debuffed_str(value: Option<bool>) -> &'static str {
    match value {
        Some(value) => match value {
            true => "Negative ",
            false => "Non-Negative ",
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
