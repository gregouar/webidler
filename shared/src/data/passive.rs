use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::data::trigger::TriggerSpecs;

pub use super::stat_effect::StatEffect;

pub type PassiveNodeId = String;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PassiveNodeType {
    Attack,
    Life,
    Spell,
    Armor,
    Critical,
    Mana,
    Gold,
    Physical,
    Poison,
    Fire,
    Status,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PassivesTreeSpecs {
    pub nodes: HashMap<PassiveNodeId, PassiveNodeSpecs>,
    pub connections: Vec<PassiveConnection>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PassivesTreeState {
    pub purchased_nodes: HashSet<PassiveNodeId>,
    // TODO: Should we have 3 layers? Blueprint, Specs & State?
    pub ascended_nodes: HashMap<PassiveNodeId, u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PassiveNodeSpecs {
    pub name: String,
    pub icon: String,

    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub size: u8,

    #[serde(default)]
    pub initial_node: bool,

    // TODO: Replace by Asset uri?
    pub node_type: PassiveNodeType,

    #[serde(default)]
    pub effects: Vec<StatEffect>,
    #[serde(default)]
    pub triggers: Vec<TriggerSpecs>,

    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub upgrade_effects: Vec<StatEffect>,
    // TODO: unlocked & ascend costs?
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PassiveConnection {
    pub from: PassiveNodeId,
    pub to: PassiveNodeId,
}
