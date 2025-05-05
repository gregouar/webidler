use shared::data::{
    item::{ItemCategory, ItemRarity, ItemSpecs, LootState, QueuedLoot},
    player::PlayerSpecs,
};

use crate::rng;

const MAX_QUEUE_SIZE: usize = 2;

// TODO: LootPool, area level, ..?
pub fn drop_loot(queued_loot: &mut Vec<QueuedLoot>) {
    queued_loot.retain(|loot| loot.state != LootState::HasDisappeared);

    let last_index = queued_loot
        .last()
        .map(|x| x.identifier + 1)
        .unwrap_or_default();

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
) {
    if let Some(loot) = queued_loot
        .iter_mut()
        .find(|x| x.identifier == loot_identifier)
    {
        loot.state = LootState::HasDisappeared;
        // TODO: Bag limit
        player_specs.inventory.bag.push(loot.item_specs.clone());

        update_loot_states(queued_loot);
    }
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
