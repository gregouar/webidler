use shared::data::{
    monster::{MonsterSpecs, MonsterState},
    player::{PlayerResources, PlayerSpecs, PlayerState},
};

use super::skills_controller;

pub struct PlayerController {
    pub auto_skills: Vec<bool>,
    pub use_skills: Vec<usize>,
}

impl PlayerController {
    pub fn init(specs: &PlayerSpecs) -> Self {
        PlayerController {
            auto_skills: specs.auto_skills.clone(),
            use_skills: Vec::with_capacity(specs.skill_specs.len()),
        }
    }

    pub fn reset(&mut self) {
        self.use_skills.clear();
    }

    pub fn control_player(
        &mut self,
        player_specs: &PlayerSpecs,
        player_state: &mut PlayerState,
        monsters: &mut Vec<(&MonsterSpecs, &mut MonsterState)>,
    ) {
        if !player_state.character_state.is_alive {
            return;
        }

        for (i, (skill_specs, skill_state)) in player_specs
            .skill_specs
            .iter()
            .zip(player_state.skill_states.iter_mut())
            .enumerate()
        {
            if !skill_state.is_ready || skill_specs.mana_cost > player_state.mana {
                continue;
            }

            if !self.auto_skills.get(i).unwrap_or(&false) && !self.use_skills.contains(&i) {
                continue;
            }

            if skills_controller::use_skill(
                skill_specs,
                skill_state,
                (
                    &player_specs.character_specs,
                    &mut player_state.character_state,
                ),
                vec![],
                monsters
                    .iter_mut()
                    .map(|(specs, state)| (&specs.character_specs, &mut state.character_state))
                    .collect(),
            ) {
                player_state.mana -= skill_specs.mana_cost;
            }
        }
    }
}

pub fn reward_player(
    player_state: &mut PlayerState,
    player_resources: &mut PlayerResources,
    monster_specs: &MonsterSpecs,
) {
    player_resources.gold += monster_specs.power_factor;
    player_state.experience += monster_specs.power_factor;
}
