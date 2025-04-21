use rand::Rng;
use shared::data::{MonsterPrototype, MonsterState, PlayerPrototype, PlayerState};

use super::character_controller;

pub struct PlayerController {
    pub auto_skills: Vec<bool>,
    pub use_skills: Vec<usize>,
}

impl PlayerController {
    pub fn init(prototype: &PlayerPrototype) -> Self {
        PlayerController {
            auto_skills: prototype.auto_skills.clone(),
            use_skills: Vec::with_capacity(prototype.character_prototype.skill_prototypes.len()),
        }
    }

    pub fn reset(&mut self) {
        self.use_skills.clear();
    }

    pub fn control_player(
        &mut self,
        player_prototype: &PlayerPrototype,
        player_state: &mut PlayerState,
        monsters: &mut Vec<(&mut MonsterState, &MonsterPrototype)>,
    ) {
        if !player_state.character_state.is_alive || monsters.is_empty() {
            return;
        }

        let mut rng = rand::rng();

        for (i, (skill_prototype, skill_state)) in player_prototype
            .character_prototype
            .skill_prototypes
            .iter()
            .zip(player_state.character_state.skill_states.iter_mut())
            .enumerate()
        {
            if !skill_state.is_ready || skill_prototype.mana_cost > player_state.mana {
                continue;
            }

            if !self.auto_skills.get(i).unwrap_or(&false) && !self.use_skills.contains(&i) {
                continue;
            }

            // TODO: depending on distance, choose target
            let j = rng.random_range(0..monsters.len());
            if let Some((target_state, target_prototype)) = monsters.get_mut(j).as_deref_mut() {
                player_state.mana -= skill_prototype.mana_cost;
                character_controller::use_skill(
                    skill_prototype,
                    skill_state,
                    &mut target_state.character_state,
                    &target_prototype.character_prototype,
                );
            }
        }
    }
}
