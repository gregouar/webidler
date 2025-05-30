use serde::{Deserialize, Serialize};

pub use super::character::{CharacterSpecs, CharacterState};
use super::skill::{SkillSpecs, SkillState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MonsterSpecs {
    pub character_specs: CharacterSpecs,
    pub skill_specs: Vec<SkillSpecs>,

    pub min_initiative: f32,
    pub max_initiative: f32,
    pub power_factor: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterState {
    pub character_state: CharacterState,
    pub skill_states: Vec<SkillState>,

    pub initiative: f32,
    pub gold_reward: f64,
}
