use serde::{Deserialize, Serialize};

pub use super::skill::{SkillSpecs, SkillState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CharacterSize {
    Small,      // 1x1
    Large,      // 1x2
    Huge,       // 2x2
    Gargantuan, // 2x3
}

impl Default for CharacterSize {
    fn default() -> Self {
        CharacterSize::Small
    }
}

impl CharacterSize {
    pub fn get_xy_size(&self) -> (usize, usize) {
        match self {
            CharacterSize::Small => (1, 1),
            CharacterSize::Large => (2, 1),
            CharacterSize::Huge => (2, 2),
            CharacterSize::Gargantuan => (3, 2),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterSpecs {
    pub name: String,
    pub portrait: String,

    #[serde(default)]
    pub size: CharacterSize,

    #[serde(default)]
    pub position_x: u8,
    #[serde(default)]
    pub position_y: u8,

    pub max_health: f64,
    #[serde(default)]
    pub health_regen: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterState {
    pub is_alive: bool,
    pub just_hurt: bool,
    pub health: f64,
}
