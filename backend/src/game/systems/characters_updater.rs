use std::time::Duration;
use strum::IntoEnumIterator;

use shared::data::{
    character::{CharacterId, CharacterSpecs, CharacterState},
    passive::StatEffect,
    skill::{DamageType, SkillType},
    stat_effect::{EffectsMap, Modifier, StatType},
};

use crate::game::{
    data::event::{EventsQueue, GameEvent},
    systems::stats_controller::ApplyStatModifier,
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

    character_state.life = character_specs.max_life.min(
        character_state.life
            + (elapsed_time_f64 * character_specs.life_regen * character_specs.max_life / 100.0),
    );

    character_state.mana = character_specs.max_mana.min(
        character_state.mana
            + (elapsed_time_f64 * character_specs.mana_regen * character_specs.max_mana / 100.0),
    );

    statuses_controller::update_character_statuses(character_specs, character_state, elapsed_time);

    character_state.life = character_state.life.clamp(0.0, character_specs.max_life);
    character_state.mana = character_state.mana.clamp(0.0, character_specs.max_mana);

    if character_state.life < 0.5 {
        character_state.life = 0.0;
        character_state.is_alive = false;
        events_queue.register_event(GameEvent::Kill {
            target: character_id,
        });
    }
}

pub fn reset_character(character_state: &mut CharacterState) {
    character_state.just_hurt = false;
    character_state.just_hurt_crit = false;
    character_state.just_blocked = false;
}

pub fn stats_map_to_vec(effects: &EffectsMap) -> Vec<StatEffect> {
    let mut effects: Vec<_> = effects.into();

    effects.sort_by_key(|e| match e.modifier {
        Modifier::Flat => 0,
        Modifier::Multiplier => 1,
    });

    effects
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
    for effect in effects.iter() {
        match effect.stat {
            StatType::Life => character_specs.max_life.apply_effect(effect),
            StatType::LifeRegen => character_specs.life_regen.apply_effect(effect),
            StatType::Mana => character_specs.max_mana.apply_effect(effect),
            StatType::ManaRegen => character_specs.mana_regen.apply_effect(effect),
            StatType::Armor(armor_type) => match armor_type {
                DamageType::Physical => character_specs.armor.apply_effect(effect),
                DamageType::Fire => character_specs.fire_armor.apply_effect(effect),
                DamageType::Poison => character_specs.poison_armor.apply_effect(effect),
            },
            StatType::TakeFromManaBeforeLife => character_specs
                .take_from_mana_before_life
                .apply_effect(effect),
            StatType::Block => character_specs.block.apply_effect(effect),
            StatType::DamageTaken {
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
                            .damage_taken
                            .entry((skill, damage))
                            .or_insert(1.0)
                            .apply_effect(effect);
                    }
                }
            }
            // Only for player (for now...)
            StatType::LifeOnHit(_) | StatType::ManaOnHit(_) => {}
            // Only for player
            StatType::MovementSpeed | StatType::GoldFind => {}
            // Delegate to skills
            StatType::Damage { .. }
            | StatType::MinDamage { .. }
            | StatType::MaxDamage { .. }
            | StatType::SpellPower
            | StatType::CritChances(_)
            | StatType::CritDamage(_)
            | StatType::Speed(_) => {}
        }
    }
}
