use serde::{Deserialize, Serialize};

pub use super::skill::{SkillSpecs, SkillState};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterSpecs {
    pub name: String,
    pub portrait: String,

    #[serde(default)]
    pub position_x: u8,
    #[serde(default)]
    pub position_y: u8,

    pub max_health: f64,
    #[serde(default)]
    pub health_regen: f64,

    pub skill_specs: Vec<SkillSpecs>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterState {
    pub is_alive: bool,
    pub just_hurt: bool,
    pub health: f64,

    pub skill_states: Vec<SkillState>,
}
