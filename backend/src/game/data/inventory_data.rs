use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use shared::data::{
    item::{ItemModifiers, ItemSlot},
    player::PlayerInventory,
};

use crate::game::{
    data::master_store::MasterStore,
    systems::{inventory_controller, items_controller::init_item_specs_from_store},
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InventoryData {
    pub equipped: HashMap<ItemSlot, ItemModifiers>,
    pub bag: Vec<ItemModifiers>,
    pub max_bag_size: u8,

    #[serde(default)]
    pub sheathed: HashSet<ItemSlot>,
}

pub fn inventory_data_to_player_inventory(
    master_store: &MasterStore,
    inventory_data: InventoryData,
) -> PlayerInventory {
    let mut player_inventory = PlayerInventory {
        equipped: Default::default(),
        bag: inventory_data
            .bag
            .into_iter()
            .filter_map(|item_modifiers| {
                init_item_specs_from_store(
                    &master_store.items_store,
                    &master_store.item_affixes_table,
                    item_modifiers,
                )
            })
            .collect(),
        max_bag_size: inventory_data.max_bag_size,
        sheathed: inventory_data.sheathed,
    };

    for item_specs in inventory_data
        .equipped
        .into_values()
        .filter_map(|item_modifiers| {
            init_item_specs_from_store(
                &master_store.items_store,
                &master_store.item_affixes_table,
                item_modifiers,
            )
        })
    {
        let _ = inventory_controller::equip_item(&mut player_inventory, item_specs);
    }

    player_inventory
}
