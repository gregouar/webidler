use leptos::prelude::*;
use std::sync::Arc;

use shared::{
    data::{item::ItemSlot, player::EquippedSlot},
    messages::client::{EquipItemMessage, FilterLootMessage, SellItemsMessage, UnequipItemMessage},
};

use crate::components::{
    game::game_context::GameContext,
    shared::inventory::{Inventory, InventoryConfig, SellType},
    ui::confirm::ConfirmContext,
    websocket::WebsocketContext,
};

#[component]
pub fn GameInventoryPanel(open: RwSignal<bool>) -> impl IntoView {
    let game_context = expect_context::<GameContext>();
    let conn = expect_context::<WebsocketContext>();
    let confirm_context = expect_context::<ConfirmContext>();

    // Loot filter
    Effect::new({
        let conn = conn.clone();
        move || {
            conn.send(
                &FilterLootMessage {
                    preferred_loot: game_context.loot_preference.get(),
                }
                .into(),
            );
        }
    });

    // Equip
    let try_equip = {
        let conn = conn.clone();
        let confirm_context = confirm_context.clone();
        move |item_index: u8| {
            let conn = conn.clone();
            let equip = Arc::new({
                move || {
                    conn.send(&EquipItemMessage { item_index }.into());
                }
            });

            let inventory = game_context.player_inventory.read();
            let need_confirm = inventory
                .bag
                .get(item_index as usize)
                .and_then(|x| inventory.equipped.get(&x.base.slot))
                .and_then(|x| match x {
                    EquippedSlot::ExtraSlot(item_slot) => inventory.equipped.get(item_slot),
                    x => Some(x),
                })
                .map(|x| {
                    if let EquippedSlot::MainSlot(x) = x {
                        x.weapon_specs.is_some()
                    } else {
                        false
                    }
                })
                .unwrap_or_default();

            if need_confirm {
                (confirm_context
                        .confirm)(
                        "Equipping a new weapon will reset the weapon attack skill upgrade level to 1, are you sure?"
                            .to_string(),
                        equip.clone(),
                    );
            } else {
                equip();
            }
        }
    };

    // Unequip
    let try_unequip = {
        let conn = conn.clone();
        let confirm_context = confirm_context.clone();
        move |item_slot: ItemSlot| {
            let conn = conn.clone();
            let unequip = Arc::new({
                move || {
                    conn.send(&UnequipItemMessage { item_slot }.into());
                }
            });

            let inventory = game_context.player_inventory.read();
            let need_confirm = inventory
                .equipped
                .get(&item_slot)
                .map(|x| {
                    if let EquippedSlot::MainSlot(x) = x {
                        x.weapon_specs.is_some()
                    } else {
                        false
                    }
                })
                .unwrap_or_default();

            if need_confirm {
                (confirm_context
                        .confirm)(
                        "Unequipping your weapon will reset the weapon attack skill upgrade level to 1, are you sure?"
                            .to_string(),
                        unequip.clone(),
                    );
            } else {
                unequip();
            }
            // on_close.run(());
        }
    };

    // Sell
    let sell = {
        let conn = conn.clone();
        move |item_indexes| {
            conn.send(&SellItemsMessage { item_indexes }.into());
        }
    };

    let inventory_config = InventoryConfig {
        player_inventory: game_context.player_inventory,
        loot_preference: Some(game_context.loot_preference),
        on_unequip: Some(Arc::new(try_unequip)),
        on_equip: Some(Arc::new(try_equip)),
        on_sell: Some(Arc::new(sell)),
        sell_type: SellType::Sell,
    };

    view! { <Inventory open=open inventory=inventory_config /> }
}
