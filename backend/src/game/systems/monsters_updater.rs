use std::time::Duration;

use shared::{
    constants::THREAT_EFFECT,
    data::{
        area::AreaThreat,
        character::CharacterId,
        modifier::Modifier,
        monster::{MonsterSpecs, MonsterState},
        stat_effect::{StatEffect, StatType},
    },
};

use crate::game::data::{event::EventsQueue, master_store::StatusesStore};

use super::{characters_updater, skills_updater};

pub fn update_monster_states(
    statuses_store: &StatusesStore,
    events_queue: &mut EventsQueue,
    elapsed_time: Duration,
    monster_specs: &[MonsterSpecs],
    monster_states: &mut [MonsterState],
    area_threat: &AreaThreat,
) {
    for (monster_id, (monster_state, monster_specs)) in monster_states
        .iter_mut()
        .zip(monster_specs.iter())
        .enumerate()
        .filter(|(_, (s, _))| s.character_state.is_alive)
    {
        characters_updater::update_character_state(
            statuses_store,
            events_queue,
            elapsed_time,
            CharacterId::Monster(monster_id),
            &monster_specs.character_specs,
            &mut monster_state.character_state,
            None,
            area_threat,
        );
    }
}

pub fn reset_monsters(monster_states: &mut [MonsterState]) {
    for monster_state in monster_states.iter_mut() {
        characters_updater::reset_character(&mut monster_state.character_state);
    }
}

pub fn update_monster_specs(
    statuses_store: &StatusesStore,
    character_id: CharacterId,
    base_specs: &MonsterSpecs,
    monster_specs: &mut MonsterSpecs,
    monster_state: &MonsterState,
    area_threat: &AreaThreat,
) {
    monster_specs.character_specs = characters_updater::update_character_specs(
        statuses_store,
        area_threat,
        &base_specs.character_specs,
        &monster_state.character_state,
        None,
        [StatEffect {
            stat: StatType::Damage {
                skill_filter: Default::default(),
                damage_type: None,
                min_max: None,
                is_hit: None,
            },
            modifier: Modifier::Increased,
            value: ((1.0 + THREAT_EFFECT).powf(area_threat.threat_level as f64) - 1.0) * 100.0,
            bypass_ignore: false,
        }]
        .into(),
    );

    let effects = &monster_specs.character_specs.effects;

    for skill_specs in monster_specs.character_specs.skills_specs.iter_mut() {
        skills_updater::apply_effects_to_skill_specs(statuses_store, skill_specs, effects.iter());
    }

    for trigger_effect in monster_specs.character_specs.triggers.effects_iter_mut() {
        for skill_effect in trigger_effect.effects.iter_mut() {
            skills_updater::compute_skill_specs_effect(
                statuses_store,
                &trigger_effect.trigger_id,
                trigger_effect.skill_type,
                skill_effect,
                effects.iter(),
            );
        }
    }

    characters_updater::extend_triggers_from_skills_and_statuses(
        statuses_store,
        character_id,
        &mut monster_specs.character_specs,
        &monster_state.character_state,
    );
}
