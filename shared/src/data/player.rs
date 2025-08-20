use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

pub use super::character::{CharacterSpecs, CharacterState};
use super::{
    item::{ItemSlot, ItemSpecs},
    skill::{SkillSpecs, SkillState},
    stat_effect::EffectsMap,
    trigger::TriggeredEffect,
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerSpecs {
    pub character_specs: CharacterSpecs,
    pub skills_specs: Vec<SkillSpecs>,
    pub auto_skills: Vec<bool>, // Should move to a separate synced struct

    pub max_skills: u8,
    pub buy_skill_cost: f64,
    pub bought_skills: HashSet<String>,

    pub level: u8,
    pub experience_needed: f64,

    // Should move to a DerivedPlayerSpecs
    pub movement_cooldown: f32,
    pub gold_find: f64,
    pub effects: EffectsMap,

    pub triggers: Vec<TriggeredEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerState {
    pub character_state: CharacterState,
    pub skills_states: Vec<SkillState>,

    pub just_leveled_up: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerResources {
    pub experience: f64,
    pub passive_points: u16,
    pub gold: f64,
    pub gems: f64,
    pub shards: f64,
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

impl PlayerInventory {
    pub fn equipped_items(&self) -> impl Iterator<Item = (ItemSlot, &Box<ItemSpecs>)> + Clone {
        self.equipped
            .iter()
            .filter_map(|(slot, equipped_slot)| match equipped_slot {
                EquippedSlot::MainSlot(item_specs) => Some((*slot, item_specs)),
                EquippedSlot::ExtraSlot(_) => None,
            })
    }
    pub fn equipped_items_mut(&mut self) -> impl Iterator<Item = (ItemSlot, &mut Box<ItemSpecs>)> {
        self.equipped
            .iter_mut()
            .filter_map(|(slot, equipped_slot)| match equipped_slot {
                EquippedSlot::MainSlot(item_specs) => Some((*slot, item_specs)),
                EquippedSlot::ExtraSlot(_) => None,
            })
    }

    pub fn all_items(&self) -> impl Iterator<Item = &ItemSpecs> + Clone {
        return self.bag.iter().chain(
            self.equipped_items()
                .map(|(_, item_specs)| item_specs.as_ref()),
        );
    }

    pub fn all_items_mut(&mut self) -> impl Iterator<Item = &mut ItemSpecs> {
        return self.bag.iter_mut().chain(
            self.equipped
                .iter_mut()
                .filter_map(|(slot, equipped_slot)| match equipped_slot {
                    EquippedSlot::MainSlot(item_specs) => Some((*slot, item_specs)),
                    EquippedSlot::ExtraSlot(_) => None,
                })
                .map(|(_, item_specs)| item_specs.as_mut()),
        );
    }
}
