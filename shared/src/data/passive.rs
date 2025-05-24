use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

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
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PassivesTreeSpecs {
    pub nodes: HashMap<PassiveNodeId, PassiveNodeSpecs>,
    pub connections: Vec<PassiveConnection>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PassivesTreeState {
    pub purchased_nodes: HashSet<PassiveNodeId>,
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

    pub effects: Vec<StatEffect>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PassiveConnection {
    pub from: PassiveNodeId,
    pub to: PassiveNodeId,
}
