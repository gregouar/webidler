use shared::data::{
    item::{ArmorSpecs, ItemBase, ItemRarity, ItemSlot, ItemSpecs},
    loot::{LootState, QueuedLoot},
    player::PlayerSpecs,
};

use crate::game::utils::rng;

use super::items_controller;

const MAX_QUEUE_SIZE: usize = 5;

// TODO: LootPool, area level, ..?
pub fn generate_loot() -> ItemSpecs {
    let rarity = match rng::random_range(0..4).unwrap_or(0) {
        0 => ItemRarity::Normal,
        1 => ItemRarity::Magic,
        2 => ItemRarity::Rare,
        _ => ItemRarity::Unique,
    };

    items_controller::update_item_specs(ItemSpecs {
        base: ItemBase {
            name: "Helmet".to_string(),
            icon: "items/helmet.webp".to_string(),
            description: "Save your brain".to_string(),
            item_slot: ItemSlot::Helmet,
            armor_specs: Some(ArmorSpecs { armor: 10.0 }),
            weapon_specs: None,
            min_level: 1,
        },
        rarity: rarity,
        level: 1,
        armor_specs: None,
        weapon_specs: None,
        affixes: vec![],
    })
}

pub fn drop_loot(queued_loot: &mut Vec<QueuedLoot>, item_specs: ItemSpecs) {
    drop_loot_impl(queued_loot, item_specs, true);
}

pub fn pickup_loot(
    player_specs: &mut PlayerSpecs,
    queued_loot: &mut Vec<QueuedLoot>,
    loot_identifier: u32,
) -> bool {
    let mut move_item = None;

    if let Some(loot) = queued_loot
        .iter_mut()
        .find(|x| x.identifier == loot_identifier)
    {
        loot.state = LootState::HasDisappeared;
        if player_specs.inventory.bag.len() < player_specs.inventory.max_bag_size as usize {
            player_specs.inventory.bag.push(loot.item_specs.clone());
        } else {
            move_item = Some(loot.item_specs.clone());
        }
    }

    if let Some(item_specs) = move_item {
        drop_loot_impl(queued_loot, item_specs, false);
        return false;
    }

    update_loot_states(queued_loot);
    true
}

fn drop_loot_impl(
    queued_loot: &mut Vec<QueuedLoot>,
    item_specs: ItemSpecs,
    purge_disappeared: bool,
) {
    let last_index = queued_loot
        .iter()
        .map(|x| x.identifier + 1)
        .max()
        .unwrap_or_default();

    // This feels very hacky =/
    if purge_disappeared {
        queued_loot.retain(|loot| loot.state != LootState::HasDisappeared);
    }

    queued_loot.push(QueuedLoot {
        identifier: last_index,
        item_specs,
        state: LootState::Normal,
    });

    update_loot_states(queued_loot);
}

fn update_loot_states(queued_loot: &mut Vec<QueuedLoot>) {
    let mut queued_loot: Vec<_> = queued_loot
        .iter_mut()
        .filter(|x| x.state != LootState::HasDisappeared)
        .collect();

    for loot in queued_loot.iter_mut() {
        loot.state = LootState::Normal;
    }

    for i in 0..queued_loot
        .len()
        .checked_sub(MAX_QUEUE_SIZE)
        .unwrap_or_default()
    {
        queued_loot[i].state = LootState::HasDisappeared;
    }

    let mut queued_loot: Vec<_> = queued_loot
        .iter_mut()
        .filter(|x| x.state != LootState::HasDisappeared)
        .collect();

    for i in 0..queued_loot
        .len()
        .checked_sub(MAX_QUEUE_SIZE - 1)
        .unwrap_or_default()
    {
        queued_loot[i].state = LootState::WillDisappear;
    }
}
