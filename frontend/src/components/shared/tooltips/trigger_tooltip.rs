use leptos::prelude::*;

use shared::data::{
    item::SkillRange,
    skill::{DamageType, TargetType},
    stat_effect::Modifier,
    temple::{StatEffect, StatType},
    trigger::{
        EventTrigger, HitTrigger, KillTrigger, TriggerEffectModifier, TriggerEffectModifierSource,
        TriggerSpecs,
    },
};

use crate::components::shared::tooltips::{
    effects_tooltip::{damage_type_str, format_stat, status_type_str},
    skill_tooltip::{self, skill_type_str, EffectLi},
};

pub fn format_trigger_modifier_as<'a>(
    modifier: Option<&'a TriggerEffectModifier>,
) -> Option<impl IntoView + use<>> {
    modifier.map(|modifier| {
        let factor_str = match modifier.modifier {
            Modifier::Multiplier => format!("{:0}", modifier.factor),
            Modifier::Flat => format!("{:0}", 100.0 * modifier.factor),
        };
        view! {
            <span class="font-semibold">{factor_str}"%"</span>
            " of "
            {trigger_modifier_source_str(modifier.source)}
            " as"
        }
    })
}

pub fn format_trigger_modifier_per(modifier: Option<&TriggerEffectModifier>) -> Option<String> {
    modifier.map(|modifier| {
        if let TriggerEffectModifierSource::HitCrit = modifier.source {
            " on Critical Hit".to_string()
        } else {
            format!(" per {}", trigger_modifier_source_str(modifier.source))
        }
    })
}

pub fn format_extra_trigger_modifiers<'a>(
    modifiers: &'a [TriggerEffectModifier],
) -> impl IntoView + use<> {
    let modifiers_str: Vec<_> = modifiers
        .iter()
        .filter(|modifier| match modifier.stat {
            StatType::Damage { .. } => modifier.modifier == Modifier::Multiplier,
            StatType::Restore(_) => modifier.modifier == Modifier::Multiplier,
            _ => true,
        })
        .map(|modifier| {
            let stat_effect = StatEffect {
                stat: modifier.stat.clone(),
                modifier: modifier.modifier,
                value: modifier.factor,
                bypass_ignore: false,
            };
            view! { <li>{format_stat(&stat_effect)}{format_trigger_modifier_per(Some(modifier))}</li> }
            .into_any()
        })
        .collect();

    view! { <ul>{modifiers_str}</ul> }
}

pub fn trigger_modifier_source_str(modifier_source: TriggerEffectModifierSource) -> String {
    match modifier_source {
        TriggerEffectModifierSource::HitDamage(damage_type) => {
            format!("{}Hit Damage", damage_type_str(damage_type))
        }
        TriggerEffectModifierSource::HitCrit => "Critical".to_string(),
        TriggerEffectModifierSource::AreaLevel => "Area Level".to_string(),
        TriggerEffectModifierSource::StatusValue(stat_status_type) => {
            format!("{} Power", status_type_str(stat_status_type))
        }
        TriggerEffectModifierSource::StatusDuration(stat_status_type) => {
            format!("{} Duration", status_type_str(stat_status_type))
        }
        TriggerEffectModifierSource::StatusStacks(stat_status_type) => {
            format!("{} Stack", status_type_str(stat_status_type))
        }
    }
}

pub fn format_trigger(trigger: TriggerSpecs) -> impl IntoView {
    // let effects = if trigger.triggered_effect.modifiers.is_empty() {
    //     trigger
    //         .triggered_effect
    //         .effects
    //         .into_iter()
    //         .map(skill_tooltip::format_effect)
    //         .collect::<Vec<_>>()
    // } else {
    //     vec![]
    // };

    let effects = trigger
        .triggered_effect
        .effects
        .into_iter()
        .map(|x| skill_tooltip::format_effect(x, Some(&trigger.triggered_effect.modifiers)))
        .collect::<Vec<_>>();

    view! {
        <EffectLi>{format_trigger_event(&trigger.triggered_effect.trigger)}":"</EffectLi>
        {if trigger.description.is_empty() {
            view! { {effects} }.into_any()
        } else {
            view! { <EffectLi>{trigger.description}</EffectLi> }.into_any()
        }}
    }
}

fn format_trigger_event(event_trigger: &EventTrigger) -> String {
    match event_trigger {
        EventTrigger::OnHit(hit_trigger) => format!("On {}Hit", format_hit_trigger(hit_trigger)),
        EventTrigger::OnTakeHit(hit_trigger) => {
            format!("On {}Hit Taken", format_hit_trigger(hit_trigger))
        }
        EventTrigger::OnKill(kill_trigger) => {
            format!("On {}Kill", format_kill_trigger(kill_trigger))
        }
        EventTrigger::OnWaveCompleted => "At the end of each Wave completed".to_string(),
        EventTrigger::OnThreatIncreased => "On Threat increased".to_string(),
        EventTrigger::OnDeath(target_type) => {
            format!("On {}Death", format_target_type(target_type))
        }
    }
}

fn format_hit_trigger(hit_trigger: &HitTrigger) -> String {
    format!(
        "{}{}{}{}{}",
        hurt_str(hit_trigger.is_hurt),
        blocked_str(hit_trigger.is_blocked),
        critical_str(hit_trigger.is_crit),
        range_str(hit_trigger.range),
        skill_type_str(hit_trigger.skill_type),
    )
}

fn range_str(value: Option<SkillRange>) -> &'static str {
    match value {
        Some(value) => match value {
            SkillRange::Melee => "Melee ",
            SkillRange::Distance => "Ranged ",
            SkillRange::Any => "",
        },
        None => "",
    }
}

fn blocked_str(value: Option<bool>) -> &'static str {
    match value {
        Some(value) => match value {
            true => "Blocked ",
            false => "Non-Blocked ",
        },
        None => "",
    }
}

fn hurt_str(value: Option<bool>) -> &'static str {
    match value {
        Some(value) => match value {
            true => "Damaging ",
            false => "Non-Damaging ",
        },
        None => "",
    }
}

fn critical_str(value: Option<bool>) -> &'static str {
    match value {
        Some(value) => match value {
            true => "Critical ",
            false => "Non-Critical ",
        },
        None => "",
    }
}

fn format_kill_trigger(kill_trigger: &KillTrigger) -> String {
    format!(
        "{}{}{}",
        stunned_str(kill_trigger.is_stunned),
        debuffed_str(kill_trigger.is_debuffed),
        damaged_over_time_str(kill_trigger.is_damaged_over_time),
    )
}

fn stunned_str(value: Option<bool>) -> &'static str {
    match value {
        Some(value) => match value {
            true => "Stunned ",
            false => "Non-Stunned ",
        },
        None => "",
    }
}
fn debuffed_str(value: Option<bool>) -> &'static str {
    match value {
        Some(value) => match value {
            true => "Cursed ",
            false => "Non-Cursed ",
        },
        None => "",
    }
}

fn damaged_over_time_str(value: Option<DamageType>) -> &'static str {
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

fn format_target_type(target_type: &TargetType) -> &'static str {
    match target_type {
        TargetType::Enemy => "Enemy ",
        TargetType::Friend => "Friend ",
        TargetType::Me => "",
    }
}
