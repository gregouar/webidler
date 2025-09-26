use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct Chance {
    pub value: f32,

    #[serde(default)]
    pub lucky_chance: f32,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct ChanceRange<T> {
    pub min: T,
    pub max: T,

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
enum ChanceRangeDef<T> {
    Single(T),
    Range([T; 2]),
    Full {
        min: T,
        max: T,
        #[serde(default)]
        lucky_chance: f32,
    },
}

impl<'de, T: Deserialize<'de> + Copy> Deserialize<'de> for ChanceRange<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match ChanceRangeDef::<T>::deserialize(deserializer)? {
            ChanceRangeDef::Single(value) => Self {
                min: value,
                max: value,
                lucky_chance: 0.0,
            },
            ChanceRangeDef::Range([min, max]) => Self {
                min,
                max,
                lucky_chance: 0.0,
            },
            ChanceRangeDef::Full {
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
