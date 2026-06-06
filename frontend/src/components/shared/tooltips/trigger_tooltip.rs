use itertools::Itertools;
use leptos::prelude::*;

use shared::data::{
    chance::ChanceRange,
    item::{SkillRange, SkillShape},
    modifier::{ModifiableValue, Modifier},
    skill::{SkillType, TargetType},
    stat_effect::{EffectsMap, StatEffect, StatSkillFilter, StatType},
    trigger::{
        EventTrigger, HitTrigger, KillTrigger, StatusTrigger, TriggerEffectModifier,
        TriggerEffectModifierSource, TriggerSpecs, TriggerTarget,
    },
    values::NonNegative,
};

use crate::components::{
    shared::tooltips::{
        conditions_tooltip,
        effects_tooltip::{
            damage_type_str, format_stat, skill_status_filter_str, status_type_value_str,
        },
        skill_tooltip::{self, EffectLi, shape_str, skill_type_str},
    },
    ui::number::format_number,
};

pub fn format_trigger(
    trigger: TriggerSpecs,
    show_details: bool,
    effects_map: Option<&EffectsMap>,
    trigger_status_name: Option<&str>,
    trigger_status_value: Option<&ChanceRange<ModifiableValue<NonNegative>>>,
) -> impl IntoView + use<> {
    let effects = trigger
        .trigger_effect
        .effects
        .into_iter()
        .map(|x| {
            skill_tooltip::format_skill_effect(
                x,
                Some(&trigger.trigger_effect.modifiers),
                effects_map,
                trigger.trigger_effect.target == TriggerTarget::Me,
                trigger_status_name,
                trigger_status_value,
            )
        })
        .collect::<Vec<_>>();

    let details_infos = show_details.then(|| {
        if matches!(trigger.trigger_effect.skill_type, SkillType::Other) {
            format!(" ({})", trigger_target_str(trigger.trigger_effect.target))
        } else {
            format!(
                " ({}, {})",
                skill_type_str(Some(trigger.trigger_effect.skill_type)),
                trigger_target_str(trigger.trigger_effect.target)
            )
        }
    });

    let shape_infos = (trigger.trigger_effect.skill_shape != SkillShape::Single)
        .then(|| format!(", {}", shape_str(trigger.trigger_effect.skill_shape)));

    // let name_str = trigger_status_name.map(|name| format!("{} do ", name));

    view! {
        <EffectLi>
            <ul>
                <EffectLi>
                    // {name_str}
                    {format_trigger_event(&trigger.trigger)} {shape_infos} {details_infos}":"
                </EffectLi>
                {trigger
                    .description
                    .map(|description| view! { <EffectLi>{description}</EffectLi> }.into_any())
                    .unwrap_or(view! { {effects} }.into_any())}
            </ul>
        </EffectLi>
    }
}

pub fn format_trigger_modifier(
    modifier: Option<&TriggerEffectModifier>,
    suffix: &'static str,
    factor: Option<f64>,
    value_color: Option<&str>,
    trigger_status_name: Option<&str>,
    trigger_status_value: Option<&ChanceRange<ModifiableValue<NonNegative>>>,
) -> Option<impl IntoView + use<>> {
    modifier.map(|modifier| {
        let factor = modifier.factor * factor.unwrap_or(1.0);

        if let TriggerEffectModifierSource::TriggerStatusValue = modifier.source
            && let Some(trigger_status_value) = trigger_status_value
            && trigger_status_value.max.get() > 0.0
        {
            return view! {
                <span class=format!(
                    "font-semibold {}",
                    value_color.unwrap_or_default(),
                )>
                    {skill_tooltip::format_min_max(ChanceRange {
                        min: (trigger_status_value.min.get() * factor).into(),
                        max: (trigger_status_value.max.get() * factor).into(),
                        lucky_chance: trigger_status_value.lucky_chance,
                    })}
                </span>
            }
            .into_any();
        }

        let factor_str = (factor != 1.0).then(|| {
            let factor_str = match modifier.modifier {
                Modifier::Increased | Modifier::More => format!("{:0}", format_number(factor)),
                Modifier::Flat => format!("{:0}", format_number(100.0 * factor)),
            };
            view! {
                <span class="font-semibold">{factor_str}"%"</span>
                " of "
            }
        });
        view! {
            {factor_str}
            {trigger_modifier_source_str(&modifier.source, trigger_status_name)}
            {suffix}
        }
        .into_any()
    })
}

pub fn format_trigger_modifier_per(modifier: Option<&TriggerEffectModifier>) -> Option<String> {
    modifier.map(|modifier| {
        if let TriggerEffectModifierSource::HitCrit = modifier.source {
            " on Critical Hit".to_string()
        } else {
            format!(
                " per {}",
                trigger_modifier_source_str(&modifier.source, None)
            )
        }
    })
}

pub fn format_extra_trigger_modifiers(
    modifiers: &[TriggerEffectModifier],
) -> impl IntoView + use<> {
    let modifiers_str: Vec<_> = modifiers
        .iter()
        .filter(|modifier| match modifier.stat {
            StatType::Damage { .. } => modifier.modifier == Modifier::Increased,
            StatType::StatusDuration { .. } => modifier.modifier == Modifier::Increased,
            StatType::StatusPower { .. } => modifier.modifier == Modifier::Increased,
            StatType::Restore{..} => modifier.modifier == Modifier::Increased,
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

pub fn trigger_modifier_source_str(
    modifier_source: &TriggerEffectModifierSource,
    trigger_status_name: Option<&str>,
    // trigger_status_value: Option<&ChanceRange<ModifiableValue<NonNegative>>>,
) -> String {
    match modifier_source {
        TriggerEffectModifierSource::HitDamage(damage_type) => {
            format!("{}Hit Damage", damage_type_str(*damage_type))
        }
        TriggerEffectModifierSource::HitCrit => "Critical".to_string(),
        TriggerEffectModifierSource::AreaLevel => "Area Power Level".to_string(),
        TriggerEffectModifierSource::StatusValue {
            status_filter,
            skill_type,
        } => {
            format!(
                "{}{}",
                skill_type_str(*skill_type),
                status_type_value_str(status_filter)
            )
        }
        TriggerEffectModifierSource::StatusDuration {
            status_filter,
            skill_type,
        } => {
            format!(
                "{} Duration",
                skill_status_filter_str(
                    &StatSkillFilter {
                        skill_type: *skill_type,
                        ..Default::default()
                    },
                    status_filter,
                    false
                )
            )
        }
        TriggerEffectModifierSource::StatusStacks {
            status_filter,
            skill_type,
        } => {
            format!(
                "{} Stacks",
                skill_status_filter_str(
                    &StatSkillFilter {
                        skill_type: *skill_type,
                        ..Default::default()
                    },
                    status_filter,
                    false
                )
            )
        }
        TriggerEffectModifierSource::TriggerStatusDuration => match trigger_status_name {
            Some(trigger_status_name) => format!("{} Duration", trigger_status_name),
            None => "Status Duration".to_string(),
        },
        TriggerEffectModifierSource::TriggerStatusValue => match trigger_status_name {
            Some(trigger_status_name) => format!("{} Value", trigger_status_name),
            None => "Status Value".to_string(),
        },
    }
}

fn format_trigger_event(event_trigger: &EventTrigger) -> String {
    match event_trigger {
        EventTrigger::OnHit(hit_trigger) => format!(
            "On {}Hit{}",
            format_hit_trigger(hit_trigger),
            format_hit_trigger_conditions(hit_trigger, " against ", " Enemies")
        ),
        EventTrigger::OnTakeHit(hit_trigger) => match hit_trigger.is_blocked {
            Some(true) => format!("On {}Block", format_blocked_hit_trigger(hit_trigger)),
            _ => {
                format!(
                    "On {}Hit Taken{}",
                    format_hit_trigger(hit_trigger),
                    format_hit_trigger_conditions(hit_trigger, " when ", "")
                )
            }
        },
        EventTrigger::OnKill(kill_trigger) => format_kill_trigger(kill_trigger),
        EventTrigger::OnWaveCompleted => "At the end of each Wave completed".to_string(),
        EventTrigger::OnThreatIncreased => "On Threat increased".to_string(),
        EventTrigger::OnDeath(target_type) => {
            format!("On {}Death", format_target_type(target_type))
        }
        EventTrigger::OnApplyStatus(status_trigger) => {
            format!("On Applying {}", format_status_trigger(status_trigger))
        }
        EventTrigger::OnReceiveStatus(status_trigger) => match status_trigger.is_evaded {
            Some(true) => match (&status_trigger.skill_type, &status_trigger.status_filter) {
                (None, status_filter)
                    if status_filter.status_id.is_none() && status_filter.damage_type.is_none() =>
                {
                    "On Evade".to_string()
                }
                _ => format!("On Evaded {}", format_status_trigger(status_trigger)),
            },
            _ => format!("On Affected by {}", format_status_trigger(status_trigger)),
        },
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

fn format_hit_trigger_conditions(
    hit_trigger: &HitTrigger,
    prefix: &'static str,
    middlefix: &'static str,
) -> String {
    if hit_trigger.conditions.is_empty() {
        "".into()
    } else {
        format!(
            "{}{}{}{}",
            prefix,
            conditions_tooltip::format_skill_modifier_conditions_pre(&hit_trigger.conditions, ""),
            middlefix,
            conditions_tooltip::format_skill_modifier_conditions_post(&hit_trigger.conditions, "")
        )
    }
}

fn format_blocked_hit_trigger(hit_trigger: &HitTrigger) -> String {
    format!(
        "{}{}{}{}",
        critical_str(hit_trigger.is_crit),
        range_str(hit_trigger.range),
        skill_type_str(hit_trigger.skill_type),
        damage_type_str(hit_trigger.damage_type),
    )
}

fn format_status_trigger(status_trigger: &StatusTrigger) -> String {
    skill_status_filter_str(
        &StatSkillFilter {
            skill_type: status_trigger.skill_type,
            ..Default::default()
        },
        &status_trigger.status_filter,
        false,
    )
    .to_string()
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
        "On {}Enemy{} Kill",
        conditions_tooltip::format_skill_modifier_conditions_pre(&kill_trigger.conditions, ""),
        conditions_tooltip::format_skill_modifier_conditions_post(&kill_trigger.conditions, ""),
    )
}

fn format_target_type(target_type: &TargetType) -> &'static str {
    match target_type {
        TargetType::Enemy => "Enemy ",
        TargetType::Friend => "Friend ",
        TargetType::Me => "",
    }
}

pub fn trigger_text(trigger: TriggerSpecs) -> String {
    format!(
        "{} {} {}",
        format_trigger_event(&trigger.trigger),
        trigger.description.unwrap_or_default(),
        trigger
            .trigger_effect
            .effects
            .into_iter()
            .map(|x| skill_tooltip::skill_effect_text(x, Some(&trigger.trigger_effect.modifiers)))
            .join(" ")
    )
}
