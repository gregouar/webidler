use shared::data::{
    item::{ItemCategory, ItemRarity, ItemSpecs, LootState, QueuedLoot},
    player::PlayerSpecs,
};

use crate::rng;

const MAX_QUEUE_SIZE: usize = 5;

// TODO: LootPool, area level, ..?
pub fn drop_loot(queued_loot: &mut Vec<QueuedLoot>) {
    let last_index = queued_loot
        .iter()
        .map(|x| x.identifier + 1)
        .max()
        .unwrap_or_default();

    queued_loot.retain(|loot| loot.state != LootState::HasDisappeared);

    let rarity = match rng::random_range(0..4).unwrap_or(0) {
        0 => ItemRarity::Normal,
        1 => ItemRarity::Magic,
        2 => ItemRarity::Rare,
        _ => ItemRarity::Unique,
    };

    queued_loot.push(QueuedLoot {
        identifier: last_index,
        item_specs: ItemSpecs {
            name: "Trinky".to_string(),
            icon: "items/battleaxe.webp".to_string(),
            description: "Some trinket".to_string(),
            rarity: rarity,
            item_level: 1,
            item_category: ItemCategory::Trinket,
            affixes: vec![],
        },
        state: LootState::Normal,
    });

    update_loot_states(queued_loot);
}

pub fn pickup_loot(
    player_specs: &mut PlayerSpecs,
    queued_loot: &mut Vec<QueuedLoot>,
    loot_identifier: u32,
) -> bool {
    let mut move_item = None;

    if let Some((index, loot)) = queued_loot
        .iter_mut()
        .enumerate()
        .find(|(_, x)| x.identifier == loot_identifier)
    {
        if player_specs.inventory.bag.len() < player_specs.inventory.max_bag_size as usize {
            loot.state = LootState::HasDisappeared;
            player_specs.inventory.bag.push(loot.item_specs.clone());
        } else {
            move_item = Some(index);
        }
    }

    if let Some(index) = move_item {
        let mut i = index;
        while i < queued_loot.len() - 1 {
            queued_loot.swap(i, i + 1);
            i += 1;
        }
    }

    update_loot_states(queued_loot);

    move_item.is_none()
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
