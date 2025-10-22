use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use shared::data::{
    item::{ItemModifiers, ItemSlot},
    player::PlayerInventory,
};

use crate::game::{
    data::items_store::ItemsStore,
    systems::{inventory_controller, items_controller::init_item_specs_from_store},
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InventoryData {
    pub equipped: HashMap<ItemSlot, ItemModifiers>,
    pub bag: Vec<ItemModifiers>,
    pub max_bag_size: u8,
}

pub fn inventory_data_to_player_inventory(
    items_store: &ItemsStore,
    inventory_data: InventoryData,
) -> PlayerInventory {
    let mut player_inventory = PlayerInventory {
        equipped: Default::default(),
        bag: inventory_data
            .bag
            .into_iter()
            .filter_map(|item_modifiers| init_item_specs_from_store(items_store, item_modifiers))
            .collect(),
        max_bag_size: inventory_data.max_bag_size,
    };

    for item_specs in inventory_data
        .equipped
        .into_values()
        .filter_map(|item_modifiers| init_item_specs_from_store(items_store, item_modifiers))
    {
        let _ = inventory_controller::equip_item(&mut player_inventory, item_specs);
    }

    player_inventory
}
