use std::time::Duration;
use strum::IntoEnumIterator;

use shared::{
    constants::{MAX_BLOCK, MAX_EVADE},
    data::{
        area::AreaThreat,
        character::{CharacterId, CharacterSpecs, CharacterState},
        conditional_modifier::ConditionalModifier,
        skill::{DamageType, RestoreModifier, RestoreType, SkillType},
        stat_effect::{LuckyRollType, StatConverterSource, StatEffect, StatType},
    },
};

use crate::game::{
    data::event::{EventsQueue, GameEvent},
    systems::{characters_controller::restore_character, stats_updater},
};

use super::statuses_controller;

pub fn update_character_state(
    events_queue: &mut EventsQueue,
    elapsed_time: Duration,
    character_id: CharacterId,
    character_specs: &CharacterSpecs,
    character_state: &mut CharacterState,
    area_threat: &AreaThreat,
) {
    if !character_state.is_alive {
        return;
    }

    let elapsed_time_f64 = elapsed_time.as_secs_f64();

    statuses_controller::update_character_statuses(character_specs, character_state, elapsed_time);

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
        elapsed_time_f64 * *character_specs.life_regen * 0.1,
        RestoreModifier::Percent,
    );

    restore_character(
        &mut (character_id, (character_specs, character_state)),
        RestoreType::Mana,
        elapsed_time_f64 * *character_specs.mana_regen * 0.1,
        RestoreModifier::Percent,
    );

    character_state.life = character_state
        .life
        .get()
        .min(character_specs.max_life.get())
        .into();
    character_state.mana = character_state
        .mana
        .get()
        .min(character_specs.max_mana.get())
        .into();

    if character_state.life.get() < 0.5 {
        character_state.life = 0.0.into();
        character_state.is_alive = false;
        events_queue.register_event(GameEvent::Kill {
            target: character_id,
        });
    }

    let new_conditions = stats_updater::compute_conditions(
        area_threat,
        character_specs,
        character_state,
        &character_specs.conditional_modifiers,
    );
    if character_state.monitored_conditions != new_conditions {
        character_state.monitored_conditions = new_conditions;
        character_state.dirty_specs = true;
    }
}

pub fn reset_character(character_state: &mut CharacterState) {
    character_state.just_hurt = false;
    character_state.just_hurt_crit = false;
    character_state.just_blocked = false;
    character_state.just_evaded = false;
}

/// Return converted stats for propagation
pub fn update_character_specs(
    base_specs: &CharacterSpecs,
    effects: &[StatEffect],
) -> (CharacterSpecs, Vec<StatEffect>) {
    let mut character_specs = base_specs.clone();

    for skill_type in SkillType::iter() {
        character_specs
            .block
            .entry(skill_type)
            .or_default()
            .value
            .set_bounds(Some(0.0), Some(MAX_BLOCK));
    }

    for damage_type in DamageType::iter() {
        character_specs
            .evade
            .entry(damage_type)
            .or_default()
            .value
            .set_bounds(Some(0.0), Some(MAX_EVADE));
    }

    let converted_effects = compute_character_specs(&mut character_specs, effects);
    (character_specs, converted_effects)
}

fn compute_character_specs(
    character_specs: &mut CharacterSpecs,
    effects: &[StatEffect],
) -> Vec<StatEffect> {
    let mut stats_converters = Vec::new();

    for effect in effects.iter() {
        match &effect.stat {
            StatType::Life => character_specs.max_life.apply_effect(effect),
            StatType::LifeRegen => character_specs.life_regen.apply_effect(effect),
            StatType::Mana => character_specs.max_mana.apply_effect(effect),
            StatType::ManaRegen => character_specs.mana_regen.apply_effect(effect),
            StatType::Armor(damage_type) => match damage_type {
                Some(damage_type) => character_specs
                    .armor
                    .entry(*damage_type)
                    .or_default()
                    .apply_effect(effect),
                None => {
                    for damage_type in DamageType::iter() {
                        character_specs
                            .armor
                            .entry(damage_type)
                            .or_default()
                            .apply_effect(effect)
                    }
                }
            },
            StatType::TakeFromManaBeforeLife => character_specs
                .take_from_mana_before_life
                .apply_effect(effect),
            StatType::TakeFromLifeBeforeMana => character_specs
                .take_from_life_before_mana
                .apply_effect(effect),
            StatType::Block(skill_type) => match skill_type {
                Some(skill_type) => character_specs
                    .block
                    .entry(*skill_type)
                    .or_default()
                    .value
                    .apply_effect(effect),
                None => {
                    for skill_type in SkillType::iter() {
                        character_specs
                            .block
                            .entry(skill_type)
                            .or_default()
                            .value
                            .apply_effect(effect)
                    }
                }
            },
            StatType::BlockDamageTaken => character_specs.block_damage.apply_effect(effect),
            StatType::Evade(damage_type) => match damage_type {
                Some(damage_type) => character_specs
                    .evade
                    .entry(*damage_type)
                    .or_default()
                    .value
                    .apply_effect(effect),
                None => {
                    for damage_type in DamageType::iter() {
                        character_specs
                            .evade
                            .entry(damage_type)
                            .or_default()
                            .value
                            .apply_effect(effect)
                    }
                }
            },
            StatType::EvadeDamageTaken => character_specs.evade_damage.apply_effect(effect),
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
                        character_specs
                            .damage_resistance
                            .entry((skill, damage))
                            .or_default()
                            .apply_effect(effect);
                    }
                }
            }
            StatType::StatusResistance {
                skill_type,
                status_type,
            } => {
                let skill_types = match skill_type {
                    Some(skill_type) => vec![*skill_type],
                    None => SkillType::iter().collect(),
                };

                for &skill in &skill_types {
                    character_specs
                        .status_resistances
                        .entry((skill, status_type.clone()))
                        .or_default()
                        .apply_effect(effect);
                }
            }
            StatType::Lucky {
                skill_type,
                roll_type: LuckyRollType::Block,
            } => match skill_type {
                Some(skill_type) => character_specs
                    .block
                    .entry(*skill_type)
                    .or_default()
                    .lucky_chance
                    .apply_effect(effect),
                None => {
                    for skill_type in SkillType::iter() {
                        character_specs
                            .block
                            .entry(skill_type)
                            .or_default()
                            .lucky_chance
                            .apply_effect(effect)
                    }
                }
            },
            StatType::Lucky {
                skill_type: _,
                roll_type: LuckyRollType::Evade(damage_type),
            } => match damage_type {
                Some(damage_type) => character_specs
                    .evade
                    .entry(*damage_type)
                    .or_default()
                    .lucky_chance
                    .apply_effect(effect),
                None => {
                    for damage_type in DamageType::iter() {
                        character_specs
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
            StatType::StatConditionalModifier { stat, conditions } => {
                character_specs
                    .conditional_modifiers
                    .push(ConditionalModifier {
                        conditions: conditions.clone(),
                        effects: [StatEffect {
                            stat: *(*stat).clone(),
                            modifier: effect.modifier,
                            value: effect.value,
                            bypass_ignore: effect.bypass_ignore,
                        }]
                        .into(),
                    });
            }
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
            | StatType::Speed(_)
            | StatType::Lucky { .. }
            | StatType::SuccessChance { .. }
            | StatType::SkillLevel(_)
            | StatType::SkillConditionalModifier { .. }
            | StatType::SkillTargetModifier { .. } => {}
            // Other
            StatType::ItemRarity
            | StatType::GemsFind
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
                StatConverterSource::MaxLife => character_specs
                    .max_life
                    .convert_value(factor, specs.is_extra, false)
                    .get(),
                StatConverterSource::MaxMana => character_specs
                    .max_mana
                    .convert_value(factor, specs.is_extra, false)
                    .get(),
                StatConverterSource::ManaRegen => {
                    character_specs
                        .mana_regen
                        .convert_value(factor, specs.is_extra, false)
                }
                StatConverterSource::LifeRegen => {
                    character_specs
                        .life_regen
                        .convert_value(factor, specs.is_extra, false)
                }
                StatConverterSource::Block(skill_type) => {
                    if let Some(block) = character_specs.block.get_mut(&skill_type) {
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
    character_specs: &CharacterSpecs,
    source: &StatConverterSource,
) -> f64 {
    match source {
        StatConverterSource::MaxLife => character_specs.max_life.get(),
        StatConverterSource::MaxMana => character_specs.max_mana.get(),
        StatConverterSource::ManaRegen => *character_specs.mana_regen,
        StatConverterSource::LifeRegen => *character_specs.life_regen,
        StatConverterSource::Block(skill_type) => {
            if let Some(block) = character_specs.block.get(skill_type) {
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
