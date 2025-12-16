use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::data::{
    chance::ChanceRange,
    character_status::StatusSpecs,
    item::{ItemSlot, SkillRange, SkillShape},
    passive::StatEffect,
    skill::{
        DamageType, ItemStatsSource, ModifierEffect, ModifierEffectSource, RestoreType,
        SkillEffect, SkillEffectType, SkillRepeatTarget, SkillSpecs, SkillTargetsGroup, SkillType,
        TargetType,
    },
    stat_effect::Modifier,
    temple::StatType,
    trigger::TriggerEffectModifier,
};

use crate::components::{
    shared::tooltips::{
        effects_tooltip::{self, formatted_effects_list},
        trigger_tooltip::{
            format_extra_trigger_modifiers, format_trigger, format_trigger_modifier_as,
            format_trigger_modifier_per,
        },
    },
    ui::number::format_number,
};

use super::effects_tooltip::damage_type_str;

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
                        <span class="text-white">{format!("{:.1}s", skill_specs.cooldown)}</span>
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

            <ul class="list-none space-y-1 text-xs xl:text-sm">
                {targets_lines}{trigger_lines}{modifier_lines}
            </ul>

            {(skill_specs.next_upgrade_cost > 0.0)
                .then(|| {
                    view! {
                        <hr class="border-t border-gray-700" />
                        <ul class="text-xs xl:text-sm ">
                            <li>
                                <span class="text-gray-400 leading-snug">"Next upgrade:"</span>
                            </li>
                            {effects_tooltip::formatted_effects_list(
                                skill_specs.base.upgrade_effects.clone(),
                            )}
                        </ul>

                        <hr class="border-t border-gray-700" />
                        <p class="text-xs xl:text-sm text-gray-400 leading-snug">
                            "Level: "
                            {if skill_specs.level_modifier > 0 {
                                view! {
                                    <span class="text-blue-400">
                                        {skill_specs
                                            .upgrade_level
                                            .saturating_add(skill_specs.level_modifier)}
                                    </span>
                                }
                            } else {
                                view! {
                                    <span class="text-white">{skill_specs.upgrade_level}</span>
                                }
                            }} " | Upgrade Cost: "
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
        SkillShape::Contact => ", Contact",
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

    let repeat = if targets_group.repeat.value.max > 1 {
        format!(
            ", {} {}",
            match targets_group.repeat.target {
                SkillRepeatTarget::Any => "Repeat",
                SkillRepeatTarget::Same => "Multi-Hit",
                SkillRepeatTarget::Different => "Chain",
            },
            format_min_max(targets_group.repeat.value),
        )
    } else {
        "".into()
    };

    let effects = targets_group
        .effects
        .into_iter()
        .map(|x| format_effect(x, None))
        .collect::<Vec<_>>();

    view! {
        <hr class="border-t border-gray-700" />
        <EffectLi>{range}{shape}{repeat}</EffectLi>
        {effects}
    }
}

// fn format_value<'a>(
//     value: ChanceRange<f64>,
//     stat: StatType,
//     modifiers: Option<&'a [TriggerEffectModifier]>,
// ) -> String {
//     if let Some(modifier) = modifiers
//         .unwrap_or_default()
//         .iter()
//         .find(|modifier| modifier.stat.is_match(&stat))
//     {
//         let factor_str = match modifier.modifier {
//             Modifier::Multiplier => format!("{:.0}", modifier.factor),
//             Modifier::Flat => format!(
//                 "{}+{:.0}",
//                 format_min_max(value)
//                 format_min_max(ChanceRange {
//                     min:  modifier.factor,
//                     max:  modifier.factor,
//                     lucky_chance: 0.0
//                 })
//             ),
//         };
//         format!(
//             "{}% {}",
//             factor_str,
//             format_trigger_modifier(modifier.source),
//         )
//     } else {
//         format_min_max(value)
//     }
// }

// fn format_ratio_per<'a>(
//     value: ChanceRange<f64>,
//     stat: StatType,
//     modifiers: Option<&'a [TriggerEffectModifier]>,
// ) -> Option<String> {
//     modifiers
//         .unwrap_or_default()
//         .iter()
//         .find(|modifier| modifier.stat.is_match(&stat))
//         .map(|modifier| {
//             let factor_str = match modifier.modifier {
//                 Modifier::Multiplier => format!("{:.0}% of", modifier.factor),
//                 Modifier::Flat => format!(
//                     "{:.1} per",
//                     format_min_max(ChanceRange {
//                         min: value.min + modifier.factor,
//                         max: value.max + modifier.factor,
//                         lucky_chance: value.lucky_chance
//                     })
//                 ),
//             };
//             format!(
//                 "{} {}",
//                 factor_str,
//                 format_trigger_modifier(modifier.source),
//             )
//         })
// }

fn find_trigger_modifier<'a>(
    stat: StatType,
    modifiers: Option<&'a [TriggerEffectModifier]>,
) -> Option<&'a TriggerEffectModifier> {
    modifiers
        .unwrap_or_default()
        .iter()
        .find(|modifier| modifier.stat.is_match(&stat) && modifier.modifier == Modifier::Flat)
}

pub fn format_effect<'a>(
    effect: SkillEffect,
    modifiers: Option<&'a [TriggerEffectModifier]>,
) -> impl IntoView + use<> {
    let success_chance = if effect.success_chance.value < 100.0 {
        Some(view! {
            <span class="font-semibold">{format!("{:.0}%", effect.success_chance.value)}</span>
            " chance to "
        })
    } else {
        None
    };

    let base_effects = match effect.effect_type {
        SkillEffectType::FlatDamage {
            damage,
            crit_chance,
            crit_damage,
            ..
        } => view! {
            {damage
                .into_iter()
                .map(|(damage_type, value)| {
                    let success_chance = success_chance.clone();
                    let damage_color = damage_color(damage_type);
                    let trigger_modifier_str = format_trigger_modifier_as(
                        find_trigger_modifier(
                            StatType::Damage {
                                damage_type: Some(damage_type),
                                skill_type: None,
                            },
                            modifiers.clone(),
                        ),
                    );
                    view! {
                        <EffectLi>
                            {success_chance}"Deal "
                            <span class=format!(
                                "font-semibold {damage_color}",
                            )>{format_min_max(value)}</span>{trigger_modifier_str} " "
                            {damage_type_str(Some(damage_type))} "Damage"
                        </EffectLi>
                    }
                })
                .collect::<Vec<_>>()}
            {if crit_chance.value > 0.0 {
                Some(
                    view! {
                        <EffectLi>
                            "Critical hit chance: "
                            <span class="font-semibold">
                                {format!("{:.2}%", crit_chance.value)}
                            </span>
                        </EffectLi>
                        <EffectLi>
                            "Critical hit damage: "
                            <span class="font-semibold">{format!("+{:.0}%", crit_damage)}</span>
                        </EffectLi>
                    },
                )
            } else {
                None
            }}
        }
        .into_any(),
        SkillEffectType::ApplyStatus { statuses, duration } => {
            let mut stat_effects = Vec::new();
            let mut trigger_effects = Vec::new();
            let mut max_stat_effects = Vec::new();

            let formatted_status_effects: Vec<_> = statuses
                .iter()
                .cloned()
                .map(|status_effect| match status_effect.status_type {
                    StatusSpecs::Stun => {
                        let success_chance = success_chance.clone();
                        view! { <EffectLi>{success_chance}"Stun " {format_duration(duration)}</EffectLi> }
                        .into_any()
                    }
                    StatusSpecs::DamageOverTime { damage_type, .. } => {
                        let success_chance = success_chance.clone();
                        let damage_color = damage_color(damage_type);
                        let trigger_modifier_str = format_trigger_modifier_as(
                                    find_trigger_modifier(
                                        StatType::Damage {
                                            damage_type: Some(damage_type),
                                            skill_type: None,
                                        },
                                        modifiers.clone(),
                                    ),
                                );
                        view! {
                            <EffectLi>
                                {success_chance}"Deal "
                                <span class=format!(
                                    "font-semibold {damage_color}",
                                )>{format_min_max(status_effect.value)}</span>
                                {trigger_modifier_str}"  " {damage_type_str(Some(damage_type))}
                                "Damage per second " {format_duration(duration)}
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
                                stat: stat.clone(),
                            modifier,
                            value: if debuff {
                                -status_effect.value.min
                            } else {
                                status_effect.value.min
                            },
                            bypass_ignore: false,
                        });
                        if status_effect.value.min != status_effect.value.max {
                            max_stat_effects.push(StatEffect {
                                stat: stat.clone(),
                                modifier,
                                value: if debuff {
                                    -status_effect.value.max
                                } else {
                                    status_effect.value.max
                                },
                                bypass_ignore: false,
                            });
                        }
                        ().into_any()
                    }
                    StatusSpecs::Trigger(trigger_specs) => {
                        trigger_effects.push(view! { <ul>{format_trigger(*trigger_specs)}</ul> });
                        ().into_any()
                    }
                })
                .collect();

            let formatted_stats_effects = {
                (!stat_effects.is_empty() || !trigger_effects.is_empty()).then(|| {
                    view! {
                        <EffectLi>
                            {success_chance}"Apply the following status "
                            {format_duration(duration)} ":"
                            {(!stat_effects.is_empty())
                                .then(|| {
                                    view! {
                                        <ul>
                                            {effects_tooltip::formatted_effects_list(stat_effects)}
                                        </ul>
                                    }
                                        .into_any()
                                })}
                            {(!max_stat_effects.is_empty())
                                .then(|| {
                                    view! {
                                        "to"
                                        <ul>
                                            {effects_tooltip::formatted_effects_list(max_stat_effects)}
                                        </ul>
                                    }
                                        .into_any()
                                })} {trigger_effects}
                        </EffectLi>
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
            value,
            modifier,
        } => {
            let trigger_modifier =
                find_trigger_modifier(StatType::Restore(Some(restore_type)), modifiers.clone());
            let trigger_modifier_str = format_trigger_modifier_per(trigger_modifier.clone());
            let trigger_modifier_factor_str =
                trigger_modifier.map(|trigger_modifier| format!("{:.0}", trigger_modifier.factor));
            view! {
                <EffectLi>
                    {success_chance}"Restore "
                    <span class="font-semibold">
                        {format_min_max(value)} {trigger_modifier_factor_str}
                        {match modifier {
                            Modifier::Multiplier => "%",
                            Modifier::Flat => "",
                        }}
                    </span> {restore_type_str(Some(restore_type))}{trigger_modifier_str}
                </EffectLi>
            }
            .into_any()
        }
        SkillEffectType::Resurrect => {
            view! { <EffectLi>{success_chance}"Resurrect"</EffectLi> }.into_any()
        }
    };

    let formatted_modifiers = modifiers.map(|modifiers| format_extra_trigger_modifiers(modifiers));

    view! {
        {base_effects}
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

fn format_min_max<T>(value: ChanceRange<T>) -> String
where
    T: Into<f64> + PartialEq + Copy,
{
    if value.min != value.max {
        format!(
            "{} - {}",
            format_number(value.min.into()),
            format_number(value.max.into())
        )
    } else if value.min.into() != 0.0 {
        format_number(value.min.into()).to_string()
    } else {
        "".to_string()
    }
}

fn format_duration<T>(value: ChanceRange<T>) -> impl IntoView
where
    T: Into<f64> + PartialEq + Copy,
{
    let format_min_max = |value: ChanceRange<f64>| {
        if value.min != value.max {
            format!("{:.1} - {:.1}", value.min, value.max,)
        } else {
            format!("{:.1}", value.min)
        }
    };

    if value.min.into() > 9999.0f64 {
        view! { "forever" }.into_any()
    } else if value.min.into() >= 60.0f64 {
        let value = ChanceRange::<f64> {
            min: value.min.into() / 60.0,
            max: value.max.into() / 60.0,
            lucky_chance: value.lucky_chance,
        };
        view! {
            "for "
            <span class="font-semibold">{format_min_max(value)}</span>
            " minutes"
        }
        .into_any()
    } else {
        let value = ChanceRange::<f64> {
            min: value.min.into(),
            max: value.max.into(),
            lucky_chance: value.lucky_chance,
        };
        view! {
            "for "
            <span class="font-semibold">{format_min_max(value)}</span>
            " seconds"
        }
        .into_any()
    }
}

#[component]
pub fn EffectLi(children: Children) -> impl IntoView {
    view! {
        <li class="text-xs xl:text-sm text-violet-200 leading-snug whitespace-pre-line">
            {children()}
        </li>
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
                        format!("average {}Damage", damage_type_str(damage_type)),
                    ItemStatsSource::MinDamage(damage_type) =>
                        format!("Minimum {}Damage", damage_type_str(damage_type)),
                    ItemStatsSource::MaxDamage(damage_type) =>
                        format!("Maximum {}Damage", damage_type_str(damage_type)),
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
        ModifierEffectSource::PlaceHolder => todo!(),
    };
    let effects = formatted_effects_list(skill_modifier.effects);

    view! {
        <EffectLi>{source_description}</EffectLi>
        {effects}
    }
}
