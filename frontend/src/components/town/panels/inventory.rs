use leptos::prelude::*;
use std::sync::Arc;

use shared::data::{item::ItemSlot, player::EquippedSlot};

use crate::components::{
    shared::inventory::{Inventory, InventoryConfig},
    town::TownContext,
};

#[component]
pub fn TownInventoryPanel(open: RwSignal<bool>) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let inventory_config = InventoryConfig {
        player_inventory: town_context.inventory,
        loot_preference: None,
        on_unequip: None,
        on_equip: None,
        on_sell: None,
    };

    view! { <Inventory open=open inventory=inventory_config /> }
}
