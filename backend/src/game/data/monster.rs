use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use shared::data::{
    chance::ChanceRange,
    modifier::Modifier,
    monster::{MonsterRarity, MonsterSpecs},
    player::CharacterSpecs,
    skill::{BaseSkillSpecs, SkillSpecs, SkillType},
    stat_effect::StatStatusType,
};
use strum::IntoEnumIterator;

use crate::game::utils::json::LoadJsonFromFile;

use super::DataInit;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusResistanceBlueprint {
    skill_type: Option<SkillType>,
    status_type: Option<StatStatusType>,
    value: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseMonsterSpecs {
    #[serde(flatten)]
    pub character_specs: CharacterSpecs,
    pub skills: Vec<BaseSkillSpecs>,

    #[serde(default)]
    pub rarity: MonsterRarity,

    pub initiative: ChanceRange<f32>,
    pub power_factor: f64,

    #[serde(default)]
    pub status_resistances: Vec<StatusResistanceBlueprint>,
}

impl DataInit<BaseMonsterSpecs> for MonsterSpecs {
    fn init(specs: BaseMonsterSpecs) -> Self {
        let mut monster_specs = Self {
            character_specs: specs.character_specs,
            skill_specs: specs.skills.iter().cloned().map(SkillSpecs::init).collect(),
            rarity: specs.rarity,
            initiative: specs.initiative,
            power_factor: specs.power_factor,
            reward_factor: specs.power_factor,
        };

        monster_specs.character_specs.status_resistances = specs
            .status_resistances
            .into_iter()
            .fold(HashMap::new(), |mut acc, status_resistance| {
                let mut apply = |skill_type| {
                    acc.entry((skill_type, status_resistance.status_type.clone()))
                        .or_default()
                        .apply_modifier(status_resistance.value, Modifier::Flat);
                };

                if let Some(skill_type) = status_resistance.skill_type {
                    apply(skill_type);
                } else {
                    for skill_type in SkillType::iter() {
                        apply(skill_type);
                    }
                }
                acc
            });

        monster_specs
    }
}

impl LoadJsonFromFile for BaseMonsterSpecs {}
