use serde::{Deserialize, Serialize};

pub use super::character::{CharacterSpecs, CharacterState};
use super::item::ItemSpecs;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerSpecs {
    pub character_specs: CharacterSpecs,

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

    // TODO: Should this be in PlayerResources?
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
}
