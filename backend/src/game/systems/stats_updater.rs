use std::collections::HashMap;

use shared::data::{
    area::AreaThreat,
    conditional_modifier::{Condition, ConditionalModifier},
    player::{CharacterSpecs, CharacterState},
    stat_effect::{StatEffect, compare_options},
};

// pub fn stats_map_to_vec(effects: &EffectsMap) -> Vec<StatEffect> {
//     combine_effects(effects.into())
// }

// pub fn combine_effects(mut effects: Vec<StatEffect>) -> Vec<StatEffect> {
//     sort_stat_effects(&mut effects);
//     effects
// }

// pub fn sort_stat_effects(effects: &mut [StatEffect]) {
//     effects.sort_by_key(|e| {
//         (
//             match e.stat {
//                 StatType::StatConverter(ref specs) => match specs.target_modifier {
//                     Modifier::Flat => 1,
//                     Modifier::Multiplier | Modifier::More => 3,
//                 },
//                 _ => match e.modifier {
//                     Modifier::Flat => 0,
//                     Modifier::Multiplier | Modifier::More => 2,
//                 },
//             },
//             e.stat.clone(),
//         )
//     });
// }

pub fn compute_conditional_modifiers(
    area_threat: &AreaThreat,
    character_specs: &CharacterSpecs,
    character_state: &CharacterState,
    conditional_modifiers: &[ConditionalModifier],
) -> Vec<StatEffect> {
    conditional_modifiers
        .iter()
        .flat_map(|conditional_modifier| {
            let factor: f64 = conditional_modifier
                .conditions
                .iter()
                .map(|condition| {
                    check_condition(area_threat, character_specs, character_state, condition)
                })
                .product();
            conditional_modifier
                .effects
                .iter()
                .cloned()
                .map(move |effect| StatEffect {
                    value: effect.value * factor,
                    ..effect
                })
        })
        .collect()
}

pub fn check_condition(
    area_threat: &AreaThreat,
    character_specs: &CharacterSpecs,
    character_state: &CharacterState,
    condition: &Condition,
) -> f64 {
    match condition {
        Condition::HasStatus {
            status_type,
            skill_type,
            not,
        } => {
            (character_state
                .statuses
                .iter()
                .any(|(status_specs, status_state)| {
                    compare_options(status_type, &Some(status_specs.into()))
                        && compare_options(skill_type, &Some(status_state.skill_type))
                })
                != *not) as usize as f64
        }
        Condition::StatusStacks {
            status_type,
            skill_type,
        } => character_state
            .statuses
            .iter()
            .filter(|(status_specs, status_state)| {
                compare_options(status_type, &Some(status_specs.into()))
                    && compare_options(skill_type, &Some(status_state.skill_type))
            })
            .count() as f64,
        Condition::MaximumLife => {
            (character_state.life.get() >= character_specs.max_life.get() * 0.99) as usize as f64
        }
        Condition::MaximumMana => {
            (character_state.mana.get() >= character_specs.max_mana.get() * 0.99) as usize as f64
        }
        Condition::LowLife => {
            (character_state.life.get() <= character_specs.max_life.get() * 0.5) as usize as f64
        }
        Condition::LowMana => {
            (character_state.mana.get() <= character_specs.max_mana.get() * 0.5) as usize as f64
        }
        Condition::ThreatLevel => area_threat.threat_level as f64,
    }
}

pub fn compute_conditions(
    area_threat: &AreaThreat,
    character_specs: &CharacterSpecs,
    character_state: &CharacterState,
    conditional_modifiers: &[ConditionalModifier],
) -> HashMap<Condition, f64> {
    conditional_modifiers
        .iter()
        .fold(HashMap::new(), |mut acc, value| {
            for condition in &value.conditions {
                acc.entry(condition.clone()).or_insert(check_condition(
                    area_threat,
                    character_specs,
                    character_state,
                    condition,
                ));
            }
            acc
        })
}
