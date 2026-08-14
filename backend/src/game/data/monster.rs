use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use shared::data::{
    character::CharacterSize,
    character_status::StatusId,
    modifier::Modifier,
    monster::{MonsterRarity, MonsterSpecs},
    player::CharacterSpecs,
    skill::{BaseSkillSpecs, SkillType},
};
use strum::IntoEnumIterator;

use crate::game::utils::json::LoadJsonFromFile;

use super::DataInit;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusResistanceBlueprint {
    skill_type: Option<SkillType>,
    status_id: Option<StatusId>,
    value: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseMonsterSpecs {
    #[serde(flatten)]
    pub character_specs: CharacterSpecs,
    pub skills: Vec<BaseSkillSpecs>,

    #[serde(default)]
    pub rarity: MonsterRarity,

    #[serde(default)]
    pub status_resistances: Vec<StatusResistanceBlueprint>,
}

impl DataInit<&BaseMonsterSpecs> for MonsterSpecs {
    fn init(specs: &BaseMonsterSpecs) -> Self {
        let reward_factor = match specs.character_specs.character_static.size {
            CharacterSize::Small => 1.0,
            CharacterSize::Large | CharacterSize::Tall => 2.0,
            CharacterSize::Huge => 4.0,
            CharacterSize::Gargantuan => 6.0,
        };
        let mut monster_specs = Self {
            character_specs: specs.character_specs.clone(),
            rarity: specs.rarity,
            experience_reward: reward_factor,
            gold_reward: reward_factor,
            skill_reward: reward_factor,
        };

        monster_specs
            .character_specs
            .character_attrs
            .status_resistances =
            specs
                .status_resistances
                .iter()
                .fold(HashMap::new(), |mut acc, status_resistance| {
                    let mut apply = |skill_type| {
                        acc.entry((skill_type, status_resistance.status_id.clone()))
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
