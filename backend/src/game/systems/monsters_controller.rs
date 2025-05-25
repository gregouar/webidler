use shared::data::{
    character::CharacterId,
    monster::{MonsterSpecs, MonsterState},
    player::{PlayerSpecs, PlayerState},
};

use crate::game::data::event::EventsQueue;

use super::skills_controller;

pub fn control_monsters(
    events_queue: &mut EventsQueue,
    monster_specs: &[MonsterSpecs],
    monster_states: &mut [MonsterState],
    player_specs: &PlayerSpecs,
    player_state: &mut PlayerState,
) {
    if !player_state.character_state.is_alive {
        return;
    }

    let mut player = vec![(
        CharacterId::Player,
        (
            &player_specs.character_specs,
            &mut player_state.character_state,
        ),
    )];

    for (monster_id, (monster_specs, monster_state)) in monster_specs
        .iter()
        .zip(monster_states.iter_mut())
        .enumerate()
        .filter(|(_, (_, m))| {
            m.character_state.is_alive && m.initiative == 0.0 && !m.character_state.is_stunned()
        })
    {
        let mut monster = (
            CharacterId::Monster(monster_id),
            (
                &monster_specs.character_specs,
                &mut monster_state.character_state,
            ),
        );
        for (skill_specs, skill_state) in monster_specs
            .skill_specs
            .iter()
            .zip(monster_state.skill_states.iter_mut())
            .filter(|(_, s)| s.is_ready)
        {
            skills_controller::use_skill(
                events_queue,
                skill_specs,
                skill_state,
                &mut monster,
                &mut vec![], // TODO
                &mut player,
            );
        }
    }
}
