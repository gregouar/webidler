use serde::{Deserialize, Serialize};

use crate::data::player::{CharacterSpecs, CharacterState};

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

    pub rarity: MonsterRarity,
    // pub initiative: ChanceRange<f32>,
    pub power_factor: f64,
    pub reward_factor: f64, // gold_factor?
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterState {
    pub character_state: CharacterState,

    // pub initiative: f32,
    pub gold_reward: f64,
    pub gems_reward: f64,
}
