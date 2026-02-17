use serde::{Deserialize, Deserializer, Serialize};

use crate::data::{
    modifier::ModifiableValue,
    values::{BoundedValue, Luck, Percent},
};

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct Chance {
    pub value: ModifiableValue<Percent>,
    pub lucky_chance: ModifiableValue<Luck>,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct BoundedChance {
    pub value: ModifiableValue<BoundedValue<f32>>,
    pub lucky_chance: ModifiableValue<Luck>,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct ChanceRange<T> {
    pub min: T,
    pub max: T,
    pub lucky_chance: ModifiableValue<Luck>,
}

impl Chance {
    pub fn new_sure() -> Self {
        Self {
            value: Percent::new(100.0).into(),
            lucky_chance: Default::default(),
        }
    }
}

// Spicy serde
// -----------

// Chance

#[derive(Deserialize)]
#[serde(untagged)]
enum ChanceDef {
    Single(Percent),
    Full(ChanceDefFull),
}

#[derive(Deserialize)]
struct ChanceDefFull {
    pub value: Percent,
    pub lucky_chance: ModifiableValue<Luck>,
}

impl<'de> Deserialize<'de> for Chance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let full_def = if deserializer.is_human_readable() {
            match ChanceDef::deserialize(deserializer)? {
                ChanceDef::Single(value) => ChanceDefFull {
                    value,
                    lucky_chance: Default::default(),
                },
                ChanceDef::Full(full_chance_def) => full_chance_def,
            }
        } else {
            ChanceDefFull::deserialize(deserializer)?
        };

        Ok(Self {
            value: full_def.value.into(),
            lucky_chance: full_def.lucky_chance,
        })
    }
}

// BoundedChance

#[derive(Deserialize)]
#[serde(untagged)]
enum BoundedChanceDef {
    Single(BoundedValue<f32>),
    Full(BoundedChanceDefFull),
}

#[derive(Deserialize)]
struct BoundedChanceDefFull {
    pub value: BoundedValue<f32>,
    pub lucky_chance: ModifiableValue<Luck>,
}

impl<'de> Deserialize<'de> for BoundedChance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let full_def = if deserializer.is_human_readable() {
            match BoundedChanceDef::deserialize(deserializer)? {
                BoundedChanceDef::Single(value) => BoundedChanceDefFull {
                    value,
                    lucky_chance: Default::default(),
                },
                BoundedChanceDef::Full(full_chance_def) => full_chance_def,
            }
        } else {
            BoundedChanceDefFull::deserialize(deserializer)?
        };

        Ok(Self {
            value: full_def.value.into(),
            lucky_chance: full_def.lucky_chance,
        })
    }
}

// ChanceRange

#[derive(Deserialize)]
#[serde(untagged)]
enum ChanceRangeDef<T> {
    Single(T),
    Range([T; 2]),
    Full(ChanceRangeDefFull<T>),
}

#[derive(Deserialize)]
struct ChanceRangeDefFull<T> {
    min: T,
    max: T,
    #[serde(default)]
    lucky_chance: ModifiableValue<Luck>,
}

impl<'de, T: Deserialize<'de> + Copy> Deserialize<'de> for ChanceRange<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let full_def = if deserializer.is_human_readable() {
            match ChanceRangeDef::deserialize(deserializer)? {
                ChanceRangeDef::Single(value) => ChanceRangeDefFull {
                    min: value,
                    max: value,
                    lucky_chance: Default::default(),
                },
                ChanceRangeDef::Range([min, max]) => ChanceRangeDefFull {
                    min,
                    max,
                    lucky_chance: Default::default(),
                },
                ChanceRangeDef::Full(full_chance_def) => full_chance_def,
            }
        } else {
            ChanceRangeDefFull::deserialize(deserializer)?
        };

        Ok(Self {
            min: full_def.min,
            max: full_def.max,
            lucky_chance: full_def.lucky_chance,
        })
    }
}
