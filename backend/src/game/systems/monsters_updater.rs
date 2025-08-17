use std::time::Duration;

use shared::data::{
    character::CharacterId,
    monster::{MonsterSpecs, MonsterState},
};

use crate::game::{data::event::EventsQueue, systems::statuses_controller};

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
    monster_states: &MonsterState,
) {
    let effects = characters_updater::stats_map_to_vec(
        &statuses_controller::generate_effects_map_from_statuses(
            &monster_states.character_state.statuses,
        ),
    );

    monster_specs.character_specs =
        characters_updater::update_character_specs(&base_specs.character_specs, &effects);
    monster_specs.skill_specs = base_specs.skill_specs.clone();

    for skill_specs in monster_specs.skill_specs.iter_mut() {
        skills_updater::apply_effects_to_skill_specs(skill_specs, effects.iter());
    }
}
