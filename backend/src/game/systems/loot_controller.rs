use shared::data::{
    item::{ItemRarity, ItemSpecs},
    loot::{LootState, QueuedLoot},
    player::PlayerInventory,
};

use super::player_controller::PlayerController;

const MAX_QUEUE_SIZE: usize = 5;

pub fn drop_loot(
    player_controller: &PlayerController,
    queued_loot: &mut Vec<QueuedLoot>,
    item_specs: ItemSpecs,
) -> Vec<ItemSpecs> {
    drop_loot_impl(player_controller, queued_loot, item_specs, true)
}

pub fn pickup_loot(
    player_controller: &PlayerController,
    player_inventory: &mut PlayerInventory,
    queued_loot: &mut Vec<QueuedLoot>,
    loot_identifier: u32,
) -> bool {
    // Will contain back the item if inventory is full
    let mut move_item = None;

    if let Some(loot) = queued_loot
        .iter_mut()
        .find(|x| x.identifier == loot_identifier && x.state != LootState::HasDisappeared)
    {
        loot.state = LootState::HasDisappeared;
        if player_inventory.bag.len() < player_inventory.max_bag_size as usize {
            player_inventory.bag.push(loot.item_specs.clone());
        } else {
            move_item = Some(loot.item_specs.clone());
        }
    }

    // Put back item at front of queue if couldn't pickup
    if let Some(item_specs) = move_item {
        drop_loot_impl(player_controller, queued_loot, item_specs, false);
        return false;
    }

    update_loot_states(player_controller, queued_loot);
    true
}

// Return discarded loot
fn drop_loot_impl(
    player_controller: &PlayerController,
    queued_loot: &mut Vec<QueuedLoot>,
    item_specs: ItemSpecs,
    purge_disappeared: bool,
) -> Vec<ItemSpecs> {
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

    update_loot_states(player_controller, queued_loot)
}

// Return discarded loot
fn update_loot_states(
    player_controller: &PlayerController,
    queued_loot: &mut [QueuedLoot],
) -> Vec<ItemSpecs> {
    // TODO: replace by reference
    let mut discarded_loot = Vec::new();

    let mut queued_loot: Vec<_> = queued_loot
        .iter_mut()
        .filter(|x| x.state != LootState::HasDisappeared)
        .collect();

    for loot in queued_loot.iter_mut() {
        loot.state = LootState::Normal;
    }

    let amount_to_discard = queued_loot.len().saturating_sub(MAX_QUEUE_SIZE);
    let last_index = queued_loot.len().saturating_sub(1);
    for i in 0..amount_to_discard {
        // If new loot is worst than the one we want to discard, we discard the new one instead
        // and put back old loot in front
        if is_better_loot(
            player_controller,
            &queued_loot[i].item_specs,
            &queued_loot[last_index].item_specs,
        ) && i != last_index
        {
            let (left, right) = queued_loot.split_at_mut(last_index);
            std::mem::swap(&mut left[i].item_specs, &mut right[0].item_specs);
        }

        queued_loot[i].state = LootState::HasDisappeared;
        discarded_loot.push(queued_loot[i].item_specs.clone());
    }

    let mut queued_loot: Vec<_> = queued_loot
        .iter_mut()
        .filter(|x| x.state != LootState::HasDisappeared)
        .collect();

    for i in 0..queued_loot.len().saturating_sub(MAX_QUEUE_SIZE - 1) {
        queued_loot[i].state = LootState::WillDisappear;
    }

    discarded_loot
}

fn is_better_loot(
    player_controller: &PlayerController,
    item_1: &ItemSpecs,
    item_2: &ItemSpecs,
) -> bool {
    item_score(player_controller, item_1) > item_score(player_controller, item_2)
}

fn item_score(player_controller: &PlayerController, item: &ItemSpecs) -> usize {
    let mut score = 0;
    if let Some(item_category) = player_controller.preferred_loot {
        if item.base.categories.contains(&item_category) {
            score += 1_000_000;
        }
    }

    score += match item.modifiers.rarity {
        ItemRarity::Normal | ItemRarity::Magic | ItemRarity::Rare | ItemRarity::Masterwork => 0,
        ItemRarity::Unique => 1_500_000,
    };

    score += item
        .modifiers
        .affixes
        .iter()
        .map(|a| a.tier as usize)
        .sum::<usize>()
        * 10_000;

    score += item.base.min_area_level as usize * 1_000;

    score
}
