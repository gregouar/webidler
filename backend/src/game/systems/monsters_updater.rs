use std::time::Duration;

use shared::data::{MonsterPrototype, MonsterState};

use super::characters_updater;

pub fn update_monster_states(
    elapsed_time: Duration,
    monster_prototypes: &Vec<MonsterPrototype>,
    monster_states: &mut Vec<MonsterState>,
) {
    for (monster_state, monster_prototype) in monster_states
        .iter_mut()
        .zip(monster_prototypes.iter())
        .filter(|(s, _)| s.character_state.is_alive)
    {
        monster_state.initiative = (monster_state.initiative - elapsed_time.as_secs_f32()).max(0.0);
        if monster_state.initiative > 0.0 {
            continue;
        }

        characters_updater::update_character_state(
            elapsed_time,
            &monster_prototype.character_prototype,
            &mut monster_state.character_state,
        );
    }
}
