use shared::data::{
    monster::{MonsterSpecs, MonsterState},
    player::{PlayerSpecs, PlayerState},
};

use super::character_controller;

pub fn control_monsters(
    monsters: &mut Vec<(&MonsterSpecs, &mut MonsterState)>,
    player_specs: &PlayerSpecs,
    player_state: &mut PlayerState,
) {
    if !player_state.character_state.is_alive {
        return;
    }

    for (monster_specs, monster_state) in monsters
        .iter_mut()
        .filter(|(_, m)| m.character_state.is_alive && m.initiative == 0.0)
    {
        for (skill_specs, skill_state) in monster_specs
            .skill_specs
            .iter()
            .zip(monster_state.skill_states.iter_mut())
            .filter(|(_, s)| s.is_ready)
        {
            character_controller::use_skill(
                &skill_specs,
                skill_state,
                (
                    &monster_specs.character_specs,
                    &mut monster_state.character_state,
                ),
                vec![], // TODO
                vec![(
                    &player_specs.character_specs,
                    &mut player_state.character_state,
                )],
            );
        }
    }
}
