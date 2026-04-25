use std::collections::HashMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::data::{
    modifier::Modifier,
    stat_effect::{StatEffect, StatType},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BenedictionsCategory {
    pub title: String,

    pub price: f64,
    pub price_increase_factor: f64,
    pub max_upgrade_level: Option<u64>,

    pub benedictions: IndexMap<String, BenedictionSpecs>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BenedictionSpecs {
    pub value: f64,
    pub effect: BenedictionEffect,

    pub upgrade_modifier: Modifier,
    pub upgrade_value: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum BenedictionEffect {
    StartingGold,
    StartingLevel,
    StatEffect { stat: StatType, modifier: Modifier },
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct PlayerBenedictions {
    pub categories: IndexMap<String, PlayerBenedictionsCategory>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct PlayerBenedictionsCategory {
    pub upgrade_level: u64,
    pub purchased_benedictions: HashMap<String, u64>,
}

impl BenedictionsCategory {
    pub fn compute_price(&self, upgrade_level: u64) -> f64 {
        self.price * self.price_increase_factor.powi(upgrade_level as i32)
    }

    pub fn compute_total_price(&self, upgrade_level: u64) -> f64 {
        (0..upgrade_level)
            .map(|level| self.compute_price(level))
            .sum()
    }
}

impl BenedictionSpecs {
    pub fn compute_value(&self, upgrade_level: u64) -> Option<f64> {
        let mut value = self.value;

        if upgrade_level == 0 {
            return None;
        }

        match self.upgrade_modifier {
            // TODO
            Modifier::Increased | Modifier::More => {
                let exponent = upgrade_level.saturating_sub(1) as i32;
                value *= (1.0 + self.upgrade_value * 0.01).powi(exponent);
            }
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
}
