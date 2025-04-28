use shared::data::{
    monster::{MonsterSpecs, MonsterState},
    player::{PlayerResources, PlayerSpecs, PlayerState},
};

use crate::rng;

use super::character_controller;

pub struct PlayerController {
    pub auto_skills: Vec<bool>,
    pub use_skills: Vec<usize>,
}

impl PlayerController {
    pub fn init(specs: &PlayerSpecs) -> Self {
        PlayerController {
            auto_skills: specs.auto_skills.clone(),
            use_skills: Vec::with_capacity(specs.character_specs.skill_specs.len()),
        }
    }

    pub fn reset(&mut self) {
        self.use_skills.clear();
    }

    pub fn control_player(
        &mut self,
        player_specs: &PlayerSpecs,
        player_state: &mut PlayerState,
        player_resources: &mut PlayerResources,
        monsters: &mut Vec<(&MonsterSpecs, &mut MonsterState)>,
    ) {
        if !player_state.character_state.is_alive {
            return;
        }

        // let mut rewards = Vec::new();
        for (i, (skill_specs, skill_state)) in player_specs
            .character_specs
            .skill_specs
            .iter()
            .zip(player_state.character_state.skill_states.iter_mut())
            .enumerate()
        {
            if !skill_state.is_ready || skill_specs.mana_cost > player_state.mana {
                continue;
            }

            if !self.auto_skills.get(i).unwrap_or(&false) && !self.use_skills.contains(&i) {
                continue;
            }

            // TODO: depending on distance, choose target
            // let j = rng::random_range(0..monsters.len()).unwrap_or_default();
            // if let Some((target_specs, target_state)) = monsters.get_mut(j).as_deref_mut() {
            // player_state.mana -= skill_specs.mana_cost;
            if character_controller::use_skill(
                skill_specs,
                skill_state,
                &mut monsters
                    .iter_mut()
                    .map(|(specs, state)| (&specs.character_specs, &mut state.character_state))
                    .collect(),
            ) {
                player_state.mana -= skill_specs.mana_cost;
                // rewards.push(*target_specs);
            }
            // }
        }

        // for reward in rewards {
        //     reward_player(player_state, player_resources, &reward);
        // }
    }
}

fn reward_player(
    player_state: &mut PlayerState,
    player_resources: &mut PlayerResources,
    target_specs: &MonsterSpecs,
) {
    player_resources.gold += target_specs.power_factor;
    player_state.experience += target_specs.power_factor;
}
