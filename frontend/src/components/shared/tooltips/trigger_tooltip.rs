use leptos::prelude::*;

use shared::data::{
    item::{SkillRange, SkillShape},
    skill::TargetType,
    stat_effect::Modifier,
    temple::{StatEffect, StatType},
    trigger::{
        EventTrigger, HitTrigger, KillTrigger, StatusTrigger, TriggerEffectModifier,
        TriggerEffectModifierSource, TriggerSpecs, TriggerTarget,
    },
};

use crate::components::shared::tooltips::{
    conditions_tooltip,
    effects_tooltip::{damage_type_str, format_stat, status_type_str},
    skill_tooltip::{self, EffectLi, shape_str, skill_type_str},
};

pub fn format_trigger_modifier(
    modifier: Option<&TriggerEffectModifier>,
    suffix: &'static str,
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
            {suffix}
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

pub fn format_extra_trigger_modifiers(
    modifiers: &[TriggerEffectModifier],
) -> impl IntoView + use<> {
    let modifiers_str: Vec<_> = modifiers
        .iter()
        .filter(|modifier| match modifier.stat {
            StatType::Damage { .. } => modifier.modifier == Modifier::Multiplier,
            StatType::StatusDuration { .. } => modifier.modifier == Modifier::Multiplier,
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
    let effects = trigger
        .triggered_effect
        .effects
        .into_iter()
        .map(|x| skill_tooltip::format_effect(x, Some(&trigger.triggered_effect.modifiers)))
        .collect::<Vec<_>>();

    let target_infos = (trigger.triggered_effect.target != TriggerTarget::SameTarget)
        .then(|| format!(", {}", trigger_target_str(trigger.triggered_effect.target)));

    let shape_infos = (trigger.triggered_effect.skill_shape != SkillShape::Single)
        .then(|| format!(", {}", shape_str(trigger.triggered_effect.skill_shape)));

    view! {
        <EffectLi class:mt-2>
            {format_trigger_event(&trigger.triggered_effect.trigger)}{target_infos}{shape_infos}":"
        </EffectLi>
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
            format!("On {}Enemy Kill", format_kill_trigger(kill_trigger))
        }
        EventTrigger::OnWaveCompleted => "At the end of each Wave completed".to_string(),
        EventTrigger::OnThreatIncreased => "On Threat increased".to_string(),
        EventTrigger::OnDeath(target_type) => {
            format!("On {}Death", format_target_type(target_type))
        }
        EventTrigger::OnApplyStatus(status_trigger) => {
            format!("On Applying {}", format_status_trigger(status_trigger))
        }
    }
}

fn format_hit_trigger(hit_trigger: &HitTrigger) -> String {
    format!(
        "{}{}{}{}{}{}",
        hurt_str(hit_trigger.is_hurt),
        blocked_str(hit_trigger.is_blocked),
        critical_str(hit_trigger.is_crit),
        range_str(hit_trigger.range),
        skill_type_str(hit_trigger.skill_type),
        damage_type_str(hit_trigger.damage_type),
    )
}

fn format_status_trigger(status_trigger: &StatusTrigger) -> String {
    format!(
        "{}{}",
        skill_type_str(status_trigger.skill_type),
        status_type_str(status_trigger.status_type),
    )
}

fn trigger_target_str(trigger_target: TriggerTarget) -> &'static str {
    match trigger_target {
        TriggerTarget::SameTarget => "Same Target",
        TriggerTarget::Source => "Source",
        TriggerTarget::Me => "Self",
    }
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
        conditions_tooltip::stunned_str(kill_trigger.is_stunned),
        conditions_tooltip::debuffed_str(kill_trigger.is_debuffed),
        conditions_tooltip::damaged_over_time_str(kill_trigger.is_damaged_over_time),
    )
}

fn format_target_type(target_type: &TargetType) -> &'static str {
    match target_type {
        TargetType::Enemy => "Enemy ",
        TargetType::Friend => "Friend ",
        TargetType::Me => "",
    }
}
