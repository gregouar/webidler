use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Chance {
    pub value: f32,

    #[serde(default)]
    pub lucky_chance: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ValueChance {
    pub min: f64,
    pub max: f64,

    #[serde(default)]
    pub lucky_chance: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct QuantityChance {
    pub min: u16,
    pub max: u16,

    #[serde(default)]
    pub lucky_chance: f32,
}
