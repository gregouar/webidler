use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub use super::character::{CharacterSpecs, CharacterState};
use super::{
    item::{ItemSlot, ItemSpecs},
    skill::{SkillSpecs, SkillState},
    stat_effect::EffectsMap,
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerSpecs {
    pub character_specs: CharacterSpecs,
    pub skills_specs: Vec<SkillSpecs>,

    pub level: u8,
    pub experience_needed: f64,

    pub max_mana: f64,
    pub mana_regen: f64,
    pub movement_cooldown: f32,

    pub gold_find: f64,
    pub effects: EffectsMap,

    pub auto_skills: Vec<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerState {
    pub character_state: CharacterState,
    pub skills_states: Vec<SkillState>,

    pub mana: f64,

    pub just_leveled_up: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerResources {
    pub experience: f64,
    pub passive_points: u16,
    pub gold: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EquippedSlot {
    MainSlot(Box<ItemSpecs>),
    ExtraSlot(ItemSlot), // Link to main slot taking the extra slot
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerInventory {
    pub equipped: HashMap<ItemSlot, EquippedSlot>,

    pub bag: Vec<ItemSpecs>,
    pub max_bag_size: u8,
}
