use serde::{Deserialize, Serialize};

use crate::data::{chance::ValueChance, trigger::TriggeredEffect};

pub use super::character::{CharacterSpecs, CharacterState};
use super::skill::{SkillSpecs, SkillState};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MonsterRarity {
    #[default]
    Normal,
    Champion,
    Boss,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MonsterSpecs {
    pub character_specs: CharacterSpecs,
    pub skill_specs: Vec<SkillSpecs>,

    pub rarity: MonsterRarity,
    pub initiative: ValueChance,
    pub power_factor: f64,
    pub reward_factor: f64,

    #[serde(default)]
    pub triggers: Vec<TriggeredEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterState {
    pub character_state: CharacterState,
    pub skill_states: Vec<SkillState>,

    pub initiative: f32,
    pub gold_reward: f64,
    pub gems_reward: f64,
}
