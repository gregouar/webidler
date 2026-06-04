use std::sync::Arc;

use itertools::Itertools;
use leptos::{html::*, prelude::*};

use shared::data::{
    chance::{Chance, ChanceRange},
    character_status::{StatusEffect, StatusEffectType, StatusModifier},
    conditional_modifier::ConditionalModifier,
    item::{ItemSlot, SkillRange, SkillShape},
    modifier::{BaseModifiableValue, ModifiableValue, Modifier},
    player::PlayerBaseSkill,
    skill::{
        DamageType, ItemStatsSource, ModifierEffect, ModifierEffectSource, RestoreModifier,
        RestoreType, SkillEffect, SkillEffectType, SkillRepeat, SkillRepeatTarget, SkillSpecs,
        SkillTargetsGroup, SkillType, TargetType,
    },
    stat_effect::{
        EffectsMap, Matchable, StatEffect, StatSkillEffectType, StatSkillFilter, StatStatusFilter,
        StatType,
    },
    trigger::TriggerEffectModifier,
    values::NonNegative,
};
use strum::IntoEnumIterator;

use crate::components::{
    data_context::DataContext,
    shared::tooltips::{
        conditions_tooltip,
        effects_tooltip::{
            self, format_multiplier_stat_name,
            formatted_effects_list, min_max_str, stat_skill_effect_type_str,
        },
        frame::{TooltipFrame, TooltipFramePalette},
        item_tooltip,
        trigger_tooltip::{
            format_extra_trigger_modifiers, format_trigger, format_trigger_modifier,
            format_trigger_modifier_per,
        },
    },
    ui::{
        Separator,
        number::{self, format_number},
    }, utils::stats_computations,
};

use super::effects_tooltip::damage_type_str;

pub fn skill_type_str(skill_type: Option<SkillType>) -> &'static str {
    match skill_type {
        Some(SkillType::Attack) => "Attack ",
        Some(SkillType::Spell) => "Spell ",
        Some(SkillType::Curse) => "Curse ",
        Some(SkillType::Blessing) => "Blessing ",
        Some(SkillType::Other) => "",
        None => "",
    }
}

pub fn skills_type_str(skill_type: Option<SkillType>) -> &'static str {
    match skill_type {
        Some(SkillType::Attack) => "Attacks ",
        Some(SkillType::Spell) => "Spells ",
        Some(SkillType::Curse) => "Curses ",
        Some(SkillType::Blessing) => "Blessings ",
        Some(SkillType::Other) => "Others ",
        None => "",
    }
}

pub fn skill_filter_str(skill_filter: &StatSkillFilter, prefix: &str, plural: bool) -> String {
    let skill_name = skill_filter
        .skill_id
        .as_ref()
        .map(|skill_id| {
            let data_context: DataContext = expect_context();
            format!("{} ", data_context.skill_name(skill_id))
        })
        .unwrap_or_default();

    let prefix = if skill_name.is_empty() && skill_filter.skill_type.is_none() {
        ""
    } else {
        prefix
    };

    format!(
        "{}{}{}",
        prefix,
        skill_name,
        if plural {
            skills_type_str(skill_filter.skill_type)
        } else {
            skill_type_str(skill_filter.skill_type)
        }
    )
}

pub fn restore_type_str(restore_type: Option<RestoreType>) -> &'static str {
    match restore_type {
        Some(RestoreType::Life) => " Life",
        Some(RestoreType::Mana) => " Mana",
        None => "",
    }
}

#[component]
pub fn SkillTooltip(
    skill_specs: Arc<SkillSpecs>,
    #[prop(default = None)] player_base_skill: Option<Arc<PlayerBaseSkill>>,
) -> impl IntoView {
    let palette = TooltipFramePalette {
        border_class: "border-[#70508a]/92",
        inner_border_class: "border-fuchsia-200/10",
        shine_color: "rgba(228,183,255,0.42)",
    };

    let targets_lines = skill_specs
        .targets
        .clone()
        .into_iter()
        .map(|target| format_target(target))
        .collect::<Vec<_>>();

    let trigger_lines = skill_specs
        .triggers
        .clone()
        .into_iter()
        .map(|trigger| format_trigger(trigger, false, None, None))
        .collect::<Vec<_>>();

    let auto_use_conditions = player_base_skill
        .as_ref()
        .map(|player_base_skill| {
            player_base_skill
                .base_skill_specs
                .auto_use_conditions
                .clone()
        })
        .unwrap_or_default();

    let modifier_lines: Vec<_> = player_base_skill
        .as_ref()
        .map(|player_base_skill| {
            player_base_skill
                .base_skill_specs
                .modifier_effects
                .clone()
                .into_iter()
                .filter(|skill_modifier| !skill_modifier.hidden)
                .map(format_skill_modifier)
                .collect()
        })
        .unwrap_or_default();

    let ignore_stat_effects: Vec<_> = skill_specs
        .ignore_stat_effects
        .clone()
        .into_iter()
        .map(format_ignored_stat)
        .collect();

    view! {
        <TooltipFrame palette class="max-w-xs">
            <strong class="text-sm xl:text-base font-bold text-violet-300 font-display text-shadow-md/80">
                <ul class="list-none xl:space-y-1 mb-2">
                    <li class=" whitespace-pre-line">{skill_specs.name.clone()}</li>
                </ul>
            </strong>

            <Separator />

            <p class="text-xs xl:text-sm text-stone-400 ">
                {skill_type_str(Some(skill_specs.skill_type))} "| "
                {if skill_specs.cooldown.get() > 0.0 {
                    view! {
                        "Cooldown: "
                        <span class="text-stone-100">
                            {format!("{:.1}s", skill_specs.cooldown.get())}
                        </span>
                    }
                        .into_any()
                } else {
                    view! { "Permanent" }.into_any()
                }}
                {(skill_specs.mana_cost.get() > 0.0)
                    .then(|| {
                        view! {
                            " | Mana Cost: "
                            <span class="text-stone-100">
                                {skill_specs.mana_cost.get().round()}
                            </span>
                        }
                    })}
            </p>

            {(!auto_use_conditions.is_empty())
                .then(|| {
                    view! {
                        <Separator />
                        <ul class="text-xs xl:text-sm ">
                            <li>
                                <span class="text-stone-400 ">
                                    "Auto-use only when "
                                    {conditions_tooltip::format_skill_modifier_conditions_pre(
                                        &auto_use_conditions,
                                        "",
                                    )}
                                    {conditions_tooltip::format_skill_modifier_conditions_post(
                                        &auto_use_conditions,
                                        "",
                                    )}
                                </span>
                            </li>
                        </ul>
                    }
                })}

            <ul class="list-none xl:space-y-1 text-xs xl:text-sm">
                {targets_lines}{trigger_lines}
                {(!modifier_lines.is_empty()).then(|| view! { <Separator /> })} {modifier_lines}
            </ul>

            <ul class="list-none xl:space-y-1 text-xs xl:text-sm">{ignore_stat_effects}</ul>

            {player_base_skill
                .as_ref()
                .filter(|player_base_skill| player_base_skill.next_upgrade_cost > 0.0)
                .map(|player_base_skill| {
                    let upgrade_effects = player_base_skill
                        .base_skill_specs
                        .upgrade_effects
                        .clone();
                    let upgrade_level = player_base_skill.upgrade_level;
                    let next_upgrade_cost = player_base_skill.next_upgrade_cost;
                    view! {
                        <Separator />
                        <ul class="text-xs xl:text-sm ">
                            <li>
                                <span class="text-stone-400 ">"Next upgrade:"</span>
                            </li>
                            {effects_tooltip::formatted_effects_list(upgrade_effects)}
                        </ul>

                        <Separator />
                        <p class="text-xs xl:text-sm text-stone-400 ">
                            "Level: "
                            {if skill_specs.level_modifier > 0 {
                                view! {
                                    <span class="text-blue-300">
                                        {upgrade_level.saturating_add(skill_specs.level_modifier)}
                                    </span>
                                }
                            } else {
                                view! { <span class="text-stone-100">{upgrade_level}</span> }
                            }} " | Upgrade Cost: "
                            <span class="text-stone-100">
                                {format_number(next_upgrade_cost)}" Gold"
                            </span>
                        </p>
                    }
                })}

            {(!skill_specs.description.is_empty())
                .then(|| {
                    view! {
                        <Separator />
                        <p class="text-xs xl:text-sm italic text-stone-400 ">
                            {skill_specs.description.clone()}
                        </p>
                    }
                })}
        </TooltipFrame>
    }
}

fn format_target(targets_group: SkillTargetsGroup) -> impl IntoView {
    let shape = shape_str(targets_group.shape);

    let range = match targets_group.range {
        SkillRange::Melee => {
            if targets_group.target_type == TargetType::Me {
                "Self"
            } else {
                "Melee"
            }
        }
        SkillRange::Distance => "Distance",
        SkillRange::Any => "Any",
    };

    let repeat = if targets_group.repeat.value.max > 1 {
        format!(", {}", repeat_str(&targets_group.repeat))
    } else {
        "".into()
    };

    let effects = targets_group
        .effects
        .into_iter()
        .map(|x| format_skill_effect( x, None, None, targets_group.target_type == TargetType::Me, None ))
        .collect::<Vec<_>>();

    view! {
        <Separator />
        <EffectLi>{range}", "{shape}{repeat}</EffectLi>
        {effects}
    }
}

pub fn repeat_str(skill_repeat: &SkillRepeat) -> String {
    format!(
        "{} {}",
        match skill_repeat.target {
            SkillRepeatTarget::Any => "Multi-Hit",
            SkillRepeatTarget::Same => "Repeat",
            SkillRepeatTarget::Different => "Chain",
        },
        format_min_max_f64(skill_repeat.value.min, skill_repeat.value.max),
    )
}

pub fn shape_str(shape: SkillShape) -> &'static str {
    match shape {
        SkillShape::Single => "Single",
        SkillShape::Vertical2 => "Area 2x1",
        SkillShape::Horizontal2 => "Area 1x2",
        SkillShape::Horizontal3 => "Area 1x3",
        SkillShape::Square4 => "Area 2x2",
        SkillShape::All => "All",
        SkillShape::Contact => "Contact",
    }
}

fn find_trigger_modifier(
    stat: StatType,
    modifiers: Option<&[TriggerEffectModifier]>,
) -> Option<&TriggerEffectModifier> {
    modifiers
        .unwrap_or_default()
        .iter()
        .find(|modifier| modifier.stat.is_match(&stat) && modifier.modifier == Modifier::Flat)
}

fn format_status_trigger_value(
    modifiers: Option<&[TriggerEffectModifier]>,
    prefix: &'static str,
    factor: Option<f64>,
    trigger_status_name: Option<&str>,
) -> Option<impl IntoView + use<>> {
    format_trigger_modifier(
        find_trigger_modifier(
            StatType::StatusPower {
                status_filter: Default::default(),
                skill_filter: Default::default(),
                min_max: None,
            },
            modifiers,
        ),
        "",
        factor,
        trigger_status_name
    )
    .map(|modifier_str| {
        view! {
            {prefix}
            {modifier_str}
        }
    })
}

fn format_status_effect_line(
    status_name: &str,
    status_effect: StatusEffect,
    skill_value: &ChanceRange<ModifiableValue<NonNegative>>,
    skill_damage_type: Option<DamageType>,
    modifiers: Option<&[TriggerEffectModifier]>,
    effects_map: Option<&EffectsMap>,
) -> impl IntoView + use<> {
    let value = computed_status_effect_value(&status_effect, skill_value);

    match status_effect.status_effect_type {
        StatusEffectType::DamageOverTime { damage_type } => {
            let damage_type = skill_damage_type.unwrap_or(damage_type);
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
                None,
                Some(status_name),
            );

            view! {
                <span>
                    <span class=format!(
                        "font-semibold {damage_color}",
                    )>{format_min_max_f64(value.0, value.1)}</span>
                    {trigger_modifier_damage_str}
                    " "
                    {damage_type_str(Some(damage_type))}
                    " Damage per Second"
                </span>
            }
            .into_any()
        }
        StatusEffectType::StatModifier {
            stat,
            modifier,
            debuff,
        } => {
            let effect = StatEffect {
                stat,
                modifier,
                value: if debuff { -value.0 } else { value.0 },
                bypass_ignore: false,
            };
            if value.0 != value.1 {
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
                }.into_any()
            }
        }
        StatusEffectType::Trigger(trigger_specs) => {
         
            format_trigger(
                *trigger_specs,
                false,
                effects_map,
                Some(status_name),
            )
            .into_any()
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
) -> (f64, f64) {
    match status_effect.modifier {
        StatusModifier::Flat => (status_effect.value.get(), status_effect.value.get()),
        StatusModifier::Percent => {
            let factor = status_effect.value.get() * 0.01;
            (
                skill_value.min.get() * factor,
                skill_value.max.get() * factor,
            )
        }
    }
}

fn format_escalation(escalation: f64) -> Option<impl IntoView + use<>> {
    (escalation > 0.0).then(|| {
        view! {
            ", Escalating by "
            {format_number(escalation)}
            "% More Damage per Second"
        }
    })
}

pub fn format_chance(chance: &Chance, precise: bool) -> String {
    if precise {
        let luck_chance = chance
            .luck_estimate()
            .map(|luck_estimate| format!(" ({:.2}%)", luck_estimate))
            .unwrap_or_default();

        format!("{:.2}%{luck_chance}", chance.value.get())
    } else {
        let luck_chance = chance
            .luck_estimate()
            .map(|luck_estimate| format!(" ({:.0}%)", luck_estimate))
            .unwrap_or_default();

        format!("{:.0}%{luck_chance}", chance.value.get())
    }
}

pub fn format_skill_effect(
    skill_effect: SkillEffect,
    modifiers: Option<&[TriggerEffectModifier]>,
    effects_map: Option<&EffectsMap>,
    self_target: bool,
    trigger_status_name: Option<&str>,
) -> impl IntoView + use<> {
    let success_chance = if skill_effect.success_chance.value.get() < 100.0 {
        Some(view! {
            <span class="font-semibold">{format_chance(&skill_effect.success_chance, false)}</span>
            " chance to "
        })
    } else {
        None
    };

    let base_effects = match skill_effect.effect_type {
        SkillEffectType::FlatDamage {
            damage,
            crit_chance,
            crit_damage,
            ..
        } => view! {
            {
                let mut damage_lines = Vec::new();
                for damage_type in DamageType::iter() {
                    let value = damage.get(&damage_type).copied().unwrap_or_default();
                    let success_chance = success_chance.clone();
                    let damage_color = damage_color(damage_type);
                    let trigger_modifier_str = format_trigger_modifier(
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
                        None,
                        trigger_status_name,
                    );
                    if value.min.get() > 0.0 || value.max.get() > 0.0
                        || trigger_modifier_str.is_some()
                    {
                        damage_lines
                            .push(
                                view! {
                                    <EffectLi>
                                        {success_chance}"Hit for "
                                        <span class=format!(
                                            "font-semibold {damage_color}",
                                        )>{format_min_max(value)}</span> {trigger_modifier_str} " "
                                        {damage_type_str(Some(damage_type))} "Damage"
                                    </EffectLi>
                                },
                            );
                    }
                }
                damage_lines
            }
            {if crit_chance.value.get() > 0.0 {
                Some(
                    view! {
                        <EffectLi>
                            "Critical Hit Chance: "
                            <span class="font-semibold">{format_chance(&crit_chance, true)}</span>
                        </EffectLi>
                        <EffectLi>
                            "Critical Hit Damage: "
                            <span class="font-semibold">
                                {format!("+{}%", number::format_number(*crit_damage))}
                            </span>
                        </EffectLi>
                    },
                )
            } else {
                None
            }}
        }
        .into_any(),
        SkillEffectType::ApplyStatus {
            status_id,
            value,
            duration,
            escalation,
            damage_type,
            ..
        } => {
            let data_context: DataContext = expect_context();
            let status_specs = data_context
                .statuses_specs
                .read_untracked()
                .get(&status_id)
                .cloned();
            let status_name = data_context.status_name(&status_id);

            match status_specs {
                Some(status_specs) => {

                        
                    let duration = duration
                        .map(|duration| (duration.min.get(), duration.max.get()))
                        .unwrap_or_else(|| {
                            (status_specs.duration.min.get(), status_specs.duration.max.get())
                        });
                    let escalation = escalation
                        .map(|escalation| escalation.get())
                        .unwrap_or(status_specs.escalation.get());
                    let status_filter = StatStatusFilter {
                        status_id: Some(status_id.clone()),
                        damage_type: damage_type.or(status_specs.damage_type).map(Into::into),
                    };
                    let trigger_modifier_duration_str = format_trigger_modifier(
                        find_trigger_modifier(
                            StatType::StatusDuration {
                                status_filter: status_filter.clone(),
                                skill_filter: Default::default(),
                            },
                            modifiers,
                        ),
                        "",
                        None,
                        trigger_status_name
                    );
                    let apply_str = if self_target {
                        "Gain"
                    } else {
                        "Inflict"
                    };

                    let stacks_str = (status_specs.max_stacks > 1).then(|| format!(", up to {} Stacks",status_specs.max_stacks ));
                    
                    let effect_lines = status_specs
                        .effects
                        .iter()
                        .cloned()
                        .map(|status_effect| {
                            format_status_effect_line(
                                &status_specs.name,
                                status_effect,
                                &value,
                                damage_type,
                                modifiers,
                                effects_map,
                            )
                        })
                        .collect::<Vec<_>>();

                    
                    let value_factor = effects_map.map(|effects_map| {
                        stats_computations::compute_stats_effects_status_value(effects_map, &status_filter)
                    });
                    let modified_value_str =
                        format_status_trigger_value(modifiers, " based on ", value_factor,trigger_status_name);

                    let empty_status = effect_lines.is_empty();
                    if empty_status {
                        view! {}
                        .into_any()
                    } 
                    // else if effect_lines.len() == 1 {
                    //     view! {
                    //         <EffectLi>
                    //             {success_chance}{apply_str}" "{status_name}" "{effect_lines}" "
                    //             {format_duration_values(duration.0, duration.1)}
                    //             {trigger_modifier_duration_str} {format_escalation(escalation)}
                    //         </EffectLi>
                    //     }
                    //     .into_any()
                    // } 
                    else {
                        view! {
                            <EffectLi>
                                {success_chance}{apply_str}" "{status_name}{modified_value_str}" "
                                {format_duration_values(duration.0, duration.1)}
                                {trigger_modifier_duration_str} {stacks_str}":"
                                <ul>
                                    {effect_lines
                                        .into_iter()
                                        .map(|line| view! { <EffectLi>{line}</EffectLi> })
                                        .collect::<Vec<_>>()}
                                </ul> {format_escalation(escalation)}
                            </EffectLi>
                        }
                        .into_any()
                    }
                }
                None => view! { <EffectLi>{success_chance}"Apply "{status_name}" " {format_min_max(value)}</EffectLi> }
                .into_any(),
            }
        }
        SkillEffectType::Restore {
            restore_type,
            value,
            modifier,
        } => {
            let trigger_modifier = find_trigger_modifier(
                StatType::Restore {
                    restore_type: Some(restore_type),
                    skill_filter: Default::default(),
                },
                modifiers,
            );
            let trigger_modifier_str = format_trigger_modifier_per(trigger_modifier);
            let trigger_modifier_factor_str =
                trigger_modifier.map(|trigger_modifier| format!("{:.0}", trigger_modifier.factor));
            view! {
                <EffectLi>
                    {success_chance}"Restore "
                    <span class="font-semibold">
                        {format_min_max(value)} {trigger_modifier_factor_str}
                        {match modifier {
                            RestoreModifier::Percent => "%",
                            RestoreModifier::Flat => "",
                        }}
                    </span> {restore_type_str(Some(restore_type))}{trigger_modifier_str}
                </EffectLi>
            }
            .into_any()
        }
        SkillEffectType::Resurrect => {
            view! { <EffectLi>{success_chance}"Resurrect"</EffectLi> }.into_any()
        }
        SkillEffectType::RefreshCooldown {
            skill_filter,
            value,
            modifier,
        } => {
            let value_str = (matches!(modifier, RestoreModifier::Percent)
                && *value.max == 100.0
                && *value.min == 100.0)
                .then(|| match modifier {
                    RestoreModifier::Percent => view! {
                        {format_min_max(value)}
                        "% of "
                    }
                    .into_any(),
                    RestoreModifier::Flat => view! {
                        {format_duration(value)}
                        "s from "
                    }
                    .into_any(),
                });

            let skill_filter_str = skill_filter_str(&skill_filter, " ", true);
            let skill_filter_str = if skill_filter_str.is_empty() {
                "All Skills".to_string()
            } else {
                skill_filter_str
            };
            view! { <EffectLi>{success_chance}"Refresh " {value_str} {skill_filter_str} " Cooldown"</EffectLi> }
            .into_any()
        }
    };

    let formatted_modifiers = modifiers.map(format_extra_trigger_modifiers);

    let mut conditional_modifiers = skill_effect.conditional_modifiers.clone();
    conditional_modifiers.sort_by_key(|conditional_modifier| {
        (
            conditional_modifier.conditions.first().cloned(),
            conditional_modifier
                .effects
                .first()
                .map(|effect| effect.stat.clone()),
        )
    });
    let conditional_modifiers = conditional_modifiers
        .into_iter()
        .map(format_conditional_modifier)
        .collect::<Vec<_>>();

    // let ignore_stat_effects: Vec<_> = effect
    //     .ignore_stat_effects
    //     .clone()
    //     .into_iter()
    //     .map(format_ignored_stat)
    //     .collect();

    view! {
        {base_effects}
        {conditional_modifiers}
        {formatted_modifiers}
    }
}

fn damage_color(damage_type: DamageType) -> &'static str {
    match damage_type {
        DamageType::Physical => "text-white",
        DamageType::Fire => "text-red-400",
        DamageType::Poison => "text-lime-400",
        DamageType::Storm => "text-amber-400",
    }
}

fn format_min_max_f64<T>(min: T, max: T) -> String
where
    T: Into<f64> + PartialEq + Copy,
{
    let min = min.into();
    let max = max.into();
    if min != max {
        format!("{} - {}", format_number(min), format_number(max))
    } else if min != 0.0 {
        format_number(min).to_string()
    } else {
        "".to_string()
    }
}

fn format_min_max<T>(value: ChanceRange<ModifiableValue<T>>) -> String
where
    T: std::ops::Add<Output = T> + BaseModifiableValue + Default + Copy,
    T: Into<f64> + PartialEq,
{
    format_min_max_f64(*value.min, *value.max)
}

fn format_duration<T>(value: ChanceRange<ModifiableValue<T>>) -> impl IntoView
where
    T: std::ops::Add<Output = T> + BaseModifiableValue + Default + Copy,
    T: Into<f64> + PartialEq,
{
    let min = (*value.min).into();
    let max = (*value.max).into();

    format_duration_values(min, max)
}

fn format_duration_values(min: f64, max: f64) -> impl IntoView {
    let format_min_max = |min, max| {
        if min != max {
            format!("{:.1} - {:.1}", min, max,)
        } else {
            format!("{:.1}", min)
        }
    };

    if min > 9999.0f64 {
        view! { "forever" }.into_any()
    } else if min >= 60.0f64 {
        view! {
            "for "
            <span class="font-semibold">{format_min_max(min / 60.0, max / 60.0)}</span>
            " minutes"
        }
        .into_any()
    } else if min > 0.0 {
        view! {
            "for "
            <span class="font-semibold">{format_min_max(min, max)}</span>
            " seconds"
        }
        .into_any()
    } else {
        view! { "for " }.into_any()
    }
}

#[component]
pub fn EffectLi(
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <li class=format!(
            "text-xs xl:text-sm text-violet-200 whitespace-pre-line {}",
            class.unwrap_or(""),
        )>{children()}</li>
    }
}

pub fn format_skill_modifier(skill_modifier: ModifierEffect) -> impl IntoView {
    let source_description = match skill_modifier.source {
        ModifierEffectSource::ItemStats {
            slot,
            category,
            item_stats,
        } => {
            format!(
                "Per {}{} on equipped {}{}:",
                format_number(1.0 / skill_modifier.factor),
                match item_stats {
                    ItemStatsSource::Armor => " Armor".to_string(),
                    ItemStatsSource::Block => "% Block".to_string(),
                    ItemStatsSource::Cooldown => "s Cooldown".to_string(),
                    ItemStatsSource::CritChance => "% Critical Hit Chance".to_string(),
                    ItemStatsSource::CritDamage => "% Critical Hit Damage".to_string(),
                    ItemStatsSource::Damage {
                        damage_type,
                        min_max,
                    } => format!(
                        " {}{}Damage",
                        min_max_str(min_max),
                        damage_type_str(damage_type)
                    ),
                    ItemStatsSource::Range => " Range".into(),
                    ItemStatsSource::Shape => " Shape".into(),
                },
                match category {
                    Some(category) => item_tooltip::item_category_str(category),
                    None => "",
                },
                match slot {
                    Some(slot) => match slot {
                        ItemSlot::Amulet => "Amulet",
                        ItemSlot::Body => "Body Armor",
                        ItemSlot::Boots => "Boots",
                        ItemSlot::Gloves => "Gloves",
                        ItemSlot::Helmet => "Helmet",
                        ItemSlot::Ring => "Ring",
                        ItemSlot::Shield => "Shield",
                        ItemSlot::Accessory => "Cloak",
                        ItemSlot::Weapon => "Weapon",
                    },
                    None => "Item",
                }
            )
        }
        ModifierEffectSource::CharacterStats(stat_converter) => format!(
            "Per {} {}:",
            format_number(1.0 / skill_modifier.factor),
            effects_tooltip::stat_converter_source_str(stat_converter),
        ),
    };
    let effects = formatted_effects_list(skill_modifier.effects);

    view! {
        <EffectLi>{source_description}</EffectLi>
        {effects}
    }
}

fn format_conditional_modifier(conditional_modifier: ConditionalModifier) -> impl IntoView {
    conditional_modifier
        .effects
        .into_iter()
        .map(move |effect| {
            let conditions = conditional_modifier.conditions.clone();
            view! {
                <EffectLi>
                    {effects_tooltip::format_stat(
                        &StatEffect {
                            stat: StatType::SkillConditionalModifier {
                                stat: Box::new(effect.stat),
                                skill_filter: Default::default(),
                                conditions,
                            },
                            modifier: effect.modifier,
                            value: effect.value,
                            bypass_ignore: effect.bypass_ignore,
                        },
                    )}
                </EffectLi>
            }

            // if
            // effects_tooltip::effect_li(format!(
            //     "{} against {}Enemies{}",
            //     effects_tooltip::format_flat_stat(&effect.stat, Some(effect.value)),
            //     // skill_filter_str(skill_filter, " with ", true),
            //     conditions_tooltip::format_skill_modifier_conditions_pre(
            //         &conditional_modifier.conditions,
            //         ""
            //     ),
            //     conditions_tooltip::format_skill_modifier_conditions_post(
            //         &conditional_modifier.conditions
            //     )
            // ))
        })
        .collect::<Vec<_>>()
}

pub fn skill_effect_text(
    effect: SkillEffect,
    modifiers: Option<&[TriggerEffectModifier]>,
) -> String {
    let _ = modifiers;
    let stat_skill_effect: Option<StatSkillEffectType> = (&effect.effect_type).into();
    match effect.effect_type {
        SkillEffectType::FlatDamage { damage, .. } => {
            format!(
                "Hit {}Damage",
                damage
                    .keys()
                    .map(|damage_type| damage_type_str(Some(*damage_type)))
                    .join(" ")
            )
        }
        SkillEffectType::ApplyStatus { status_id, .. } => {
            let data_context: DataContext = expect_context();
            let status_name = data_context.status_name(&status_id);
            // let status_specs = data_context
            //     .statuses_specs
            //     .read_untracked()
            //     .get(&status_id)
            //     .cloned();

            // TODO: status_str

            format!("Apply {status_name}")
        }
        SkillEffectType::Resurrect
        | SkillEffectType::Restore { .. }
        | SkillEffectType::RefreshCooldown { .. } => {
            stat_skill_effect_type_str(stat_skill_effect.as_ref())
        }
    }
}

fn format_ignored_stat(stat_type: StatType) -> impl IntoView {
    view! {
        <EffectLi class="italic">
            "Unaffected by "{format_multiplier_stat_name(&stat_type)}" Modifiers"
        </EffectLi>
    }
}
