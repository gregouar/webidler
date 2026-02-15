use std::collections::HashMap;

use shared::data::{
    area::AreaThreat,
    conditional_modifier::{Condition, ConditionalModifier},
    modifier::Modifier,
    player::{CharacterSpecs, CharacterState},
    stat_effect::{
        EffectsMap, StatConverterSource, StatConverterSpecs, StatEffect, StatType, compare_options,
    },
};

// maybe AreaThreat should be some kind of more generic "Context"
pub fn stats_map_to_vec(effects: &EffectsMap, area_threat: &AreaThreat) -> Vec<StatEffect> {
    combine_effects(effects.into(), area_threat)
}

// maybe AreaThreat should be some kind of more generic "Context"
pub fn combine_effects(mut effects: Vec<StatEffect>, area_threat: &AreaThreat) -> Vec<StatEffect> {
    let to_add: Vec<_> = effects
        .iter()
        .flat_map(|effect| {
            if let StatType::StatConverter(StatConverterSpecs {
                source: StatConverterSource::ThreatLevel,
                target_stat,
                target_modifier,
                ..
            }) = &effect.stat
            {
                Some(StatEffect {
                    stat: *target_stat.clone(),
                    modifier: *target_modifier,
                    value: if target_stat.is_multiplicative() {
                        ((1.0 + effect.value * 0.01).powf(area_threat.threat_level as f64) - 1.0)
                            * 100.0
                    } else {
                        effect.value * area_threat.threat_level as f64
                    },
                    bypass_ignore: false,
                })
            } else {
                None
            }
        })
        .collect();

    effects.extend(to_add);
    sort_stat_effects(&mut effects);

    effects
}

pub fn sort_stat_effects(effects: &mut [StatEffect]) {
    effects.sort_by_key(|e| {
        (
            match e.stat {
                StatType::StatConverter(ref specs) => match specs.target_modifier {
                    Modifier::Flat => 1,
                    Modifier::Multiplier | Modifier::More => 3,
                },
                _ => match e.modifier {
                    Modifier::Flat => 0,
                    Modifier::Multiplier | Modifier::More => 2,
                },
            },
            e.stat.clone(),
        )
    });
}

pub fn compute_conditional_modifiers(
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
                .map(|condition| check_condition(character_specs, character_state, condition))
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
            (character_state.life >= character_specs.max_life.evaluate() * 0.99) as usize as f64
        }
        Condition::MaximumMana => {
            (character_state.mana >= character_specs.max_mana.evaluate() * 0.99) as usize as f64
        }
        Condition::LowLife => {
            (character_state.life <= character_specs.max_life.evaluate() * 0.5) as usize as f64
        }
        Condition::LowMana => {
            (character_state.mana <= character_specs.max_mana.evaluate() * 0.5) as usize as f64
        }
    }
}

pub fn compute_conditions(
    character_specs: &CharacterSpecs,
    character_state: &CharacterState,
    conditional_modifiers: &[ConditionalModifier],
) -> HashMap<Condition, f64> {
    conditional_modifiers
        .iter()
        .fold(HashMap::new(), |mut acc, value| {
            for condition in &value.conditions {
                acc.entry(condition.clone()).or_insert(check_condition(
                    character_specs,
                    character_state,
                    condition,
                ));
            }
            acc
        })
}
