use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::data::{
    area::AreaLevel,
    character::{CharacterAttrs, CharacterStatic},
    skill::BaseSkillSpecs,
    skill_mastery::PlayerSkillMasteries,
    stat_effect::StatEffect,
    trigger::TriggerEffect,
    values::{AtLeastOne, NonNegative},
};

pub use super::character::{CharacterSpecs, CharacterState};
use super::item::{ItemSlot, ItemSpecs};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerBaseSpecs {
    pub character_static: CharacterStatic,
    pub character_attrs: CharacterAttrs,
    pub effects: Vec<StatEffect>,

    pub buy_skill_cost: f64,
    pub max_skills: u8,
    pub skills: IndexMap<String, PlayerBaseSkill>,
    pub skill_masteries: PlayerSkillMasteries,

    pub level: u8,
    pub experience_needed: f64,
    pub max_level: u8,

    pub movement_cooldown: AtLeastOne,
    pub gold_find: NonNegative,
    pub threat_gain: NonNegative,

    pub max_area_level: AreaLevel,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerSpecs {
    pub character_specs: CharacterSpecs,

    pub movement_cooldown: AtLeastOne,
    pub gold_find: NonNegative,
    pub threat_gain: NonNegative,

    pub computed_status_triggers: HashMap<String, TriggerEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerBaseSkill {
    pub base_skill_specs: BaseSkillSpecs,
    pub item_slot: Option<ItemSlot>,

    pub upgrade_level: u16,
    pub next_upgrade_cost: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerState {
    pub character_state: CharacterState,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerResources {
    pub experience: f64,
    pub passive_points: u16,

    pub gold: f64,
    pub gems: f64,
    pub shards: f64,

    pub gold_total: f64,

    pub skill_masteries_experience: HashMap<String, f64>,
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

    // TODO: Have proper migration and have sheathed as part of EquippedSlot
    #[serde(default)]
    pub sheathed: HashSet<ItemSlot>,
}

impl PlayerInventory {
    pub fn get_equipped_item(&self, slot: ItemSlot) -> Option<&ItemSpecs> {
        match self.equipped.get(&slot) {
            Some(EquippedSlot::MainSlot(x)) => Some(x),
            Some(EquippedSlot::ExtraSlot(slot)) => self.get_equipped_item(*slot),
            None => None,
        }
    }

    // Get equipped items, preserving slot order
    pub fn equipped_items(&self) -> impl Iterator<Item = (ItemSlot, &Box<ItemSpecs>)> + Clone {
        ItemSlot::iter().filter_map(|slot| match self.equipped.get(&slot) {
            Some(EquippedSlot::MainSlot(item_specs)) => Some((slot, item_specs)),
            _ => None,
        })
    }
    // pub fn equipped_items(&self) -> impl Iterator<Item = (ItemSlot, &Box<ItemSpecs>)> + Clone {
    //     self.equipped
    //         .iter()
    //         .filter_map(|(slot, equipped_slot)| match equipped_slot {
    //             EquippedSlot::MainSlot(item_specs) => Some((*slot, item_specs)),
    //             EquippedSlot::ExtraSlot(_) => None,
    //         })
    // }

    pub fn equipped_items_mut(&mut self) -> impl Iterator<Item = (ItemSlot, &mut Box<ItemSpecs>)> {
        self.equipped
            .iter_mut()
            .filter_map(|(slot, equipped_slot)| match equipped_slot {
                EquippedSlot::MainSlot(item_specs) => Some((*slot, item_specs)),
                EquippedSlot::ExtraSlot(_) => None,
            })
    }

    pub fn all_items(&self) -> impl Iterator<Item = &ItemSpecs> + Clone {
        self.bag.iter().chain(
            self.equipped_items()
                .map(|(_, item_specs)| item_specs.as_ref()),
        )
    }

    pub fn all_items_mut(&mut self) -> impl Iterator<Item = &mut ItemSpecs> {
        self.bag.iter_mut().chain(
            self.equipped
                .iter_mut()
                .filter_map(|(slot, equipped_slot)| match equipped_slot {
                    EquippedSlot::MainSlot(item_specs) => Some((*slot, item_specs)),
                    EquippedSlot::ExtraSlot(_) => None,
                })
                .map(|(_, item_specs)| item_specs.as_mut()),
        )
    }

    pub fn nth(&self, index: usize) -> Option<&ItemSpecs> {
        if index < 9 {
            self.equipped
                .get(&index.try_into().ok()?)
                .and_then(|equipped_item| match equipped_item {
                    EquippedSlot::MainSlot(item_specs) => Some(item_specs.as_ref()),
                    _ => None,
                })
        } else {
            self.bag.get(index.saturating_sub(9))
        }
    }

    pub fn nth_mut(&mut self, index: usize) -> Option<&mut ItemSpecs> {
        if index < 9 {
            self.equipped
                .get_mut(&index.try_into().ok()?)
                .and_then(|equipped_item| match equipped_item {
                    EquippedSlot::MainSlot(item_specs) => Some(item_specs.as_mut()),
                    _ => None,
                })
        } else {
            self.bag.get_mut(index.saturating_sub(9))
        }
    }
}
