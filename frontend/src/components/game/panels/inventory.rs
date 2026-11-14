use leptos::prelude::*;
use std::sync::Arc;

use shared::{
    computations,
    data::item::ItemSlot,
    messages::client::{
        EquipItemMessage, SellItemsMessage, SheathItemMessage, SortInventoryMessage,
        UnequipItemMessage,
    },
};

use crate::components::{
    game::{game_context::GameContext, websocket::WebsocketContext},
    shared::{
        inventory::{Inventory, InventoryConfig, InventoryEquipFilter},
        loot_filter::LootFilterPanel,
        resources::show_resource_reward,
    },
};

#[component]
pub fn GameInventoryPanel(open: RwSignal<bool>) -> impl IntoView {
    let game_context = expect_context::<GameContext>();
    let conn = expect_context::<WebsocketContext>();
    let open_loot_filter = RwSignal::new(false);
    let sell_reward = RwSignal::new(Default::default());

    // Equip
    let try_equip = {
        let conn = conn.clone();
        move |item_index: u8| {
            conn.send(&EquipItemMessage { item_index }.into());
        }
    };

    // Unequip
    let try_unequip = {
        let conn = conn.clone();
        move |item_slot: ItemSlot| {
            conn.send(&UnequipItemMessage { item_slot }.into());
        }
    };

    // Sheathe / unsheathe
    let try_sheathe = {
        let conn = conn.clone();
        move |item_slot: ItemSlot| {
            conn.send(&SheathItemMessage { item_slot }.into());
        }
    };

    // Sell
    let sell = {
        let conn = conn.clone();
        move |item_indexes: Vec<u8>| {
            let (gold_reward, gems_reward) = game_context
                .player_inventory
                .read_untracked()
                .bag
                .iter()
                .enumerate()
                .filter(|(index, _)| item_indexes.contains(&(*index as u8)))
                .fold((0.0, 0.0), |(gold, gems), (_, item_specs)| {
                    (
                        gold + item_specs.gold_price,
                        gems + computations::item_gems_price(
                            item_specs.modifiers.level,
                            item_specs.modifiers.rarity,
                            game_context.realm.get_untracked().is_ssf(),
                        ),
                    )
                });
            show_resource_reward(sell_reward, gold_reward, gems_reward);
            conn.send(&SellItemsMessage { item_indexes }.into());
        }
    };

    let sort = {
        let conn = conn.clone();
        move |sort_type| {
            conn.send(&SortInventoryMessage { sort_type }.into());
        }
    };

    let inventory_config = InventoryConfig {
        player_inventory: game_context.player_inventory,
        on_loot_filter: Some(Arc::new(move || open_loot_filter.set(true))),
        on_unequip: Some(Arc::new(try_unequip)),
        on_sheathe: Some(Arc::new(try_sheathe)),
        on_equip: Some(Arc::new(try_equip)),
        on_sell: Some(Arc::new(sell)),
        on_sort: Some(Arc::new(sort)),
        sell_reward,
        max_item_level: Signal::derive(move || {
            game_context.player_base_specs.read().max_area_level
        }),
        equip_filter: Signal::derive(move || InventoryEquipFilter::Slot),
    };

    Effect::new(move || {
        if !open.get() {
            open_loot_filter.set(false);
        }
    });

    view! {
        <Inventory open=open inventory=inventory_config />
        <LootFilterPanel
            open=open_loot_filter
            loot_filter=game_context.loot_filter
            character_id=game_context.character_id.get_untracked()
            character_name=game_context
                .player_specs
                .read_untracked()
                .character_specs
                .character_static
                .name
                .clone()
        />
    }
}
