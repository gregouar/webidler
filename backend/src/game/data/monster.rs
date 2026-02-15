use serde::{Deserialize, Serialize};

use shared::data::{
    chance::ChanceRange,
    monster::{MonsterRarity, MonsterSpecs},
    player::CharacterSpecs,
    skill::{BaseSkillSpecs, SkillSpecs},
};

use crate::game::utils::json::LoadJsonFromFile;

use super::DataInit;

// struct StatusResistanceBlueprint {
//     skill_type: Option<SkillType>,
//     status_type: Option<StatStatusType>,
//     value: f64,
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseMonsterSpecs {
    #[serde(flatten)]
    pub character_specs: CharacterSpecs,
    pub skills: Vec<BaseSkillSpecs>,

    #[serde(default)]
    pub rarity: MonsterRarity,

    pub initiative: ChanceRange<f32>,
    pub power_factor: f64,
    // TODO:
    // pub status_resistances: Vec<StatusResistanceBlueprint>,
}

impl DataInit<BaseMonsterSpecs> for MonsterSpecs {
    fn init(specs: BaseMonsterSpecs) -> Self {
        Self {
            character_specs: specs.character_specs,
            skill_specs: specs.skills.iter().cloned().map(SkillSpecs::init).collect(),
            rarity: specs.rarity,
            initiative: specs.initiative,
            power_factor: specs.power_factor,
            reward_factor: specs.power_factor,
        }
    }
}

impl LoadJsonFromFile for BaseMonsterSpecs {}
