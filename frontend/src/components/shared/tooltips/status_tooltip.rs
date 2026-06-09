use std::collections::HashMap;

use leptos::{html::*, prelude::*};

use shared::data::{
    chance::ChanceRange,
    character_status::{StatusEffect, StatusEffectType, StatusModifier, StatusSpecs},
    modifier::ModifiableValue,
    stat_effect::{EffectsMap, StatEffect, StatType},
    trigger::{TriggerEffect, TriggerEffectModifier, TriggerSpecs},
    values::NonNegative,
};

use crate::components::shared::tooltips::{
    effects_tooltip,
    skill_tooltip::{EffectLi, damage_color, find_trigger_modifier, format_min_max_f64},
    trigger_tooltip::{format_trigger, format_trigger_modifier},
};

pub fn format_status_effects(
    status_specs: StatusSpecs,
    value: &ChanceRange<ModifiableValue<NonNegative>>,
    value_factor: Option<f64>,
    stacks: usize,
    modifiers: Option<&[TriggerEffectModifier]>,
    effects_map: Option<&EffectsMap>,
    character_triggers: Option<&HashMap<String, TriggerEffect>>,
) -> Option<impl IntoView + use<>> {
    // let value_factor = effects_map.map(|effects_map| {
    //     stats_computations::compute_stats_effects_status_value(
    //         effects_map,
    //         ignore_stat_effects.unwrap_or(&Default::default()),
    //         skill_id,
    //         skill_type,
    //         status_id,
    //         status_specs.damage_type,
    //     )
    // });

    let effect_lines = status_specs
        .effects
        .iter()
        .cloned()
        .filter_map(|status_effect| {
            format_status_effect_line(
                &status_specs.name,
                status_effect,
                value,
                value_factor,
                modifiers,
                effects_map,
                character_triggers,
                stacks,
            )
        })
        .collect::<Vec<_>>();

    let grant_str = if status_specs.debuff {
        "inflict"
    } else {
        "grant"
    };

    if effect_lines.is_empty() {
        None
    } else if effect_lines.len() == 1 && status_specs.damage_type.is_some() {
        let status_effect = effect_lines.into_iter().next();
        Some(view! { <EffectLi>{status_specs.name} " " {status_effect}</EffectLi> }.into_any())
    } else {
        Some(
            view! {
                <ul>
                    <EffectLi>{status_specs.name} " "{grant_str}":"</EffectLi>
                    {effect_lines
                        .into_iter()
                        .map(|line| view! { <EffectLi>{line}</EffectLi> })
                        .collect::<Vec<_>>()}
                </ul>
            }
            .into_any(),
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn format_status_effect_line(
    status_name: &str,
    status_effect: StatusEffect,
    skill_value: &ChanceRange<ModifiableValue<NonNegative>>,
    value_factor: Option<f64>,
    modifiers: Option<&[TriggerEffectModifier]>,
    effects_map: Option<&EffectsMap>,
    character_triggers: Option<&HashMap<String, TriggerEffect>>,
    stacks: usize,
) -> Option<impl IntoView + use<>> {
    let value = computed_status_effect_value(&status_effect, skill_value, stacks);
    match status_effect.status_effect_type {
        StatusEffectType::DamageOverTime { damage_type } => {
            let damage_color = damage_color(damage_type);
            let trigger_modifier_damage_str = format_trigger_modifier(
                find_trigger_modifier(
                    StatType::Damage {
                        damage_type: Some(damage_type),
                        skill_filter: Default::default(),
                        min_max: None,
                        is_hit: None,
                    },
                    modifiers,
                ),
                " as",
                value_factor,
                Some(damage_color),
                Some(status_name),
                Some(skill_value),
            );

            if value == (0.0, 0.0) && trigger_modifier_damage_str.is_none() {
                return None;
            }

            Some(view! {
                <span>
                    "Deal "
                    <span class=format!(
                        "font-semibold {damage_color}",
                    )>{format_min_max_f64(value.0, value.1)}</span> {trigger_modifier_damage_str}
                    " " {effects_tooltip::damage_type_str(Some(damage_type))} " Damage per Second"
                </span>
            }
            .into_any())
        }
        StatusEffectType::StatModifier {
            stat,
            modifier,
            debuff,
        } => {
            if value == (0.0, 0.0) && modifiers.is_none() {
                return None;
            }

            let effect = StatEffect {
                stat,
                modifier,
                value: if debuff { -value.0 } else { value.0 },
                bypass_ignore: false,
            };
            Some(if value.0 != value.1 {
                let max_effect = StatEffect {
                    value: if debuff { -value.1 } else { value.1 },
                    ..effect.clone()
                };
                view! {
                    <span class="text-blue-400 whitespace-pre-line">
                        {effects_tooltip::format_stat(&effect)} " to "
                        {effects_tooltip::format_stat(&max_effect)}
                    </span>
                }
                .into_any()
            } else {
                view! {
                    <span class="text-blue-400 whitespace-pre-line">
                        {effects_tooltip::format_stat(&effect)}
                    </span>
                }
                .into_any()
            })
        }
        StatusEffectType::Trigger {
            trigger_specs,
            inherit_owner_effects,
        } => {
            if inherit_owner_effects
                && let Some(character_triggers) = character_triggers
                && let Some(trigger_effect) =
                    character_triggers.get(&trigger_specs.trigger_effect.trigger_id)
            // && let Some(owned_trigger) = character_triggers
            //     .iter()
            //     .flat_map(|(_, triggers)| triggers)
            //     .find(|trigger| {
            //         trigger.trigger_effect.trigger_id == trigger_specs.trigger_effect.trigger_id
            //     })
            {
                return Some(
                    format_trigger(
                        TriggerSpecs {
                            trigger_effect: trigger_effect.clone(),
                            ..*trigger_specs
                        },
                        false,
                        effects_map,
                        Some(status_name),
                        Some(skill_value),
                    )
                    .into_any(),
                );
            }

            if value == (0.0, 0.0) && modifiers.is_none() {
                return None;
            }

            Some(
                format_trigger(
                    *trigger_specs,
                    false,
                    effects_map,
                    Some(status_name),
                    Some(skill_value),
                )
                .into_any(),
            )
            // view! {
            //     <span>
            //         {format_min_max_f64(value.0, value.1)} " " {trigger_text(*trigger_specs)}
            //         {trigger_value_str}
            //     </span>
            // }
            // .into_any()
        }
    }
}

fn computed_status_effect_value(
    status_effect: &StatusEffect,
    skill_value: &ChanceRange<ModifiableValue<NonNegative>>,
    stacks: usize,
) -> (f64, f64) {
    match status_effect.modifier {
        StatusModifier::Flat => (
            status_effect.value.get() * stacks as f64,
            status_effect.value.get() * stacks as f64,
        ),
        StatusModifier::Percent => {
            let factor = status_effect.value.get() * 0.01;
            (
                skill_value.min.get() * factor,
                skill_value.max.get() * factor,
            )
        }
    }
}
