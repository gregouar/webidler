use std::sync::Arc;

use leptos::html::*;
use leptos::prelude::*;

use shared::data::character_status::StatusType;
use shared::data::skill::SkillTargetsGroup;
use shared::data::skill::TargetType;
use shared::data::skill::{SkillEffect, SkillEffectType, SkillRange, SkillShape, SkillSpecs};

use crate::components::{game::effects_tooltip::damage_type_str, ui::number::format_number};

#[component]
pub fn SkillTooltip(skill_specs: Arc<SkillSpecs>) -> impl IntoView {
    let targets_lines = skill_specs
        .targets
        .clone()
        .into_iter()
        .map(format_target)
        .collect::<Vec<_>>();

    view! {
        <div class="
        max-w-xs p-4 rounded-xl border border-purple-700 ring-2 ring-purple-500 
        shadow-md shadow-purple-700 bg-gradient-to-br from-gray-800 via-gray-900 to-black space-y-2
        ">
            <strong class="text-lg font-bold text-purple-300">
                {skill_specs.base.name.clone()}
            </strong>
            <hr class="border-t border-gray-700" />

            <p class="text-sm text-gray-400 leading-snug">
                "Cooldown: "
                <span class="text-white">{format!("{:.2}s", skill_specs.cooldown)}</span>
                {(skill_specs.mana_cost > 0.0)
                    .then(|| {
                        view! {
                            " | Mana Cost: "
                            <span class="text-white">{skill_specs.mana_cost}</span>
                        }
                    })}
            </p>

            <ul class="list-none space-y-1">{targets_lines}</ul>

            {(!skill_specs.base.description.is_empty())
                .then(|| {
                    view! {
                        <hr class="border-t border-gray-700" />
                        <p class="text-sm italic text-gray-300 leading-snug">
                            {skill_specs.base.description.clone()}
                        </p>
                    }
                })}

            {(skill_specs.next_upgrade_cost > 0.0)
                .then(|| {
                    view! {
                        <hr class="border-t border-gray-700" />
                        <p class="text-sm text-gray-400 leading-snug">
                            "Level: " <span class="text-white">{skill_specs.upgrade_level}</span>
                            " | Upgrade Cost: "
                            <span class="text-white">
                                {format_number(skill_specs.next_upgrade_cost)}" Gold"
                            </span>
                        </p>
                    }
                })}
        </div>
    }
}

fn format_target(targets_group: SkillTargetsGroup) -> impl IntoView {
    let shape = match targets_group.shape {
        SkillShape::Single => "",
        SkillShape::Vertical2 => ", 2x1 area",
        SkillShape::Horizontal2 => ", 1x2 area",
        SkillShape::Horizontal3 => ", 1x3 area",
        SkillShape::Square4 => ", 2x2 area",
        SkillShape::All => ", all",
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

    let effects = targets_group
        .effects
        .into_iter()
        .map(format_effect)
        .collect::<Vec<_>>();

    view! {
        <hr class="border-t border-gray-700" />
        <EffectLi>{range}{shape}</EffectLi>
        {effects}
    }
}

fn format_effect(effect: SkillEffect) -> impl IntoView {
    match effect.effect_type {
            SkillEffectType::FlatDamage { damage, crit_chances, crit_damage } => {
                view! {
                    {damage
                        .into_iter()
                        .map(|(damage_type, (min, max))| {
                            view! {
                                <EffectLi>
                                    "Deals "
                                    <span class="font-semibold">{format_min_max(min, max)}</span>
                                    " " {damage_type_str(damage_type)} " Damage"
                                </EffectLi>
                            }
                        })
                        .collect::<Vec<_>>()}
                    {if crit_chances > 0.0 {
                        Some(
                            view! {
                                <EffectLi>
                                    "Critical chances: "
                                    <span class="font-semibold">
                                        {format!("{:.2}%", crit_chances * 100.0)}
                                    </span>
                                </EffectLi>
                                <EffectLi>
                                    "Critical damage: "
                                    <span class="font-semibold">
                                        {format!("+{:.0}%", crit_damage * 100.0)}
                                    </span>
                                </EffectLi>
                            },
                        )
                    } else {
                        None
                    }}
                }
                    .into_any()
            }
            SkillEffectType::Heal { min, max } => {
                view! { <EffectLi>"Heals "<span class="font-semibold">{format_min_max(min, max)}</span></EffectLi> }
                    .into_any()
            }
            SkillEffectType::ApplyStatus {
                status_type,
                min_value,
                max_value,
                min_duration,
                max_duration,
            } => {
                match status_type {
                    StatusType::Stun => {
                        view! { <EffectLi>"Stun for "{format_min_max(min_duration, max_duration)}" seconds"</EffectLi> }
                            .into_any()
                    }
                    StatusType::DamageOverTime { damage_type, .. } => {
                        view! {
                            <EffectLi>
                                "Deals "
                                <span class="font-semibold">
                                    {format_min_max(min_value, max_value)}
                                </span>"  "{damage_type_str(damage_type)}" Damage per second over "
                                {format_min_max(min_duration, max_duration)}" seconds"
                            </EffectLi>
                        }
                            .into_any()
                    }
                }
            }
        }
}

fn format_min_max(min: f64, max: f64) -> String {
    if min != max {
        format!("{}-{}", format_number(min), format_number(max))
    } else {
        format_number(min).to_string()
    }
}

#[component]
fn EffectLi(children: Children) -> impl IntoView {
    view! { <li class="text-sm text-purple-200 leading-snug">{children()}</li> }
}
