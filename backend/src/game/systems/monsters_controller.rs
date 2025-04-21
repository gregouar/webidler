use shared::data::{MonsterPrototype, MonsterState, PlayerPrototype, PlayerState};

use super::character_controller;

pub fn control_monsters(
    monsters: &mut Vec<(&mut MonsterState, &MonsterPrototype)>,
    player_prototype: &PlayerPrototype,
    player_state: &mut PlayerState,
) {
    if !player_state.character_state.is_alive {
        return;
    }

    for (monster_state, monster_prototype) in monsters
        .iter_mut()
        .filter(|(m, _)| m.character_state.is_alive && m.initiative == 0.0)
    {
        for (skill_prototype, skill_state) in monster_prototype
            .character_prototype
            .skill_prototypes
            .iter()
            .zip(monster_state.character_state.skill_states.iter_mut())
            .filter(|(_, s)| s.is_ready)
        {
            character_controller::use_skill(
                &skill_prototype,
                skill_state,
                &mut player_state.character_state,
                &player_prototype.character_prototype,
            );
        }
    }
}
