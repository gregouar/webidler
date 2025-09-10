use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::data::{stat_effect::EffectsMap, trigger::TriggerSpecs};

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
    Storm,
    Status,
    Utility,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PassivesTreeSpecs {
    pub nodes: HashMap<PassiveNodeId, PassiveNodeSpecs>,
    pub connections: Vec<PassiveConnection>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PassivesTreeAscension {
    pub ascended_nodes: HashMap<PassiveNodeId, u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PassivesTreeState {
    pub purchased_nodes: HashSet<PassiveNodeId>,
    pub ascension: PassivesTreeAscension,
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
    #[serde(default)]
    pub max_upgrade_level: Option<u8>,
    // TODO: unlocked & ascend costs?
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PassiveConnection {
    pub from: PassiveNodeId,
    pub to: PassiveNodeId,
}

impl PassiveNodeSpecs {
    pub fn aggregate_effects(&self, level: u8) -> EffectsMap {
        let effects_map =
            self.effects
                .iter()
                .fold(EffectsMap(HashMap::new()), |mut effects_map, effect| {
                    *effects_map
                        .0
                        .entry((effect.stat, effect.modifier))
                        .or_default() += effect.value;
                    effects_map
                });

        let level = if self.locked {
            level.saturating_sub(1)
        } else {
            level
        };

        self.upgrade_effects
            .iter()
            .fold(effects_map, |mut effects_map, effect| {
                *effects_map
                    .0
                    .entry((effect.stat, effect.modifier))
                    .or_default() += effect.value * level as f64;
                effects_map
            })
    }
}
