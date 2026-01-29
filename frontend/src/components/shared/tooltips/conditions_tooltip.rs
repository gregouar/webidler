use shared::data::{
    conditional_modifier::Condition, skill::DamageType, stat_effect::StatStatusType,
};

pub fn format_skill_modifier_conditions(conditions: &[Condition]) -> String {
    // TODO: sort?
    conditions
        .iter()
        .map(|condition| match condition {
            Condition::HasStatus(stat_status_type) => {
                format_status_type_condition(stat_status_type).to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_status_type_condition(stat_status_type: &StatStatusType) -> &'static str {
    match stat_status_type {
        StatStatusType::Stun => stunned_str(Some(true)),
        StatStatusType::DamageOverTime { damage_type } => damaged_over_time_str(*damage_type),
        StatStatusType::StatModifier { debuff } => match debuff {
            Some(true) => debuffed_str(Some(true)),
            Some(false) => buffed_str(Some(true)),
            None => "Under Effects",
        },
        StatStatusType::Trigger => "Under Effects",
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
            true => "Blessed ",
            false => "Non-Blessed ",
        },
        None => "",
    }
}

pub fn debuffed_str(value: Option<bool>) -> &'static str {
    match value {
        Some(value) => match value {
            true => "Cursed ",
            false => "Non-Cursed ",
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
