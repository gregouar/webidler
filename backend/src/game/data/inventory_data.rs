use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use shared::data::{
    item::{ItemModifiers, ItemSlot},
    player::{EquippedSlot, PlayerInventory},
};

use crate::game::{
    data::items_store::ItemsStore,
    systems::{inventory_controller, items_controller::init_item_specs_from_store},
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InventoryData {
    pub equipped: HashMap<ItemSlot, EquippedItemData>,
    pub bag: Vec<ItemModifiers>,
    pub max_bag_size: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EquippedItemData {
    pub modifiers: ItemModifiers,
    pub sheathed: bool,
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

    for equipped_item_data in inventory_data.equipped.into_values() {
        if let Some(item_specs) =
            init_item_specs_from_store(items_store, equipped_item_data.modifiers)
        {
            let item_slot = item_specs.base.slot;
            let _ = inventory_controller::equip_item(&mut player_inventory, item_specs);

            if let Some(EquippedSlot::MainSlot { sheathed, .. }) =
                item_slot.and_then(|item_slot| player_inventory.equipped.get_mut(&item_slot))
            {
                *sheathed = equipped_item_data.sheathed;
            }
        }
    }

    player_inventory
}
