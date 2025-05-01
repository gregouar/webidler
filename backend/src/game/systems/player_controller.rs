use shared::data::{
    monster::{MonsterSpecs, MonsterState},
    player::{PlayerResources, PlayerSpecs, PlayerState},
};

use super::{increase_factors::exponential_factor, skills_controller};

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

pub fn reward_player(player_resources: &mut PlayerResources, monster_specs: &MonsterSpecs) {
    player_resources.gold += monster_specs.power_factor;
    player_resources.experience += monster_specs.power_factor;
}

pub fn level_up(
    player_specs: &mut PlayerSpecs,
    player_state: &mut PlayerState,
    player_resources: &mut PlayerResources,
) -> bool {
    if player_resources.experience < player_specs.experience_needed {
        return false;
    }

    player_specs.level += 1;
    player_resources.experience -= player_specs.experience_needed;
    player_specs.experience_needed = 10.0 * exponential_factor(player_specs.level as f64);

    player_state.character_state.health += 10.0;
    player_specs.character_specs.max_health += 10.0;

    true
}
