use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::data::stat_effect::Modifier;

pub use super::stat_effect::StatEffect;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BenedictionSpecs {
    pub effect: StatEffect,

    pub upgrade_modifier: Modifier,
    pub upgrade_value: f64,

    pub price: f64,
    pub price_increase_factor: f64,

    #[serde(default)]
    pub max_upgrade_level: Option<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BenedictionState {
    pub upgrade_level: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerBenedictions {
    pub benedictions: HashMap<String, BenedictionState>,
}
