use std::collections::HashMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    computations,
    constants::{SKILL_MASTERY_BASE_COST, XP_INCREASE_FACTOR},
    data::{
        modifier::Modifier,
        stat_effect::{StatEffect, StatType},
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
        (SKILL_MASTERY_BASE_COST * computations::exponential(self.level() + 1, XP_INCREASE_FACTOR))
            .round()
    }

    pub fn relative_experience(&self) -> f64 {
        self.experience - self.level_cost()
    }

    pub fn level_cost(&self) -> f64 {
        (0..self.level())
            .map(|level| {
                SKILL_MASTERY_BASE_COST * computations::exponential(level + 1, XP_INCREASE_FACTOR)
            })
            .sum::<f64>()
            .round()
    }

    pub fn level(&self) -> u16 {
        let mut level = 0u16;
        let mut remaining_experience = self.experience;

        loop {
            let next_level_cost = (SKILL_MASTERY_BASE_COST
                * computations::exponential(level + 1, XP_INCREASE_FACTOR))
            .round();
            if remaining_experience < next_level_cost || level == u16::MAX {
                return level;
            }

            remaining_experience -= next_level_cost;
            level = level.saturating_add(1);
        }
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

    pub fn compute_value(&self, upgrade_level: u16) -> Option<f64> {
        if upgrade_level == 0 {
            return None;
        }

        Some(self.value + upgrade_level.saturating_sub(1) as f64 * self.upgrade_value)
    }

    pub fn compute_stat_effect(&self, upgrade_level: u16) -> Option<StatEffect> {
        match self.effect.clone() {
            SkillMasteryUpgradeEffect::StatEffect { stat, modifier } => {
                self.compute_value(upgrade_level).map(|value| StatEffect {
                    stat,
                    modifier,
                    value,
                    bypass_ignore: false,
                })
            }
        }
    }
}
