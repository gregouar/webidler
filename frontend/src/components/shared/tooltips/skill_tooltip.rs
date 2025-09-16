use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::data::{
    character_status::StatusSpecs,
    item::{ItemSlot, SkillRange, SkillShape},
    item_affix::AffixEffectScope,
    passive::StatEffect,
    skill::{
        DamageType, ItemStatsSource, ModifierEffect, ModifierEffectSource, RestoreType,
        SkillEffect, SkillEffectType, SkillRepeatTarget, SkillSpecs, SkillTargetsGroup, SkillType,
        TargetType,
    },
    trigger::TriggerSpecs,
};

use crate::components::{
    shared::tooltips::effects_tooltip::{self, formatted_effects_list},
    ui::number::format_number,
};

use super::effects_tooltip::optional_damage_type_str;

pub fn skill_type_str(skill_type: Option<SkillType>) -> &'static str {
    match skill_type {
        Some(SkillType::Attack) => "Attack ",
        Some(SkillType::Spell) => "Spell ",
        None => "",
    }
}

pub fn restore_type_str(restore_tyoe: Option<RestoreType>) -> &'static str {
    match restore_tyoe {
        Some(RestoreType::Life) => " Life",
        Some(RestoreType::Mana) => " Mana",
        None => "",
    }
}

#[component]
pub fn SkillTooltip(skill_specs: Arc<SkillSpecs>) -> impl IntoView {
    let targets_lines = skill_specs
        .targets
        .clone()
        .into_iter()
        .map(format_target)
        .collect::<Vec<_>>();

    let trigger_lines = skill_specs
        .triggers
        .clone()
        .into_iter()
        .map(format_trigger)
        .collect::<Vec<_>>();

    let modifier_lines: Vec<_> = skill_specs
        .base
        .modifier_effects
        .clone()
        .into_iter()
        .map(format_skill_modifier)
        .collect();

    view! {
        <div class="
        max-w-xs p-4 rounded-xl border border-violet-700 ring-2 ring-violet-500 
        shadow-md shadow-violet-700 bg-gradient-to-br from-gray-800 via-gray-900 to-black space-y-2
        ">
            <strong class="text-base xl:text-lg font-bold text-violet-300">
                {skill_specs.base.name.clone()}
            </strong>
            <hr class="border-t border-gray-700" />

            <p class="text-xs xl:text-sm text-gray-400 leading-snug">
                {skill_type_str(Some(skill_specs.base.skill_type))} "| "
                {if skill_specs.cooldown > 0.0 {
                    view! {
                        "Cooldown: "
                        <span class="text-white">{format!("{:.2}s", skill_specs.cooldown)}</span>
                    }
                        .into_any()
                } else {
                    view! { "Permanent" }.into_any()
                }}
                {(skill_specs.mana_cost > 0.0)
                    .then(|| {
                        view! {
                            " | Mana Cost: "
                            <span class="text-white">{skill_specs.mana_cost}</span>
                        }
                    })}
            </p>

            <ul class="list-none space-y-1">{targets_lines}{trigger_lines}{modifier_lines}</ul>

            {(skill_specs.next_upgrade_cost > 0.0)
                .then(|| {
                    view! {
                        <hr class="border-t border-gray-700" />
                        <ul>
                            <li>
                                <span class="text-xs xl:text-sm text-gray-400 leading-snug">
                                    "Next upgrade:"
                                </span>
                            </li>
                            {effects_tooltip::formatted_effects_list(
                                skill_specs.base.upgrade_effects.clone(),
                                AffixEffectScope::Local,
                            )}
                        </ul>

                        <hr class="border-t border-gray-700" />
                        <p class="text-xs xl:text-sm text-gray-400 leading-snug">
                            "Level: " <span class="text-white">{skill_specs.upgrade_level}</span>
                            " | Upgrade Cost: "
                            <span class="text-white">
                                {format_number(skill_specs.next_upgrade_cost)}" Gold"
                            </span>
                        </p>
                    }
                })}

            {(!skill_specs.base.description.is_empty())
                .then(|| {
                    view! {
                        <hr class="border-t border-gray-700" />
                        <p class="text-xs xl:text-sm italic text-gray-400 leading-snug">
                            {skill_specs.base.description.clone()}
                        </p>
                    }
                })}
        </div>
    }
}

fn format_target(targets_group: SkillTargetsGroup) -> impl IntoView {
    let shape = match targets_group.shape {
        SkillShape::Single => ", Single",
        SkillShape::Vertical2 => ", Area 2x1",
        SkillShape::Horizontal2 => ", Area 1x2",
        SkillShape::Horizontal3 => ", Area 1x3",
        SkillShape::Square4 => ", Area 2x2",
        SkillShape::All => ", All",
    };

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

    let repeat = if targets_group.repeat.max > 1 {
        format!(
            ", {} {}",
            match targets_group.repeat.target {
                SkillRepeatTarget::Any => "Repeat",
                SkillRepeatTarget::Same => "Multi-Hit",
                SkillRepeatTarget::Different => "Chain",
            },
            format_min_max(
                targets_group.repeat.min as f64,
                targets_group.repeat.max as f64
            ),
        )
    } else {
        "".into()
    };

    let effects = targets_group
        .effects
        .into_iter()
        .map(format_effect)
        .collect::<Vec<_>>();

    view! {
        <hr class="border-t border-gray-700" />
        <EffectLi>{range}{shape}{repeat}</EffectLi>
        {effects}
    }
}

fn format_effect(effect: SkillEffect) -> impl IntoView {
    let success_chances = if effect.failure_chances > 0.0 {
        Some(format!(
            "{:.0}% chance to ",
            (1.0 - effect.failure_chances) * 100.0
        ))
    } else {
        None
    };

    match effect.effect_type {
        SkillEffectType::FlatDamage {
            damage,
            crit_chances,
            crit_damage,
        } => view! {
            {damage
                .into_iter()
                .map(|(damage_type, (min, max))| {
                    let success_chances = success_chances.clone();
                    let damage_color = damage_color(damage_type);

                    view! {
                        <EffectLi>
                            {success_chances}"Deal "
                            <span class=format!(
                                "font-semibold {damage_color}",
                            )>{format_min_max(min, max)}</span> " "
                            {optional_damage_type_str(Some(damage_type))} "Damage"
                        </EffectLi>
                    }
                })
                .collect::<Vec<_>>()}
            {if crit_chances > 0.0 {
                Some(
                    view! {
                        <EffectLi>
                            "Critical chance: "
                            <span class="font-semibold">{format!("{:.2}%", crit_chances)}</span>
                        </EffectLi>
                        <EffectLi>
                            "Critical damage: "
                            <span class="font-semibold">{format!("+{:.0}%", crit_damage)}</span>
                        </EffectLi>
                    },
                )
            } else {
                None
            }}
        }
        .into_any(),
        SkillEffectType::ApplyStatus {
            statuses,
            min_duration,
            max_duration,
        } => {
            let mut stat_effects = Vec::new();
            let mut max_stat_effects = Vec::new();

            let formatted_status_effects: Vec<_> = statuses
                .iter()
                .cloned()
                .map(|status_effect| match status_effect.status_type {
                    StatusSpecs::Stun => {
                        let success_chances = success_chances.clone();
                        view! {
                            <EffectLi>
                                {success_chances}"Stun for "
                                {format_min_max(min_duration, max_duration)}" seconds"
                            </EffectLi>
                        }
                        .into_any()
                    }
                    StatusSpecs::DamageOverTime { damage_type, .. } => {
                        let success_chances = success_chances.clone();
                        let damage_color = damage_color(damage_type);
                        view! {
                            <EffectLi>
                                {success_chances}"Deal "
                                <span class=format!(
                                    "font-semibold {damage_color}",
                                )>
                                    {format_min_max(
                                        status_effect.min_value,
                                        status_effect.max_value,
                                    )}
                                </span>"  "{optional_damage_type_str(Some(damage_type))}
                                "Damage per second for "
                                {format_min_max(min_duration, max_duration)} " seconds"
                            </EffectLi>
                        }
                        .into_any()
                    }
                    StatusSpecs::StatModifier {
                        stat,
                        modifier,
                        debuff,
                    } => {
                        stat_effects.push(StatEffect {
                            stat,
                            modifier,
                            value: if debuff {
                                -status_effect.min_value
                            } else {
                                status_effect.min_value
                            },
                        });
                        if status_effect.min_value != status_effect.max_value {
                            max_stat_effects.push(StatEffect {
                                stat,
                                modifier,
                                value: if debuff {
                                    -status_effect.min_value
                                } else {
                                    status_effect.min_value
                                },
                            });
                        }
                        ().into_any()
                    }
                    StatusSpecs::Trigger(trigger_specs) => {
                        let success_chances = success_chances.clone();
                        view! {
                            <EffectLi>
                                {success_chances}"Apply the following status for "
                                {format_min_max(min_duration, max_duration)} " seconds:"
                                <ul>{format_trigger(*trigger_specs)}</ul>
                            </EffectLi>
                        }
                        .into_any()
                    }
                })
                .collect();

            let formatted_stats_effects = {
                (!stat_effects.is_empty()).then(|| {
                    view! {
                        <EffectLi>
                            {success_chances}"Apply the following status for "
                            {format_min_max(min_duration, max_duration)} " seconds:"
                            <ul>
                                {effects_tooltip::formatted_effects_list(
                                    stat_effects,
                                    AffixEffectScope::Global,
                                )}
                            </ul>
                        </EffectLi>
                        {(!max_stat_effects.is_empty())
                            .then(|| {
                                view! {
                                    "to"
                                    <ul>
                                        {effects_tooltip::formatted_effects_list(
                                            max_stat_effects,
                                            AffixEffectScope::Global,
                                        )}
                                    </ul>
                                }
                                    .into_any()
                            })}
                    }
                })
            };

            view! {
                {formatted_status_effects}
                {formatted_stats_effects}
            }
            .into_any()
        }
        SkillEffectType::Restore {
            restore_type,
            min,
            max,
        } => view! {
            <EffectLi>
                {success_chances}"Restore "
                <span class="font-semibold">{format_min_max(min, max)}</span>
                {restore_type_str(Some(restore_type))}
            </EffectLi>
        }
        .into_any(),
        SkillEffectType::Resurrect => {
            view! { <EffectLi>{success_chances}"Resurrect"</EffectLi> }.into_any()
        }
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

fn format_min_max(min: f64, max: f64) -> String {
    if min != max {
        format!("{} - {}", format_number(min), format_number(max))
    } else {
        format_number(min).to_string()
    }
}

#[component]
fn EffectLi(children: Children) -> impl IntoView {
    view! {
        <li class="text-xs xl:text-sm text-violet-200 leading-snug whitespace-pre-line">
            {children()}
        </li>
    }
}

pub fn format_trigger(trigger: TriggerSpecs) -> impl IntoView {
    let effects = if trigger.triggered_effect.modifiers.is_empty() {
        trigger
            .triggered_effect
            .effects
            .into_iter()
            .map(format_effect)
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    view! {
        <EffectLi>{trigger.description}</EffectLi>
        {effects}
    }
}

pub fn format_skill_modifier(skill_modifier: ModifierEffect) -> impl IntoView {
    let source_description = match skill_modifier.source {
        ModifierEffectSource::ItemStats { slot, item_stats } => {
            format!(
                "Per {} {} on equipped {}:",
                format_number(1.0 / skill_modifier.factor),
                match item_stats {
                    ItemStatsSource::Damage(damage_type) =>
                        format!("average {}Damage", optional_damage_type_str(damage_type)),
                    ItemStatsSource::Armor => "Armor".to_string(),
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
                        ItemSlot::Accessory => "Accessory",
                        ItemSlot::Weapon => "Weapon",
                    },
                    None => "Item",
                }
            )
        }
    };
    let effects = formatted_effects_list(skill_modifier.effects, AffixEffectScope::Local);

    view! {
        <EffectLi>{source_description}</EffectLi>
        {effects}
    }
}
