use shared::data::{MonsterSpecs, MonsterState, PlayerSpecs, PlayerState};

use super::character_controller;

pub fn control_monsters(
    monsters: &mut Vec<(&mut MonsterState, &MonsterSpecs)>,
    player_specs: &PlayerSpecs,
    player_state: &mut PlayerState,
) {
    if !player_state.character_state.is_alive {
        return;
    }

    for (monster_state, monster_specs) in monsters
        .iter_mut()
        .filter(|(m, _)| m.character_state.is_alive && m.initiative == 0.0)
    {
        for (skill_specs, skill_state) in monster_specs
            .character_specs
            .skill_specs
            .iter()
            .zip(monster_state.character_state.skill_states.iter_mut())
            .filter(|(_, s)| s.is_ready)
        {
            character_controller::use_skill(
                &skill_specs,
                skill_state,
                &mut player_state.character_state,
                &player_specs.character_specs,
            );
        }
    }
}
