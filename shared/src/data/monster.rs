use serde::{Deserialize, Serialize};

use crate::data::{
    character::CharacterAttrs,
    player::{CharacterSpecs, CharacterState},
    skill::SkillTargetsGroup,
};

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
    pub experience_reward: f64,
    pub gold_reward: f64,
    pub skill_reward: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterState {
    pub character_state: CharacterState,

    // pub initiative: f32,
    pub gold_reward: f64,
    pub gems_reward: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MonsterSpecsPatch {
    pub character_attrs: CharacterAttrs,
    pub skill_specs_targets: Vec<Vec<SkillTargetsGroup>>,
}
