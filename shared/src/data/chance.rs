use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct Chance {
    pub value: f32,
    pub lucky_chance: f32,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct ChanceRange<T> {
    pub min: T,
    pub max: T,
    pub lucky_chance: f32,
}

impl Chance {
    pub fn new_sure() -> Self {
        Self {
            value: 100.0,
            lucky_chance: 0.0,
        }
    }
}

// Spicy serde

#[derive(Deserialize)]
#[serde(untagged)]
enum ChanceDef {
    Single(f32),
    Full(ChanceDefFull),
}

#[derive(Deserialize)]
struct ChanceDefFull {
    pub value: f32,
    pub lucky_chance: f32,
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
                    lucky_chance: 0.0,
                },
                ChanceDef::Full(full_chance_def) => full_chance_def,
            }
        } else {
            ChanceDefFull::deserialize(deserializer)?
        };

        Ok(Self {
            value: full_def.value,
            lucky_chance: full_def.lucky_chance,
        })
    }
}

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
    lucky_chance: f32,
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
                    lucky_chance: 0.0,
                },
                ChanceRangeDef::Range([min, max]) => ChanceRangeDefFull {
                    min,
                    max,
                    lucky_chance: 0.0,
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

// use serde::{Deserialize, Serialize};

// #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
// #[serde(from = "ChanceDef", into = "ChanceDef")]
// pub struct Chance {
//     pub value: f32,
//     pub lucky_chance: f32,
// }

// #[derive(Serialize, Deserialize)]
// #[serde(untagged)]
// enum ChanceDef {
//     Full { value: f32, lucky_chance: f32 },
//     Single(f32),
// }

// impl From<ChanceDef> for Chance {
//     fn from(value: ChanceDef) -> Self {
//         match value {
//             ChanceDef::Single(value) => Self {
//                 value,
//                 lucky_chance: 0.0,
//             },
//             ChanceDef::Full {
//                 value,
//                 lucky_chance,
//             } => Self {
//                 value,
//                 lucky_chance,
//             },
//         }
//     }
// }

// impl From<Chance> for ChanceDef {
//     fn from(value: Chance) -> Self {
//         ChanceDef::Full {
//             value: value.value,
//             lucky_chance: value.lucky_chance,
//         }
//     }
// }

// #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
// #[serde(from = "ChanceRangeDef<T>")]
// pub struct ChanceRange<T: Copy> {
//     pub min: T,
//     pub max: T,
//     pub lucky_chance: f32,
// }

// #[derive(Deserialize)]
// #[serde(untagged)]
// enum ChanceRangeDef<T> {
//     Full {
//         min: T,
//         max: T,
//         #[serde(default)]
//         lucky_chance: f32,
//     },
//     Range([T; 2]),
//     Single(T),
// }

// impl<T: Copy> From<ChanceRangeDef<T>> for ChanceRange<T> {
//     fn from(def: ChanceRangeDef<T>) -> Self {
//         match def {
//             ChanceRangeDef::Single(value) => Self {
//                 min: value,
//                 max: value,
//                 lucky_chance: 0.0,
//             },
//             ChanceRangeDef::Range([min, max]) => Self {
//                 min,
//                 max,
//                 lucky_chance: 0.0,
//             },
//             ChanceRangeDef::Full {
//                 min,
//                 max,
//                 lucky_chance,
//             } => Self {
//                 min,
//                 max,
//                 lucky_chance,
//             },
//         }
//     }
// }
