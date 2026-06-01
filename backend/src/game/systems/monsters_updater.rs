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

use crate::game::{
    data::{event::EventsQueue, master_store::StatusesStore},
    systems::{stats_updater, statuses_controller},
};

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
    let mut effects: Vec<_> = (&statuses_controller::generate_effects_map_from_statuses(
        statuses_store,
        &monster_state.character_state.statuses,
    ))
        .into();
    effects.push(StatEffect {
        stat: StatType::Damage {
            skill_filter: Default::default(),
            damage_type: None,
            min_max: None,
            is_hit: None,
        },
        modifier: Modifier::Increased,
        value: ((1.0 + THREAT_EFFECT).powf(area_threat.threat_level as f64) - 1.0) * 100.0,
        bypass_ignore: false,
    });
    effects.extend(stats_updater::compute_conditional_modifiers(
        area_threat,
        &monster_specs.character_specs.character_attrs,
        &monster_state.character_state,
        &monster_specs.character_specs.conditional_modifiers,
    ));

    // Compute character specs & get converted stats resulting
    let (character_specs, converted_effects) =
        characters_updater::update_character_specs(&base_specs.character_specs, &effects);
    monster_specs.character_specs = character_specs;
    effects.extend(converted_effects);

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
