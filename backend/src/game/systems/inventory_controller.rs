use anyhow::Result;

use shared::data::{
    area::AreaLevel,
    item::{InventorySortType, ItemSlot, ItemSpecs},
    player::{EquippedSlot, PlayerInventory},
};

use crate::rest::AppError;

pub fn store_item_to_bag(
    player_inventory: &mut PlayerInventory,
    item_specs: ItemSpecs,
) -> Result<(), AppError> {
    if player_inventory.bag.len() >= player_inventory.max_bag_size as usize {
        return Err(AppError::UserError("Not enough space!".into()));
    }

    player_inventory.bag.push(item_specs);

    Ok(())
}

// Return equipped item, and removed item if any
pub fn equip_item_from_bag(
    max_item_level: AreaLevel,
    player_inventory: &mut PlayerInventory,
    item_index: u8,
) -> Result<(ItemSpecs, Option<ItemSpecs>), AppError> {
    let item_index = item_index as usize;
    let item_specs = player_inventory
        .bag
        .get(item_index)
        .ok_or(AppError::NotFound)?
        .clone();

    if item_specs.required_level > max_item_level {
        return Err(AppError::UserError("level too low".into()));
    }

    let old_item = equip_item(player_inventory, item_specs.clone())?;

    if let Some(old_item_specs) = old_item.clone() {
        player_inventory.bag[item_index] = old_item_specs;
    } else {
        player_inventory.bag.remove(item_index);
    }

    Ok((item_specs, old_item))
}

pub fn unequip_item_to_bag(
    player_inventory: &mut PlayerInventory,
    item_slot: ItemSlot,
) -> Result<(), AppError> {
    if player_inventory.bag.len() >= player_inventory.max_bag_size as usize {
        return Err(AppError::UserError("Your bag is full!".into()));
    }

    let old_item = unequip_item(player_inventory, item_slot).ok_or(AppError::NotFound)?;

    player_inventory.bag.push(old_item);

    Ok(())
}

/// Equip new item and return old equipped item
pub fn equip_item(
    player_inventory: &mut PlayerInventory,
    item_specs: ItemSpecs,
) -> Result<Option<ItemSpecs>, AppError> {
    let slot = item_specs
        .base
        .slot
        .ok_or(AppError::UserError("item cannot be equipped".into()))?;

    if item_specs
        .base
        .extra_slots
        .iter()
        .any(|x| match player_inventory.equipped.get(x) {
            Some(EquippedSlot::MainSlot(_)) => true,
            Some(EquippedSlot::ExtraSlot(main_slot)) => *main_slot != slot,
            None => false,
        })
    {
        return Err(AppError::UserError(
            "Not enough item slots available, please unequip first!".into(),
        ));
    }

    let old_item = unequip_item(player_inventory, slot);

    for item_slot in item_specs.base.extra_slots.iter() {
        player_inventory
            .equipped
            .insert(*item_slot, EquippedSlot::ExtraSlot(slot));
    }

    player_inventory
        .equipped
        .insert(slot, EquippedSlot::MainSlot(Box::new(item_specs)));

    Ok(old_item)
}

pub fn unequip_item(
    player_inventory: &mut PlayerInventory,
    item_slot: ItemSlot,
) -> Option<ItemSpecs> {
    match player_inventory.equipped.remove(&item_slot) {
        Some(EquippedSlot::MainSlot(old_item)) => {
            player_inventory.sheathed.remove(&item_slot);
            for item_slot in old_item.base.extra_slots.iter() {
                player_inventory.equipped.remove(item_slot);
                player_inventory.sheathed.remove(item_slot);
            }
            Some(*old_item)
        }
        Some(EquippedSlot::ExtraSlot(item_slot)) => unequip_item(player_inventory, item_slot),
        None => None,
    }
}

pub fn remove_item_from_bag(
    player_inventory: &mut PlayerInventory,
    item_index: u8,
) -> Result<ItemSpecs, AppError> {
    let item_index = item_index as usize;
    if item_index < player_inventory.bag.len() {
        Ok(player_inventory.bag.remove(item_index))
    } else {
        Err(AppError::NotFound)
    }
}

pub fn sort_bag(player_inventory: &mut PlayerInventory, sort_type: InventorySortType) {
    player_inventory.bag.sort_by(|a, b| {
        let rarity = || b.modifiers.rarity.cmp(&a.modifiers.rarity);
        let item_type = || {
            a.base
                .categories
                .iter()
                .next_back()
                .cmp(&b.base.categories.iter().next_back())
                .then_with(|| a.base.slot.cmp(&b.base.slot))
        };
        // let item_level = || b.modifiers.level.cmp(&a.modifiers.level);
        let req_level = || b.required_level.cmp(&a.required_level);

        let ordering = match sort_type {
            InventorySortType::Rarity => rarity().then_with(item_type).then_with(req_level),
            InventorySortType::ItemType => item_type().then_with(rarity).then_with(req_level),
            InventorySortType::ItemLevel => req_level().then_with(item_type).then_with(rarity),
        };

        ordering
            // .then_with(|| a.required_level.cmp(&b.required_level))
            .then_with(|| a.modifiers.level.cmp(&b.modifiers.level))
            // .then_with(|| b.modifiers.upgrade_level.cmp(&a.modifiers.upgrade_level))
            .then_with(|| b.modifiers.quality.total_cmp(&a.modifiers.quality))
            // .then_with(|| a.modifiers.name.cmp(&b.modifiers.name))
            .then_with(|| a.modifiers.base_item_id.cmp(&b.modifiers.base_item_id))
    });
}
