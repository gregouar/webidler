use std::sync::Arc;

use leptos::html::*;
use leptos::prelude::*;

use shared::data::skill::{DamageType, Range, Shape, SkillEffect, SkillEffectType, SkillSpecs};

use crate::components::ui::number::format_number;

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
            <strong class="text-lg font-bold text-purple-300">{skill_specs.name.clone()}</strong>
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

            <hr class="border-t border-gray-700" />

            <ul class="list-none space-y-1">{effect_lines}</ul>

            {(!skill_specs.description.is_empty())
                .then(|| {
                    view! {
                        <hr class="border-t border-gray-700" />
                        <p class="text-sm italic text-gray-300 leading-snug">
                            {skill_specs.description.clone()}
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
        Range::Melee => "melee",
        Range::Distance => "distance",
    };

    match &effect.effect_type {
        SkillEffectType::FlatDamage {
            min,
            max,
            damage_type,
            crit_chances,
            crit_damage,
        } => {
            let dmg_type = match damage_type {
                DamageType::Physical => "Physical",
                DamageType::Fire => "Fire",
            };
            view! {
                <li class="text-sm text-purple-200 leading-snug">
                    {format!(
                        "Deals {} {} Damage ({}{})",
                        format_min_max(*min, *max),
                        dmg_type,
                        range,
                        shape,
                    )}
                </li>
                <li class="text-gray-400 text-sm leading-snug">
                    "Critical chances: "
                    <span class="font-semibold">{format!("{:.2}%", crit_chances)}</span>
                </li>
                <li class="text-gray-400 text-sm leading-snug">
                    "Critical damage: "
                    <span class="font-semibold">{format!("+{:.0}%", crit_damage * 100.0)}</span>
                </li>
            }
            .into_any()
        }
        SkillEffectType::Heal { min, max } => view! {
            <li class="text-sm text-purple-200 leading-snug">
                {format!("Heals {}", format_min_max(*min, *max))}
            </li>
        }
        .into_any(),
    };
}

fn format_min_max(min: f64, max: f64) -> String {
    if min != max {
        format!("{}-{}", format_number(min), format_number(max))
    } else {
        format!("{}", format_number(min))
    }
}
