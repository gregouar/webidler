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

    for (monster_id, this_monster_specs) in monster_specs.iter().enumerate() {
        // We need to separate this_monster from the others, as we will need different mutable slices
        // This feels awkward, maybe I should really consider a more ECS approach and work only wih id
        // and different data stores
        let (left, rest) = monster_states.split_at_mut(monster_id);
        let (this_monster_state, right) = match rest.split_first_mut() {
            Some(x) => x,
            None => continue,
        };

        if !this_monster_state.character_state.is_alive
            || this_monster_state.initiative > 0.0
            || this_monster_state.character_state.is_stunned()
        {
            continue;
        }

        let mut me = (
            CharacterId::Monster(monster_id),
            (
                &this_monster_specs.character_specs,
                &mut this_monster_state.character_state,
            ),
        );

        let mut friends: Vec<_> = left
            .iter_mut()
            .enumerate()
            .chain(
                right
                    .iter_mut()
                    .enumerate()
                    .map(|(i, s)| (i + 1 + monster_id, s)),
            )
            .filter_map(|(i, s)| {
                monster_specs.get(i).map(|specs| {
                    (
                        CharacterId::Monster(i),
                        (&specs.character_specs, &mut s.character_state),
                    )
                })
            })
            .collect();

        let mut player = [(
            CharacterId::Player,
            (
                &player_specs.character_specs,
                &mut player_state.character_state,
            ),
        )];

        for (skill_specs, skill_state) in this_monster_specs
            .skill_specs
            .iter()
            .zip(this_monster_state.skill_states.iter_mut())
            .filter(|(_, s)| s.is_ready)
        {
            skills_controller::use_skill(
                events_queue,
                skill_specs,
                skill_state,
                &mut me,
                &mut friends,
                &mut player,
            );
        }
    }
}
