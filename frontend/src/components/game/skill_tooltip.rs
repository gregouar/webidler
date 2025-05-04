use leptos::html::*;
use leptos::prelude::*;

use shared::data::skill::{
    DamageType, Shape, SkillEffect, SkillEffectType, SkillSpecs, TargetType,
};

#[component]
pub fn SkillTooltip(skill: SkillSpecs) -> impl IntoView {
    let effect_lines = skill
        .effects
        .into_iter()
        .map(format_effect)
        .collect::<Vec<_>>();

    view! {
        <div class="
        max-w-xs p-4 rounded-xl border border-purple-700 ring-2 ring-purple-500 
        shadow-md shadow-purple-700 bg-gradient-to-br from-gray-800 via-gray-900 to-black space-y-2
        ">
            <div class="flex items-center space-x-2">
                <img src=skill.icon.clone() alt="Skill icon" class="w-6 h-6" />
                <strong class="text-lg font-bold text-purple-300">{skill.name.clone()}</strong>
            </div>
            <hr class="border-t border-gray-700" />

            <p class="text-sm text-gray-400 leading-snug">
                "Cooldown: " <span class="text-white">{format!("{:.2}s", skill.cooldown)}</span>
                {(skill.mana_cost > 0.0)
                    .then(|| {
                        view! {
                            <>", Mana Cost: " <span class="text-white">{skill.mana_cost}</span></>
                        }
                    })}
            </p>

            {(skill.upgrade_level > 0)
                .then(|| {
                    view! {
                        <p class="text-sm text-gray-400 leading-snug">
                            "Upgrade Level: " <span class="text-white">{skill.upgrade_level}</span>
                            ", Next Upgrade: "
                            <span class="text-white">{skill.next_upgrade_cost}</span>
                        </p>
                    }
                })}

            <hr class="border-t border-gray-700" />

            <ul class="list-none space-y-1">{effect_lines}</ul>

            {(!skill.description.is_empty())
                .then(|| {
                    view! {
                        <>
                            <hr class="border-t border-gray-700" />
                            <p class="text-sm italic text-gray-300 leading-snug">
                                {skill.description.clone()}
                            </p>
                        </>
                    }
                })}
        </div>
    }
}

fn format_effect(effect: SkillEffect) -> impl IntoView {
    let target = match effect.target_type {
        TargetType::Enemy => "Enemy",
        TargetType::Friend => "Ally",
        TargetType::Me => "Self",
    };

    let shape = match effect.shape {
        Shape::Single => "single target",
        Shape::Vertical2 => "vertical 2-tile line",
        Shape::Horizontal2 => "horizontal 2-tile line",
        Shape::Horizontal3 => "horizontal 3-tile line",
        Shape::Square4 => "2x2 area",
        Shape::All => "all targets",
    };

    let desc = match &effect.effect_type {
        SkillEffectType::FlatDamage {
            min,
            max,
            damage_type,
        } => {
            let dmg_type = match damage_type {
                DamageType::Physical => "Physical",
                DamageType::Fire => "Fire",
            };
            format!(
                "Deals {:.0}–{:.0} {} Damage to {} ({})",
                min, max, dmg_type, target, shape
            )
        }
        SkillEffectType::Heal { min, max } => {
            format!("Heals {:.0}–{:.0} HP to {} ({})", min, max, target, shape)
        }
    };

    view! { <li class="text-sm text-purple-200 leading-snug">{desc}</li> }
}
