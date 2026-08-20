use shared::data::{
    conditional_modifier::Condition,
    item::{ItemCategory, ItemSlot},
    skill::SkillType,
    stat_effect::{StatSkillFilter, StatStatusFilter, StatusDamageType},
};

use crate::components::{
    data_context::DataContext,
    shared::tooltips::{effects_tooltip, item_tooltip},
};

pub fn format_skill_modifier_conditions_pre(
    conditions: &[Condition],
    prefix: &'static str,
) -> String {
    // TODO: sort?
    conditions
        .iter()
        .map(|condition| match condition {
            Condition::HasStatus {
                status_filter,
                skill_type,
                not,
            } => format_adjective_status_condition(status_filter, *skill_type)
                .map(|status_str| {
                    format!(
                        " {}{}{} ",
                        prefix,
                        if *not { "Non-" } else { "" },
                        status_str,
                    )
                })
                .unwrap_or("".into()),
            Condition::StatusStacks { .. } => "".into(),
            Condition::StatusDuration { .. } => "".into(),
            Condition::Slowed => "Slowed ".into(),
            Condition::MaximumLife => "".into(),
            Condition::MaximumMana => "".into(),
            Condition::LowLife => "".into(),
            Condition::LowMana => "".into(),
            Condition::ThreatLevel => "".into(),
            Condition::HasItem { .. } => "".into(),
            // Condition::HasItem {
            //     item_slot,
            //     item_category,
            //     not,
            // } => {
            //     format!(
            //         " while {}equipped with {}",
            //         if *not { "not " } else { "" },
            //         format_has_item_condition(*item_slot, *item_category),
            //     )
            // }
        })
        .collect::<Vec<_>>()
        .join("")
}

pub fn format_skill_modifier_conditions_post(
    conditions: &[Condition],
    prefix: &'static str,
) -> String {
    // TODO: sort?
    let on_conditions_str = conditions
        .iter()
        .filter_map(|condition| match condition {
            Condition::HasStatus {
                status_filter,
                skill_type: _,
                not,
            } => format_affected_by_status_condition(status_filter).map(|status_str| format!(
                " {}{}",
                if *not { "not " } else { "" },
                status_str,
            ))
            ,
            // Condition::HasStatus { .. } => None,
            Condition::StatusStacks { .. } => None,
            Condition::StatusDuration { .. } => None,
            Condition::Slowed => None,
            Condition::MaximumLife => Some(" on Maximum Life".to_string()),
            Condition::MaximumMana => Some(" on Maximum Mana".to_string()),
            Condition::LowLife => Some(" on Low Life".to_string()),
            Condition::LowMana => Some(" on Low Mana".to_string()),
            Condition::ThreatLevel => None,
            // Condition::HasItem { .. } => None,    
             Condition::HasItem {
                item_slot,
                item_category,
                not,
            } => {
               Some(format!(
                    " {}equipped with {}",
                    if *not { "not " } else { "" },
                    format_has_item_condition(*item_slot, *item_category),
                ))
            }
        })
        .collect::<Vec<_>>();

    let per_conditions_str = conditions
        .iter()
        .filter_map(|condition| match condition {
            Condition::HasStatus { .. } => None,
            Condition::StatusStacks {
                status_filter,
                skill_type,
            } => Some(format!(
                " per Stack of {}", // on them for SkillConditionalModifier ?
                effects_tooltip::skill_status_filter_str(
                    &StatSkillFilter {
                        skill_type: *skill_type,
                        ..Default::default()
                    },
                    status_filter,
                    false
                ),
            )),
            Condition::StatusDuration {
                status_filter,
                skill_type,
            } => Some(format!(
                " per second of Duration of {}", // on them for SkillConditionalModifier ?
                effects_tooltip::skill_status_filter_str(
                    &StatSkillFilter {
                        skill_type: *skill_type,
                        ..Default::default()
                    },
                    status_filter,
                    false
                ),
            )),
            Condition::Slowed
            | Condition::MaximumLife
            | Condition::MaximumMana
            | Condition::LowLife
            | Condition::LowMana
            | Condition::HasItem { .. } => None,
            Condition::ThreatLevel => Some(" per Threat Level".into()),
        })
        .collect::<Vec<_>>();

    if on_conditions_str.is_empty() && per_conditions_str.is_empty() {
        "".into()
    } else if on_conditions_str.is_empty() {
        per_conditions_str.join(" and ")
    } else {
        format!(
            "{} {}{}",
            per_conditions_str.join(" and "),
            prefix,
            on_conditions_str.join(" and ")
        )
    }
}

fn format_has_item_condition(
    item_slot: Option<ItemSlot>,
    item_category: Option<ItemCategory>,
) -> String {
    match (item_slot, item_category) {
        (Some(item_slot), Some(item_category)) => format!(
            "{} in {} slot",
            item_tooltip::item_category_str(item_category),
            item_tooltip::item_slot_str(item_slot),
        ),
        (Some(item_slot), None) => item_tooltip::item_slot_str(item_slot).into(),
        (None, Some(item_category)) => item_tooltip::item_category_str(item_category).into(),
        (None, None) => "Item".into(),
    }
}
pub fn format_adjective_status_condition(
    status_filter: &StatStatusFilter,
    skill_type: Option<SkillType>,
) -> Option<String> {
    let status_type_str = if let Some(status_id) = &status_filter.status_id {
        let data_context: DataContext = leptos::prelude::expect_context();
        data_context.status_adjective(status_id)
    } else {
        None
        // status_filter.damage_type.map(|damage_type| {
        //     match damage_type {
        //         StatusDamageType::Any => "affected by Damage over Time ",
        //         StatusDamageType::Physical => "affected by Physical Damage over Time",
        //         StatusDamageType::Fire => "affected by Fire Damage over Time",
        //         StatusDamageType::Poison => "affected by Poison Damage over Time",
        //         StatusDamageType::Storm => "affected by Storm Damage over Time",
        //     }
        //     .to_string()
        // })
    };

    if status_type_str.is_some() || skill_type.is_some() {
        Some(format!(
            "{}{}",
            skilled_type_str(skill_type),
            status_type_str.unwrap_or("".into())
        ))
    } else {
        None
    }
}

pub fn format_affected_by_status_condition(status_filter: &StatStatusFilter) -> Option<String> {
    let status_type_str = if let Some(status_id) = &status_filter.status_id {
        let data_context: DataContext = leptos::prelude::expect_context();
        if data_context.status_adjective(status_id).is_none() {
            Some(data_context.status_name(status_id))
        } else {
            None
        }
    } else {
        status_filter.damage_type.map(|damage_type| {
            match damage_type {
                StatusDamageType::Any => "Damage over Time",
                StatusDamageType::Physical => "Physical Damage over Time",
                StatusDamageType::Fire => "Fire Damage over Time",
                StatusDamageType::Poison => "Toxic Damage over Time",
                StatusDamageType::Storm => "Storm Damage over Time",
            }
            .to_string()
        })
    };

    status_type_str.map(|status_type_str| format!("affected by {}", status_type_str))
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

pub fn format_conditions_duration(conditions_duration: u32) -> String {
    if conditions_duration > 0 {
        format!(
            " for the last {:1} seconds",
            conditions_duration as f64 * 0.1
        )
    } else {
        "".into()
    }
}
