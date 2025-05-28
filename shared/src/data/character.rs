use serde::{Deserialize, Serialize};

use super::character_status::{StatusMap, StatusType};
pub use super::skill::{SkillSpecs, SkillState};

use crate::serde_utils::is_default;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum CharacterId {
    Player,
    Monster(usize),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub enum CharacterSize {
    #[default]
    Small, // 1x1
    Large,      // 1x2
    Huge,       // 2x2
    Gargantuan, // 2x3
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

// TODO: Split more for network usage? -> might become an hassle to handle in code...
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterSpecs {
    pub name: String,
    pub portrait: String,

    #[serde(default, skip_serializing_if = "is_default")]
    pub size: CharacterSize,

    #[serde(default, skip_serializing_if = "is_default")]
    pub position_x: u8,
    #[serde(default, skip_serializing_if = "is_default")]
    pub position_y: u8,

    pub max_life: f64,
    #[serde(default, skip_serializing_if = "is_default")]
    pub life_regen: f64,

    #[serde(default, skip_serializing_if = "is_default")]
    pub armor: f64,
    #[serde(default, skip_serializing_if = "is_default")]
    pub fire_armor: f64,
    #[serde(default, skip_serializing_if = "is_default")]
    pub poison_armor: f64,
    #[serde(default, skip_serializing_if = "is_default")]
    pub block: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterState {
    pub health: f64,

    pub statuses: StatusMap,
    #[serde(skip_serializing, skip_deserializing)]
    pub buff_status_change: bool,

    pub is_alive: bool,
    pub just_hurt: bool,
    pub just_hurt_crit: bool,
    pub just_blocked: bool,
}

impl CharacterState {
    pub fn is_stunned(&self) -> bool {
        self.statuses.contains_key(&StatusType::Stun)
    }
}
