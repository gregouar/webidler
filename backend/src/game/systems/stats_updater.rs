use shared::data::{
    area::AreaThreat,
    character::CharacterAttrs,
    conditional_modifier::{Condition, ConditionalModifier},
    player::CharacterState,
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
    character_attrs: &CharacterAttrs,
    character_state: &CharacterState,
    conditional_modifiers: &[ConditionalModifier],
) -> Vec<StatEffect> {
    conditional_modifiers
        .iter()
        .filter_map(|conditional_modifier| {
            let factor: f64 = conditional_modifier
                .conditions
                .iter()
                .map(|condition| {
                    if let Some(monitored_condition) =
                        character_state.monitored_conditions.get(condition)
                        && monitored_condition.duration
                            >= conditional_modifier.conditions_duration as f64 * 0.1
                    {
                        monitored_condition.value
                    } else if conditional_modifier.conditions_duration == 0 {
                        // Last minute check, useful for skill modifiers that are not tracked
                        check_condition(area_threat, character_attrs, character_state, condition)
                    } else {
                        0.0
                    }
                })
                .product();

            if factor == 0.0 {
                None
            } else {
                Some((conditional_modifier, factor))
            }
        })
        .flat_map(|(conditional_modifier, factor)| {
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
    character_attrs: &CharacterAttrs,
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
            (character_state.life.get() >= character_attrs.max_life.get() * 0.9999) as usize as f64
        }
        Condition::MaximumMana => {
            (character_state.mana.get() >= character_attrs.max_mana.get() * 0.9999) as usize as f64
        }
        Condition::LowLife => {
            (character_state.life.get() <= character_attrs.max_life.get() * 0.5) as usize as f64
        }
        Condition::LowMana => {
            (character_state.mana.get() <= character_attrs.max_mana.get() * 0.5) as usize as f64
        }
        Condition::ThreatLevel => area_threat.threat_level as f64,
    }
}
