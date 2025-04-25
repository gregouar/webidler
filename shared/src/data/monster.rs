use serde::{Deserialize, Serialize};

pub use super::character::{CharacterSpecs, CharacterState};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterSpecs {
    pub character_specs: CharacterSpecs,

    pub max_initiative: f32,
    pub power_factor: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterState {
    pub character_state: CharacterState,

    pub initiative: f32,
}
