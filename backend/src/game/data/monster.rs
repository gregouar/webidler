use serde::{Deserialize, Serialize};

use shared::data::{
    monster::MonsterSpecs,
    player::CharacterSpecs,
    skill::{BaseSkillSpecs, SkillSpecs},
};

use crate::game::utils::json::LoadJsonFromFile;

use super::DataInit;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseMonsterSpecs {
    #[serde(flatten)]
    pub character_specs: CharacterSpecs,
    pub skills: Vec<BaseSkillSpecs>,

    pub max_initiative: f32,
    pub power_factor: f64,
}

impl DataInit<BaseMonsterSpecs> for MonsterSpecs {
    fn init(specs: &BaseMonsterSpecs) -> Self {
        Self {
            character_specs: specs.character_specs.clone(),
            skill_specs: specs.skills.iter().map(SkillSpecs::init).collect(),
            max_initiative: specs.max_initiative,
            power_factor: specs.power_factor,
        }
    }
}

impl LoadJsonFromFile for BaseMonsterSpecs {}
