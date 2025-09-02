use shared::data::player::{EquippedSlot, PlayerInventory};

use crate::{
    auth::CurrentUser,
    db::{characters::CharacterEntry, characters_data::InventoryData},
    game::{data::items_store::ItemsStore, systems::items_controller::init_item_specs_from_store},
    rest::AppError,
};

pub fn verify_character_user(
    character: &CharacterEntry,
    current_user: &CurrentUser,
) -> Result<(), AppError> {
    if character.user_id != current_user.user.user_id {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

pub fn verify_character_in_town(character: &CharacterEntry) -> Result<(), AppError> {
    if character.area_id.is_some() {
        return Err(AppError::UserError("character is grinding".to_string()));
    }
    Ok(())
}

pub fn inventory_data_to_player_inventory(
    items_store: &ItemsStore,
    inventory_data: InventoryData,
) -> PlayerInventory {
    PlayerInventory {
        equipped: inventory_data
            .equipped
            .into_iter()
            .filter_map(|(item_slot, item_modifiers)| {
                Some((
                    item_slot,
                    EquippedSlot::MainSlot(Box::new(init_item_specs_from_store(
                        items_store,
                        item_modifiers,
                    )?)),
                ))
            })
            .collect(),
        bag: inventory_data
            .bag
            .into_iter()
            .filter_map(|item_modifiers| init_item_specs_from_store(items_store, item_modifiers))
            .collect(),
        max_bag_size: inventory_data.max_bag_size,
    }
}
