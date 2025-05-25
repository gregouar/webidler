use std::time::Duration;

use shared::data::{
    character::CharacterId,
    monster::{MonsterSpecs, MonsterState},
};

use crate::game::data::event::EventsQueue;

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
        if monster_state.initiative > 0.0 {
            continue;
        }

        characters_updater::update_character_state(
            events_queue,
            elapsed_time,
            CharacterId::Monster(monster_id),
            &monster_specs.character_specs,
            &mut monster_state.character_state,
        );
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
