use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::data::{
    character_status::StatusId,
    skill::{DamageType, SkillType},
};

use super::character_status::StatusMap;
pub use super::skill::{SkillSpecs, SkillState};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterId {
    Player,
    Monster(usize),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub enum CharacterSize {
    #[default]
    Small, // 1x1
    Large,      // 1x2
    Tall,       // 2x1
    Huge,       // 2x2
    Gargantuan, // 2x3
}

impl CharacterSize {
    pub fn get_xy_size(&self) -> (usize, usize) {
        match self {
            CharacterSize::Small => (1, 1),
            CharacterSize::Large => (2, 1),
            CharacterSize::Tall => (1, 2),
            CharacterSize::Huge => (2, 2),
            CharacterSize::Gargantuan => (3, 2),
        }
    }
}

// TODO: Split more for network usage? -> might become an hassle to handle in code...
// But I think I want it. We would have the "base specs (still updated by passives and skills)"
// and an "computed stats".
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

    pub max_life: f64,
    #[serde(default)]
    pub life_regen: f64,

    #[serde(default)]
    pub max_mana: f64,
    #[serde(default)]
    pub mana_regen: f64,

    #[serde(default)]
    pub take_from_mana_before_life: f32,

    #[serde(default)]
    pub armor: HashMap<DamageType, f64>,
    #[serde(default)]
    pub block: f32,
    #[serde(default)]
    pub block_spell: f32,
    #[serde(default)]
    pub block_damage: f32,

    #[serde(default)]
    pub damage_resistance: HashMap<(SkillType, DamageType), f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CharacterState {
    pub life: f64,
    pub mana: f64,

    pub statuses: StatusMap,
    // This feels dirty
    #[serde(default, skip_serializing, skip_deserializing)]
    pub buff_status_change: bool,

    pub is_alive: bool,
    pub just_hurt: bool,
    pub just_hurt_crit: bool,
    pub just_blocked: bool,
}

impl CharacterState {
    pub fn is_stunned(&self) -> bool {
        self.statuses.unique_statuses.contains_key(&StatusId::Stun)
    }
}
