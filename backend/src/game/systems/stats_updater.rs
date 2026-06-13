use shared::data::{
    area::AreaThreat,
    character::CharacterAttrs,
    character_status::StatusEffectType,
    conditional_modifier::{Condition, ConditionalModifier},
    player::{CharacterState, PlayerInventory},
    stat_effect::{StatEffect, StatType},
};

use crate::game::data::master_store::StatusesStore;

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
    statuses_store: &StatusesStore,
    area_threat: &AreaThreat,
    character_attrs: &CharacterAttrs,
    character_state: &CharacterState,
    character_inventory: Option<&PlayerInventory>,
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
                        check_condition(
                            statuses_store,
                            area_threat,
                            character_attrs,
                            character_state,
                            character_inventory,
                            condition,
                        )
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
    statuses_store: &StatusesStore,
    area_threat: &AreaThreat,
    character_attrs: &CharacterAttrs,
    character_state: &CharacterState,
    character_inventory: Option<&PlayerInventory>,
    condition: &Condition,
) -> f64 {
    match condition {
        Condition::HasStatus {
            status_filter,
            skill_type,
            not,
        } => {
            (character_state
                .statuses
                .iter()
                .any(|(status_id, status_states)| {
                    let Some(status_specs) = statuses_store.get(status_id) else {
                        return false;
                    };
                    status_filter.is_match_with_status(status_id, status_specs.damage_type)
                        && skill_type
                            .map(|skill_type| {
                                status_states
                                    .iter()
                                    .any(|status_state| status_state.skill_type == skill_type)
                            })
                            .unwrap_or(true)
                })
                != *not) as usize as f64
        }
        Condition::StatusStacks {
            status_filter,
            skill_type,
        } => character_state
            .statuses
            .iter()
            .filter(|(status_id, status_states)| {
                let Some(status_specs) = statuses_store.get(status_id) else {
                    return false;
                };
                status_filter.is_match_with_status(status_id, status_specs.damage_type)
                    && skill_type
                        .map(|skill_type| {
                            status_states
                                .iter()
                                .any(|status_state| status_state.skill_type == skill_type)
                        })
                        .unwrap_or(true)
            })
            .count() as f64,
        Condition::Slowed => character_state.statuses.iter().any(|(status_id, _)| {
            let Some(status_specs) = statuses_store.get(status_id) else {
                return false;
            };
            status_specs.effects.iter().any(|effect| {
                matches!(
                    effect.status_effect_type,
                    StatusEffectType::StatModifier {
                        stat: StatType::Speed(_),
                        modifier: _,
                        debuff: true
                    }
                )
            })
        }) as usize as f64,

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
        Condition::HasItem {
            item_slot,
            item_category,
        } => character_inventory
            .map(|character_inventory| {
                character_inventory
                    .equipped_items()
                    .any(|(equipped_slot, item_specs)| {
                        item_slot
                            .map(|item_slot| {
                                equipped_slot == item_slot
                                    || item_specs.base.extra_slots.contains(&item_slot)
                            })
                            .unwrap_or(true)
                            && item_category
                                .map(|item_category| {
                                    item_specs.base.categories.contains(&item_category)
                                })
                                .unwrap_or(true)
                    }) as usize as f64
            })
            .unwrap_or_default(),
    }
}
