use serde::{Deserialize, Serialize};

pub use super::character::{CharacterSpecs, CharacterState};
use super::{
    item::ItemSpecs,
    skill::{SkillSpecs, SkillState},
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerSpecs {
    pub character_specs: CharacterSpecs,
    pub skill_specs: Vec<SkillSpecs>,

    pub level: u8,
    pub experience_needed: f64,

    pub max_mana: f64,
    pub mana_regen: f64,

    pub auto_skills: Vec<bool>,

    pub inventory: PlayerInventory,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerState {
    pub character_state: CharacterState,
    pub skill_states: Vec<SkillState>,

    // TODO: move to PlayerResources
    pub experience: f64,

    pub mana: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerResources {
    pub gold: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerInventory {
    pub weapon_specs: Option<ItemSpecs>, // How to ensure it's a weapon?
    pub bag: Vec<ItemSpecs>,
    pub max_bag_size: u8,
}
