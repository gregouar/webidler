use serde::{Deserialize, Serialize};

pub use super::character::{CharacterSpecs, CharacterState};
use super::{
    item::ItemSpecs,
    skill::{SkillSpecs, SkillState},
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerSpecs {
    pub character_specs: CharacterSpecs,
    pub skills_specs: Vec<SkillSpecs>,

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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerInventory {
    pub trinket_specs: Option<ItemSpecs>,
    pub helmet_specs: Option<ItemSpecs>,
    pub amulet_specs: Option<ItemSpecs>,
    pub weapon_specs: Option<ItemSpecs>,
    pub body_specs: Option<ItemSpecs>,
    pub shield_specs: Option<ItemSpecs>,
    pub gloves_specs: Option<ItemSpecs>,
    pub boots_specs: Option<ItemSpecs>,
    pub ring_specs: Option<ItemSpecs>,

    pub bag: Vec<ItemSpecs>,
    pub max_bag_size: u8,
}
