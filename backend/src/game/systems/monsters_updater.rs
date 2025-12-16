use std::time::Duration;

use shared::{
    constants::THREAT_EFFECT,
    data::{
        area::AreaThreat,
        character::CharacterId,
        character_status::StatusSpecs,
        monster::{MonsterSpecs, MonsterState},
        passive::StatEffect,
        stat_effect::{Modifier, StatType},
    },
};

use crate::game::{
    data::event::EventsQueue,
    systems::{stats_updater, statuses_controller},
};

use super::{characters_updater, skills_updater};

pub fn update_monster_states(
    events_queue: &mut EventsQueue,
    elapsed_time: Duration,
    monster_specs: &[MonsterSpecs],
    monster_states: &mut [MonsterState],
) {
    for (monster_id, (monster_state, monster_specs)) in monster_states
        .iter_mut()
        .zip(monster_specs.iter())
        .enumerate()
        .filter(|(_, (s, _))| s.character_state.is_alive)
    {
        monster_state.initiative = (monster_state.initiative - elapsed_time.as_secs_f32()).max(0.0);

        characters_updater::update_character_state(
            events_queue,
            elapsed_time,
            CharacterId::Monster(monster_id),
            &monster_specs.character_specs,
            &mut monster_state.character_state,
        );

        if monster_state.initiative > 0.0 || monster_state.character_state.is_stunned() {
            continue;
        }

        skills_updater::update_skills_states(
            elapsed_time,
            &monster_specs.skill_specs,
            &mut monster_state.skill_states,
        );
    }
}

pub fn reset_monsters(monster_states: &mut [MonsterState]) {
    for monster_state in monster_states.iter_mut() {
        characters_updater::reset_character(&mut monster_state.character_state);
        skills_updater::reset_skills(&mut monster_state.skill_states);
    }
}

pub fn update_monster_specs(
    base_specs: &MonsterSpecs,
    monster_specs: &mut MonsterSpecs,
    monster_state: &MonsterState,
    area_threat: &AreaThreat,
) {
    let mut effects = stats_updater::stats_map_to_vec(
        &statuses_controller::generate_effects_map_from_statuses(
            &monster_state.character_state.statuses,
        ),
        area_threat,
    );

    effects.push(StatEffect {
        stat: StatType::Damage {
            skill_type: None,
            damage_type: None,
        },
        value: ((1.0 + THREAT_EFFECT).powf(area_threat.threat_level as f64) - 1.0) * 100.0,
        modifier: Modifier::Multiplier,
        bypass_ignore: true,
    });

    monster_specs.character_specs =
        characters_updater::update_character_specs(&base_specs.character_specs, &effects);
    monster_specs.skill_specs = base_specs.skill_specs.clone();

    for skill_specs in monster_specs.skill_specs.iter_mut() {
        skills_updater::apply_effects_to_skill_specs(skill_specs, effects.iter());
    }

    monster_specs.character_specs.triggers = monster_specs
        .skill_specs
        .iter()
        .flat_map(|skill_specs| skill_specs.triggers.iter())
        .chain(
            monster_state
                .character_state
                .statuses
                .iter()
                .filter_map(|(status_specs, _)| match status_specs {
                    StatusSpecs::Trigger(trigger_specs) => Some(trigger_specs.as_ref()),
                    _ => None,
                }),
        )
        .map(|trigger_specs| trigger_specs.triggered_effect.clone())
        .collect();
}
