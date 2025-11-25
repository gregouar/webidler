use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub use super::stat_effect::{Modifier, StatEffect, StatType};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BenedictionSpecs {
    pub value: f64,
    pub effect: BenedictionEffect,

    pub upgrade_modifier: Modifier,
    pub upgrade_value: f64,

    pub price: f64,
    pub price_increase_factor: f64,

    #[serde(default)]
    pub max_upgrade_level: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum BenedictionEffect {
    StartingGold,
    StartingLevel,
    StatEffect { stat: StatType, modifier: Modifier },
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BenedictionState {
    pub upgrade_level: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PlayerBenedictions {
    pub purchased_benedictions: HashMap<String, BenedictionState>,
}

impl BenedictionSpecs {
    pub fn compute_value(&self, upgrade_level: u64) -> Option<f64> {
        let mut value = self.value;

        if upgrade_level == 0
            || self
                .max_upgrade_level
                .map(|max_upgrade_level| upgrade_level == max_upgrade_level)
                .unwrap_or_default()
        {
            return None;
        }

        match self.upgrade_modifier {
            Modifier::Multiplier => todo!(),
            Modifier::Flat => value += upgrade_level.saturating_sub(1) as f64 * self.upgrade_value,
        }

        Some(value)
    }

    pub fn compute_stat_effect(&self, upgrade_level: u64) -> Option<StatEffect> {
        if let BenedictionEffect::StatEffect { stat, modifier } = self.effect.clone() {
            self.compute_value(upgrade_level).map(|value| StatEffect {
                stat,
                modifier,
                value,
                bypass_ignore: false,
            })
        } else {
            None
        }
    }

    pub fn compute_price(&self, upgrade_level: u64) -> f64 {
        self.price * self.price_increase_factor.powi(upgrade_level as i32)
    }

    pub fn compute_total_price(&self, upgrade_level: u64) -> f64 {
        (0..upgrade_level)
            .map(|level| self.compute_price(level))
            .sum()
    }
}
