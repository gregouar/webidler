use std::sync::Arc;

use leptos::html::*;
use leptos::prelude::*;

use shared::data::skill::{Range, Shape, SkillEffect, SkillEffectType, SkillSpecs};

use crate::components::{game::effects_tooltip::damage_type_str, ui::number::format_number};

#[component]
pub fn SkillTooltip(skill_specs: Arc<SkillSpecs>) -> impl IntoView {
    let effect_lines = skill_specs
        .effects
        .clone()
        .into_iter()
        .map(format_effect)
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

            <ul class="list-none space-y-1">{effect_lines}</ul>

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

fn format_effect(effect: SkillEffect) -> impl IntoView {
    let shape = match effect.shape {
        Shape::Single => "",
        Shape::Vertical2 => ", 2x1 area",
        Shape::Horizontal2 => ", 1x2 area",
        Shape::Horizontal3 => ", 1x3 area",
        Shape::Square4 => ", 2x2 area",
        Shape::All => ", all",
    };

    let range = match effect.range {
        Range::Melee => "Melee",
        Range::Distance => "Distance",
        Range::Any => "Any",
    };

    view! {
        <hr class="border-t border-gray-700" />
        <EffectLi>{range}{shape}</EffectLi>
        {match effect.effect_type {
            SkillEffectType::FlatDamage { damage, crit_chances, crit_damage } => {
                view! {
                    {damage
                        .into_iter()
                        .map(|(damage_type, (min, max))| {
                            view! {
                                <EffectLi>
                                    "Deals "
                                    <span class="font-semibold">{format_min_max(min, max)}</span>
                                    " " {damage_type_str(damage_type)}
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
                view! {
                    <EffectLi>
                        "Heals "<span class="font-semibold">{format_min_max(min, max)}</span>
                    </EffectLi>
                }
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
                    shared::data::status::StatusType::Stunned => {
                        view! {
                            <EffectLi>
                                "Stun for "{format_min_max(min_duration, max_duration)}" seconds"
                            </EffectLi>
                        }
                            .into_any()
                    }
                    shared::data::status::StatusType::DamageOverTime(damage_type) => {
                        view! {
                            <EffectLi>
                                "Deals "
                                <span class="font-semibold">
                                    {format_min_max(min_value, max_value)}
                                </span> " per second " {damage_type_str(damage_type)}" over "
                                {format_min_max(min_duration, max_duration)}" seconds"
                            </EffectLi>
                        }
                            .into_any()
                    }
                }
            }
        }}
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
