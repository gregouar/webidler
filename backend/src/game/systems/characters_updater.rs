use std::time::Duration;
use strum::IntoEnumIterator;

use shared::{
    constants::{MAX_BLOCK, MAX_EVADE},
    data::{
        character::{CharacterId, CharacterSpecs, CharacterState},
        conditional_modifier::ConditionalModifier,
        passive::StatEffect,
        skill::{DamageType, RestoreType, SkillType},
        stat_effect::{ApplyStatModifier, LuckyRollType, StatConverterSource, StatType},
        temple::Modifier,
    },
};

use crate::game::{
    data::event::{EventsQueue, GameEvent},
    systems::{characters_controller::restore_character, stats_updater},
    utils::rng::Rollable,
};

use super::statuses_controller;

pub fn update_character_state(
    events_queue: &mut EventsQueue,
    elapsed_time: Duration,
    character_id: CharacterId,
    character_specs: &CharacterSpecs,
    character_state: &mut CharacterState,
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
        elapsed_time_f64 * character_specs.life_regen * 0.1,
        Modifier::Multiplier,
    );

    restore_character(
        &mut (character_id, (character_specs, character_state)),
        RestoreType::Mana,
        elapsed_time_f64 * character_specs.mana_regen * 0.1,
        Modifier::Multiplier,
    );

    character_state.life = character_state.life.min(character_specs.max_life);
    character_state.mana = character_state.mana.min(character_specs.max_mana);

    if character_state.life < 0.5 {
        character_state.life = character_state.life.min(0.0);
        character_state.is_alive = false;
        events_queue.register_event(GameEvent::Kill {
            target: character_id,
        });
    }

    let new_conditions = stats_updater::compute_conditions(
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

pub fn update_character_specs(
    base_specs: &CharacterSpecs,
    effects: &[StatEffect],
) -> CharacterSpecs {
    let mut character_specs = base_specs.clone();
    compute_character_specs(&mut character_specs, effects);
    character_specs
}

fn compute_character_specs(character_specs: &mut CharacterSpecs, effects: &[StatEffect]) {
    let mut stat_converters = Vec::new();

    for effect in effects.iter() {
        match effect.stat {
            StatType::Life => character_specs.max_life.apply_effect(effect),
            StatType::LifeRegen => character_specs.life_regen.apply_effect(effect),
            StatType::Mana => character_specs.max_mana.apply_effect(effect),
            StatType::ManaRegen => character_specs.mana_regen.apply_effect(effect),
            StatType::Armor(damage_type) => match damage_type {
                Some(damage_type) => character_specs
                    .armor
                    .entry(damage_type)
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
                    .entry(skill_type)
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
                    .entry(damage_type)
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
            StatType::DamageResistance {
                skill_type,
                damage_type,
            } => {
                let skill_types = match skill_type {
                    Some(skill_type) => vec![skill_type],
                    None => SkillType::iter().collect(),
                };

                let damage_types = match damage_type {
                    Some(damage_type) => vec![damage_type],
                    None => DamageType::iter().collect(),
                };

                for &skill in &skill_types {
                    for &damage in &damage_types {
                        character_specs
                            .damage_resistance
                            .entry((skill, damage))
                            .or_insert(0.0)
                            .apply_effect(effect);
                    }
                }
            }
            StatType::Lucky {
                skill_type,
                roll_type: LuckyRollType::Block,
            } => match skill_type {
                Some(skill_type) => character_specs
                    .block
                    .entry(skill_type)
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
            StatType::StatConverter(ref specs) => {
                stat_converters.push((specs.clone(), effect.value));
            }
            StatType::StatConditionalModifier {
                ref stat,
                ref conditions,
            } => {
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
            StatType::LifeOnHit { .. } | StatType::ManaOnHit { .. } => {}
            // Only for player
            StatType::MovementSpeed | StatType::GoldFind | StatType::ThreatGain => {}
            // Delegate to skills
            StatType::ManaCost { .. }
            | StatType::Damage { .. }
            | StatType::MinDamage { .. }
            | StatType::MaxDamage { .. }
            | StatType::Restore { .. }
            | StatType::CritChance(_)
            | StatType::CritDamage(_)
            | StatType::StatusDuration { .. }
            | StatType::StatusPower { .. }
            | StatType::Speed(_)
            | StatType::Lucky { .. }
            | StatType::SuccessChance { .. }
            | StatType::SkillLevel(_)
            | StatType::SkillConditionalModifier { .. } => {}
            // Other
            StatType::ItemRarity => {}
        }
    }

    // TODO: How to propagate to player/monster/skills? Return effects?
    if !stat_converters.is_empty() {
        let mut stats_converted = Vec::with_capacity(stat_converters.len());

        for (specs, factor) in stat_converters {
            let factor = factor * 0.01;
            let amount = match specs.source {
                StatConverterSource::MaxLife => {
                    let amount = character_specs.max_life * factor;
                    if !specs.is_extra {
                        character_specs.max_life -= amount;
                    }
                    amount
                }
                StatConverterSource::MaxMana => {
                    let amount = character_specs.max_mana * factor;
                    if !specs.is_extra {
                        character_specs.max_mana -= amount;
                    }
                    amount
                }
                StatConverterSource::ManaRegen => {
                    let amount = character_specs.mana_regen * factor;
                    if !specs.is_extra {
                        character_specs.mana_regen -= amount;
                    }
                    amount
                }
                StatConverterSource::LifeRegen => {
                    let amount = character_specs.life_regen * factor;
                    if !specs.is_extra {
                        character_specs.life_regen -= amount;
                    }
                    amount
                }
                StatConverterSource::Block(skill_type) => {
                    if let Some(block) = character_specs.block.get_mut(&skill_type) {
                        let amount = block.value as f64 * factor;
                        if !specs.is_extra {
                            block.value -= amount as f32;
                        }
                        amount
                    } else {
                        0.0
                    }
                }

                StatConverterSource::CritDamage
                | StatConverterSource::Damage { .. }
                | StatConverterSource::MinDamage { .. }
                | StatConverterSource::MaxDamage { .. }
                | StatConverterSource::ThreatLevel => {
                    continue;
                }
            };

            stats_converted.push(StatEffect {
                stat: (*specs.target_stat).clone(),
                modifier: specs.target_modifier,
                value: amount,
                bypass_ignore: true,
            });
        }

        compute_character_specs(character_specs, &stats_converted);
    }

    character_specs.max_life = character_specs.max_life.max(1.0);
    character_specs.max_mana = character_specs.max_mana.max(0.0);
    for block in character_specs.block.values_mut() {
        block.clamp();
        block.value = block.value.min(MAX_BLOCK);
    }
    for evade in character_specs.evade.values_mut() {
        evade.clamp();
        evade.value = evade.value.min(MAX_EVADE);
    }
    character_specs.block_damage = character_specs.block_damage.clamp(0.0, 100.0);
}
