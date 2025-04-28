use serde::{Deserialize, Serialize};

pub use super::character::{CharacterSpecs, CharacterState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MonsterSize {
    Small,      // 1x1
    Large,      // 1x2
    Giant,      // 2x2
    Gargantuan, // 2x3
}

impl Default for MonsterSize {
    fn default() -> Self {
        MonsterSize::Small
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterSpecs {
    pub character_specs: CharacterSpecs,

    #[serde(default)]
    pub size: MonsterSize,

    pub max_initiative: f32,
    pub power_factor: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterState {
    pub character_state: CharacterState,

    pub initiative: f32,
}
