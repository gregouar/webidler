use serde::{Deserialize, Serialize};

use crate::data::{
    character::CharacterAttrs,
    player::{CharacterSpecs, CharacterState},
    skill::SkillTargetsGroup,
    values::NonNegative,
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
    pub experience_reward: NonNegative,
    pub gold_reward: NonNegative,
    pub skill_reward: NonNegative,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterState {
    pub character_state: CharacterState,

    // pub initiative: f32,
    pub gold_reward: NonNegative,
    pub gems_reward: NonNegative,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MonsterSpecsPatch {
    pub character_attrs: CharacterAttrs,
    pub skill_specs_targets: Vec<Vec<SkillTargetsGroup>>,
}
