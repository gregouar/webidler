use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct Chance {
    pub value: f32,

    #[serde(default)]
    pub lucky_chance: f32,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct ValueChance {
    pub min: f64,
    pub max: f64,

    #[serde(default)]
    pub lucky_chance: f32,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct QuantityChance {
    pub min: u16,
    pub max: u16,

    #[serde(default)]
    pub lucky_chance: f32,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ChanceDef {
    Single(f32),
    Full(Chance),
}

impl<'de> Deserialize<'de> for Chance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match ChanceDef::deserialize(deserializer)? {
            ChanceDef::Single(value) => Self {
                value,
                lucky_chance: 0.0,
            },
            ChanceDef::Full(chance) => chance,
        })
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RangeChanceDef<T> {
    Single(T),
    Range([T; 2]),
    Full {
        min: T,
        max: T,
        #[serde(default)]
        lucky_chance: f32,
    },
}

impl<'de> Deserialize<'de> for ValueChance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match RangeChanceDef::deserialize(deserializer)? {
            RangeChanceDef::Single(value) => Self {
                min: value,
                max: value,
                lucky_chance: 0.0,
            },
            RangeChanceDef::Range([min, max]) => Self {
                min,
                max,
                lucky_chance: 0.0,
            },
            RangeChanceDef::Full {
                min,
                max,
                lucky_chance,
            } => Self {
                min,
                max,
                lucky_chance,
            },
        })
    }
}

impl<'de> Deserialize<'de> for QuantityChance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match RangeChanceDef::deserialize(deserializer)? {
            RangeChanceDef::Single(value) => Self {
                min: value,
                max: value,
                lucky_chance: 0.0,
            },
            RangeChanceDef::Range([min, max]) => Self {
                min,
                max,
                lucky_chance: 0.0,
            },
            RangeChanceDef::Full {
                min,
                max,
                lucky_chance,
            } => Self {
                min,
                max,
                lucky_chance,
            },
        })
    }
}
