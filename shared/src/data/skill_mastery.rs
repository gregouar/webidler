use std::collections::HashMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    computations,
    data::{
        modifier::Modifier,
        skill::SkillEffect,
        stat_effect::{StatEffect, StatType},
        trigger::TriggerSpecs,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
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
    pub effects: Vec<SkillMasteryUpgradeEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SkillMasteryUpgradeEffect {
    pub value: f64,
    pub upgrade_value: f64,
    #[serde(flatten)]
    pub effect_type: SkillMasteryUpgradeEffectType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SkillMasteryUpgradeEffectType {
    StatEffect {
        stat: StatType,
        modifier: Modifier,
    },
    SkillEffect {
        #[serde(flatten)]
        skill_effect: SkillEffect,
        #[serde(default)]
        target_index: usize,
    },
    Trigger(TriggerSpecs),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PlayerSkillMasteries {
    pub masteries: IndexMap<String, SkillMasteryState>,
    #[serde(default)]
    pub favorite_skills: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SkillMasteryState {
    pub experience: f64,
    pub upgrades_bought: HashMap<String, u16>,
}

impl SkillMasteryState {
    pub fn next_level_cost(&self) -> f64 {
        computations::skill_mastery_next_level_cost(self.level())
    }

    pub fn relative_experience(&self) -> f64 {
        self.experience - self.level_cost()
    }

    pub fn level_cost(&self) -> f64 {
        computations::skill_mastery_level_cost(self.level())
    }

    pub fn level(&self) -> u16 {
        computations::skill_mastery_level(self.experience)
    }
}
impl SkillMasteryUpgrade {
    pub fn compute_cost(&self, upgrade_level: u16) -> u16 {
        if upgrade_level == 0 {
            return 0;
        }

        self.base_cost.saturating_add(
            upgrade_level
                .saturating_sub(1)
                .saturating_mul(self.upgrade_cost),
        )
    }
}

impl SkillMasteryUpgradeEffect {
    pub fn compute_value(&self, upgrade_level: u16) -> Option<f64> {
        if upgrade_level == 0 {
            return None;
        }

        Some(self.value + upgrade_level.saturating_sub(1) as f64 * self.upgrade_value)
    }

    pub fn compute_stat_effect(&self, upgrade_level: u16) -> Option<StatEffect> {
        let Some(upgrade_value) = self.compute_value(upgrade_level) else {
            return None;
        };
        match &self.effect_type {
            SkillMasteryUpgradeEffectType::StatEffect { stat, modifier } => Some(StatEffect {
                stat: stat.clone(),
                modifier: *modifier,
                value: upgrade_value,
                bypass_ignore: false,
            }),
            _ => None,
        }
    }
}
