use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use shared::data::{
    item::ItemModifiers,
    passive::{PassiveNodeId, PassivesTreeAscension},
};

use crate::game::{data::items_store::ItemsStore, systems::items_controller};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct PassivesTreeAscensionData {
    pub ascended_nodes: HashMap<PassiveNodeId, u8>,
    #[serde(default)]
    pub socketed_nodes: HashMap<PassiveNodeId, ItemModifiers>,
}

impl From<&PassivesTreeAscension> for PassivesTreeAscensionData {
    fn from(value: &PassivesTreeAscension) -> Self {
        Self {
            ascended_nodes: value.ascended_nodes.clone(),
            socketed_nodes: value
                .socketed_nodes
                .iter()
                .map(|(node_id, item_specs)| (*node_id, item_specs.modifiers.clone()))
                .collect(),
        }
    }
}

pub fn ascension_data_to_passives_tree_ascension(
    items_store: &ItemsStore,
    passives_data: PassivesTreeAscensionData,
) -> PassivesTreeAscension {
    PassivesTreeAscension {
        ascended_nodes: passives_data.ascended_nodes,
        socketed_nodes: passives_data
            .socketed_nodes
            .into_iter()
            .filter_map(|(node_id, item_modifiers)| {
                items_controller::init_item_specs_from_store(items_store, item_modifiers)
                    .map(|item_specs| (node_id, item_specs))
            })
            .collect(),
    }
}
