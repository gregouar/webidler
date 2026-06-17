use std::collections::HashMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::data::{modifier::Modifier, stat_effect::StatType};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SkillMasterySpecs {
    pub max_level: u16,
    pub upgrades: IndexMap<String, SkillMasteryUpgrade>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SkillMasteryUpgrade {
    pub title: String,
    pub base_cost: u16,
    pub upgrade_cost: u16,
    pub max_level: u16,
    pub value: f64,
    pub upgrade_value: f64,
    pub effect: SkillMasteryUpgradeEffect,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SkillMasteryUpgradeEffect {
    StatEffect { stat: StatType, modifier: Modifier },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PlayerSkillMasteries {
    pub masteries: IndexMap<String, SkillMasteryState>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SkillMasteryState {
    pub experience: f64,
    pub upgrades_bought: HashMap<String, u16>,
}
