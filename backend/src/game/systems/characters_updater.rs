use std::time::Duration;
use strum::IntoEnumIterator;

use shared::{
    constants::{MAX_BLOCK, MAX_DAMAGE_RESISTANCE, MAX_EVADE},
    data::{
        area::AreaThreat,
        character::{CharacterAttrs, CharacterId, CharacterState},
        character_status::StatusEffectType,
        conditional_modifier::ConditionalModifier,
        player::{CharacterSpecs, PlayerInventory},
        skill::{DamageType, RestoreModifier, RestoreType, SkillType},
        stat_effect::{EffectsMap, LuckyRollType, StatConverterSource, StatEffect, StatType},
        trigger::TriggerEffectModifierSource,
    },
};

use crate::game::{
    data::{
        event::{EventsQueue, GameEvent},
        master_store::StatusesStore,
    },
    systems::{characters_controller::restore_character, skills_updater, stats_updater},
};

use super::statuses_controller;

#[allow(clippy::too_many_arguments)]
pub fn update_character_state(
    statuses_store: &StatusesStore,
    events_queue: &mut EventsQueue,
    elapsed_time: Duration,
    character_id: CharacterId,
    character_specs: &CharacterSpecs,
    character_state: &mut CharacterState,
    character_inventory: Option<&PlayerInventory>,
    area_threat: &AreaThreat,
) {
    if !character_state.is_alive {
        return;
    }

    let elapsed_time_f64 = elapsed_time.as_secs_f64();

    statuses_controller::update_character_statuses(
        statuses_store,
        &character_specs.character_attrs,
        character_state,
        elapsed_time,
    );

    // character_state.life = character_specs.max_life.min(
    //     character_state.life
    //         + (elapsed_time_f64 * character_specs.life_regen * character_specs.max_life * 0.001),
    // );

    // character_state.mana = character_specs.max_mana.min(
    //     character_state.mana
    //         + (elapsed_time_f64 * character_specs.mana_regen * character_specs.max_mana * 0.001),
    // );

    restore_character(
        &mut (character_id, (character_specs, character_state)),
        RestoreType::Life,
        elapsed_time_f64 * *character_specs.character_attrs.life_regen * 0.1,
        RestoreModifier::Percent,
    );

    restore_character(
        &mut (character_id, (character_specs, character_state)),
        RestoreType::Mana,
        elapsed_time_f64 * *character_specs.character_attrs.mana_regen * 0.1,
        RestoreModifier::Percent,
    );

    character_state.life = character_state
        .life
        .get()
        .min(character_specs.character_attrs.max_life.get())
        .into();
    character_state.mana = character_state
        .mana
        .get()
        .min(character_specs.character_attrs.max_mana.get())
        .into();

    if character_state.life.get() < 0.5 {
        character_state.life = 0.0.into();
        character_state.is_alive = false;
        events_queue.register_event(GameEvent::Kill {
            target: character_id,
        });
    }

    for monitored_condition in character_state.monitored_conditions.values_mut() {
        monitored_condition.duration += elapsed_time_f64;
    }

    for conditional_modifier in character_specs.conditional_modifiers.iter() {
        for condition in conditional_modifier.conditions.iter() {
            let value = stats_updater::check_condition(
                statuses_store,
                area_threat,
                &character_specs.character_attrs,
                character_state,
                character_inventory,
                condition,
            );

            let monitored_condition = character_state
                .monitored_conditions
                .entry(condition.clone())
                .or_default();

            if monitored_condition.duration - elapsed_time_f64
                < conditional_modifier.conditions_duration as f64 * 0.1
                && monitored_condition.duration
                    >= conditional_modifier.conditions_duration as f64 * 0.1
            {
                character_state.dirty_specs = true;
            }

            if monitored_condition.value != value {
                monitored_condition.value = value;
                monitored_condition.duration = 0.0;
                character_state.dirty_specs = true;
            }
        }
    }

    if !character_state.is_stunned() {
        skills_updater::update_skills_states(
            elapsed_time,
            &character_specs.skills_specs,
            &mut character_state.skills_states,
        );

        skills_updater::update_repeated_skill_effects(
            elapsed_time,
            &mut character_state.repeated_skills,
        )
    }
}

pub fn reset_character(character_state: &mut CharacterState) {
    character_state.just_hurt = false;
    character_state.just_hurt_crit = false;
    character_state.just_blocked = false;
    character_state.just_evaded = false;

    skills_updater::reset_skills(&mut character_state.skills_states);
}

/// Return converted stats for propagation
pub fn update_character_specs(
    statuses_store: &StatusesStore,
    area_threat: &AreaThreat,
    base_specs: &CharacterSpecs,
    character_state: &CharacterState,
    character_inventory: Option<&PlayerInventory>,
    mut effects: Vec<StatEffect>,
) -> CharacterSpecs {
    let mut character_specs = base_specs.clone();

    for skill_type in SkillType::iter() {
        character_specs
            .character_attrs
            .block
            .entry(skill_type)
            .or_default()
            .value
            .base_mut()
            .set_bounds(Some(0.0), Some(MAX_BLOCK));
    }

    for damage_type in DamageType::iter() {
        character_specs
            .character_attrs
            .evade
            .entry(damage_type)
            .or_default()
            .value
            .base_mut()
            .set_bounds(Some(0.0), Some(MAX_EVADE));
    }

    for skill_type in SkillType::iter() {
        for damage_type in DamageType::iter() {
            character_specs
                .character_attrs
                .damage_resistance
                .entry((skill_type, damage_type))
                .or_default()
                .base_mut()
                .set_bounds(None, Some(MAX_DAMAGE_RESISTANCE));
        }
    }

    effects.extend(statuses_controller::generate_effects_from_statuses(
        statuses_store,
        &character_state.statuses,
    ));

    let conditional_modifiers = gather_condition_modifiers(&effects);
    effects.extend(stats_updater::compute_conditional_modifiers(
        statuses_store,
        area_threat,
        &character_specs.character_attrs,
        &character_state,
        character_inventory,
        &conditional_modifiers,
    ));
    character_specs.conditional_modifiers = conditional_modifiers;

    effects.extend(compute_character_specs(&mut character_specs, &effects));

    // Aggregate effects
    character_specs.effects = EffectsMap::from(effects).into();

    character_specs
}

fn gather_condition_modifiers(effects: &[StatEffect]) -> Vec<ConditionalModifier> {
    effects
        .iter()
        .filter_map(|effect| match &effect.stat {
            StatType::StatConditionalModifier {
                stat,
                conditions,
                conditions_duration,
            } => Some(ConditionalModifier {
                conditions: conditions.clone(),
                conditions_duration: *conditions_duration,
                effects: [StatEffect {
                    stat: *(*stat).clone(),
                    modifier: effect.modifier,
                    value: effect.value,
                    bypass_ignore: effect.bypass_ignore,
                }]
                .into(),
            }),
            _ => None,
        })
        .collect()
}

fn compute_character_specs(
    character_specs: &mut CharacterSpecs,
    effects: &[StatEffect],
) -> Vec<StatEffect> {
    let mut stats_converters = Vec::new();
    let character_attrs = &mut character_specs.character_attrs;

    for effect in effects.iter() {
        match &effect.stat {
            StatType::Life => character_attrs.max_life.apply_effect(effect),
            StatType::LifeRegen => character_attrs.life_regen.apply_effect(effect),
            StatType::Mana => character_attrs.max_mana.apply_effect(effect),
            StatType::ManaRegen => character_attrs.mana_regen.apply_effect(effect),
            StatType::Armor(armor_type) => match armor_type {
                Some(armor_type) => {
                    for damage_type in DamageType::iter() {
                        if armor_type.is_match(damage_type) {
                            character_attrs
                                .armor
                                .entry(damage_type)
                                .or_default()
                                .apply_effect(effect)
                        }
                    }
                }
                None => {
                    for damage_type in DamageType::iter() {
                        character_attrs
                            .armor
                            .entry(damage_type)
                            .or_default()
                            .apply_effect(effect)
                    }
                }
            },
            StatType::TakeFromManaBeforeLife => character_attrs
                .take_from_mana_before_life
                .apply_effect(effect),
            StatType::TakeFromLifeBeforeMana => character_attrs
                .take_from_life_before_mana
                .apply_effect(effect),
            StatType::Block(skill_type) => match skill_type {
                Some(skill_type) => character_attrs
                    .block
                    .entry(*skill_type)
                    .or_default()
                    .value
                    .apply_effect(effect),
                None => {
                    for skill_type in SkillType::iter() {
                        character_attrs
                            .block
                            .entry(skill_type)
                            .or_default()
                            .value
                            .apply_effect(effect)
                    }
                }
            },
            StatType::BlockDamageTaken => character_attrs.block_damage.apply_effect(effect),
            StatType::Evade(damage_type) => match damage_type {
                Some(damage_type) => character_attrs
                    .evade
                    .entry(*damage_type)
                    .or_default()
                    .value
                    .apply_effect(effect),
                None => {
                    for damage_type in DamageType::iter() {
                        character_attrs
                            .evade
                            .entry(damage_type)
                            .or_default()
                            .value
                            .apply_effect(effect)
                    }
                }
            },
            StatType::EvadeDamageTaken => character_attrs.evade_damage.apply_effect(effect),
            StatType::DamageResistance {
                skill_type,
                damage_type,
            } => {
                let skill_types = match skill_type {
                    Some(skill_type) => vec![*skill_type],
                    None => SkillType::iter().collect(),
                };

                let damage_types = match damage_type {
                    Some(damage_type) => vec![*damage_type],
                    None => DamageType::iter().collect(),
                };

                for &skill in &skill_types {
                    for &damage in &damage_types {
                        character_attrs
                            .damage_resistance
                            .entry((skill, damage))
                            .or_default()
                            .apply_effect(effect);
                    }
                }
            }
            StatType::StatusResistance {
                skill_type,
                status_id,
            } => {
                let skill_types = match skill_type {
                    Some(skill_type) => vec![*skill_type],
                    None => SkillType::iter().collect(),
                };

                for &skill in &skill_types {
                    character_attrs
                        .status_resistances
                        .entry((skill, status_id.clone()))
                        .or_default()
                        .apply_effect(effect);
                }
            }
            StatType::Lucky {
                skill_filter,
                roll_type: LuckyRollType::Block,
            } => match skill_filter.skill_type {
                Some(skill_type) => character_attrs
                    .block
                    .entry(skill_type)
                    .or_default()
                    .lucky_chance
                    .apply_effect(effect),
                None => {
                    for skill_type in SkillType::iter() {
                        character_attrs
                            .block
                            .entry(skill_type)
                            .or_default()
                            .lucky_chance
                            .apply_effect(effect)
                    }
                }
            },
            StatType::Lucky {
                skill_filter: _,
                roll_type: LuckyRollType::Evade(damage_type),
            } => match damage_type {
                Some(damage_type) => character_attrs
                    .evade
                    .entry(*damage_type)
                    .or_default()
                    .lucky_chance
                    .apply_effect(effect),
                None => {
                    for damage_type in DamageType::iter() {
                        character_attrs
                            .evade
                            .entry(damage_type)
                            .or_default()
                            .lucky_chance
                            .apply_effect(effect)
                    }
                }
            },

            StatType::StatConverter(specs) => {
                stats_converters.push((specs.clone(), effect.modifier, effect.value));
            }
            StatType::StatConditionalModifier {..
                // stat,
                // conditions,
                // conditions_duration,
            } => {}
            // {
            //     character_specs
            //         .conditional_modifiers
            //         .push(ConditionalModifier {
            //             conditions: conditions.clone(),
            //             conditions_duration: *conditions_duration,
            //             effects: [StatEffect {
            //                 stat: *(*stat).clone(),
            //                 modifier: effect.modifier,
            //                 value: effect.value,
            //                 bypass_ignore: effect.bypass_ignore,
            //             }]
            //             .into(),
            //         });
            // }
            // /!\ No magic _ to be sure we don't forget when adding new Stats
            // Only for player (for now...)
            StatType::RestoreOnHit { .. } => {}
            // Only for player
            StatType::MovementSpeed | StatType::GoldFind | StatType::ThreatGain => {}
            // Delegate to skills
            StatType::ManaCost { .. }
            | StatType::Damage { .. }
            | StatType::Restore { .. }
            | StatType::CritChance(_)
            | StatType::CritDamage(_)
            | StatType::StatusDuration { .. }
            | StatType::StatusPower { .. }
            | StatType::StatusEscalation { .. }
            | StatType::StatusFaster { .. }
            | StatType::StatusStacks { .. }
            | StatType::Speed(_)
            | StatType::Lucky { .. }
            | StatType::SuccessChance { .. }
            | StatType::SkillLevel(_)
            | StatType::SkillConditionalModifier { .. }
            | StatType::SkillTargetModifier { .. } => {}
            // Other
            StatType::ItemRarity
            | StatType::ItemLevel
            | StatType::GemsFind
            | StatType::PowerLevel
            | StatType::Description(_)
            | StatType::Description2(_) => {}
        }
    }

    let mut stats_converted = Vec::with_capacity(stats_converters.len());
    if !stats_converters.is_empty() {
        stats_converters.sort_by_key(|(stat_converter, modifier, _)| {
            (
                stat_converter.source,
                stat_converter.stat.clone(),
                *modifier,
            )
        });

        for (specs, modifier, factor) in stats_converters {
            // let factor = factor * 0.01;
            let amount = match specs.source {
                StatConverterSource::MaxLife => character_attrs
                    .max_life
                    .convert_value(factor, specs.is_extra, false)
                    .get(),
                StatConverterSource::MaxMana => character_attrs
                    .max_mana
                    .convert_value(factor, specs.is_extra, false)
                    .get(),
                StatConverterSource::ManaRegen => {
                    character_attrs
                        .mana_regen
                        .convert_value(factor, specs.is_extra, false)
                }
                StatConverterSource::LifeRegen => {
                    character_attrs
                        .life_regen
                        .convert_value(factor, specs.is_extra, false)
                }
                StatConverterSource::Block(skill_type) => {
                    if let Some(block) = character_attrs.block.get_mut(&skill_type) {
                        block
                            .value
                            .convert_value(factor, specs.is_extra, false)
                            .get() as f64
                    } else {
                        0.0
                    }
                }

                StatConverterSource::CritDamage
                | StatConverterSource::Damage { .. }
                // | StatConverterSource::DamageOverTime { .. }
                 => {
                    continue;
                }
            };

            stats_converted.push(StatEffect {
                stat: (*specs.stat).clone(),
                modifier,
                value: amount,
                bypass_ignore: true,
            });
        }

        compute_character_specs(character_specs, &stats_converted);
    }

    stats_converted
}

pub fn compute_stat_converter(
    character_attrs: &CharacterAttrs,
    source: &StatConverterSource,
) -> f64 {
    match source {
        StatConverterSource::MaxLife => character_attrs.max_life.get(),
        StatConverterSource::MaxMana => character_attrs.max_mana.get(),
        StatConverterSource::ManaRegen => *character_attrs.mana_regen,
        StatConverterSource::LifeRegen => *character_attrs.life_regen,
        StatConverterSource::Block(skill_type) => {
            if let Some(block) = character_attrs.block.get(skill_type) {
                block.value.get() as f64
            } else {
                0.0
            }
        }

        StatConverterSource::CritDamage
        | StatConverterSource::Damage { .. }
        // | StatConverterSource::DamageOverTime { .. } 
        => 0.0,
    }
}

pub fn extend_triggers_from_skills_and_statuses(
    statuses_store: &StatusesStore,
    character_id: CharacterId,
    character_specs: &mut CharacterSpecs,
    character_state: &CharacterState,
) {
    let effects = &character_specs.effects;

    for (status_id, status_stacks) in character_state.statuses.iter() {
        let Some(status_specs) = statuses_store.get(status_id) else {
            tracing::warn!("missing status: {status_id}");
            continue;
        };

        for status_effect in status_specs.effects.iter() {
            if let StatusEffectType::Trigger {
                trigger_specs,
                inherit_owner_effects,
            } = &status_effect.status_effect_type
            {
                for status_state in status_stacks.iter() {
                    let mut trigger_effect = trigger_specs.trigger_effect.clone();

                    let modifier_effects: Vec<_> = trigger_effect
                        .modifiers
                        .iter()
                        .filter_map(|modifier_effect| {
                            let modifier_value = match modifier_effect.source {
                                TriggerEffectModifierSource::TriggerStatusValue => {
                                    status_state.value.get()
                                }
                                TriggerEffectModifierSource::TriggerStatusDuration => {
                                    status_state.duration.get()
                                }
                                _ => 0.0,
                            };

                            (modifier_value > 0.0).then(|| StatEffect {
                                stat: modifier_effect.stat.clone(),
                                modifier: modifier_effect.modifier,
                                value: modifier_value * modifier_effect.factor,
                                bypass_ignore: true,
                            })
                        })
                        .collect();

                    let combined_effects =
                        modifier_effects.iter().chain(if *inherit_owner_effects {
                            effects.iter()
                        } else {
                            [].iter()
                        });

                    // Mandatory to compute skill effects even if modifier_effects is empty to
                    // initialize trigger status with base values
                    for skill_effect in trigger_effect.effects.iter_mut() {
                        skills_updater::compute_skill_specs_effect(
                            statuses_store,
                            &trigger_effect.trigger_id,
                            trigger_effect.skill_type,
                            skill_effect,
                            combined_effects.clone(),
                        );
                    }

                    character_specs.triggers.push(
                        trigger_specs.trigger.clone(),
                        trigger_effect,
                        Some(character_id),
                    );
                }
            }
        }
    }

    for trigger_specs in character_specs
        .skills_specs
        .iter()
        .flat_map(|skill_specs| skill_specs.triggers.iter())
    {
        character_specs.triggers.push(
            trigger_specs.trigger.clone(),
            trigger_specs.trigger_effect.clone(),
            Some(character_id),
        );
    }
}
