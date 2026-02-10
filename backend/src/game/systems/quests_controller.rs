use shared::{
    constants::{ITEM_REWARDS_MAP_MIN_LEVEL, ITEM_REWARDS_MIN_LEVEL},
    data::{item::ItemCategory, quest::QuestRewards},
};

use crate::{
    app_state::MasterStore,
    game::{
        game_data::GameInstanceData,
        systems::{inventory_controller, loot_generator},
    },
    rest::AppError,
};

pub fn end_quest(master_store: &MasterStore, game_data: &mut GameInstanceData) {
    if !game_data.end_quest {
        game_data.end_quest = true;
        *game_data.quest_rewards.mutate() =
            Some(generate_end_quest_rewards(master_store, game_data));
    }
}

pub fn terminate_quest(
    game_data: &mut GameInstanceData,
    item_index: Option<u8>,
) -> Result<(), AppError> {
    if !game_data.end_quest {
        return Err(AppError::UserError("grind not yet ended".into()));
    }

    if game_data.terminate_quest {
        return Err(AppError::UserError("grind already terminated".into()));
    }

    if let Some(quest_rewards) = game_data.quest_rewards.read()
        && let Some(item_specs) =
            item_index.and_then(|item_index| quest_rewards.item_rewards.get(item_index as usize))
    {
        inventory_controller::store_item_to_bag(
            game_data.player_inventory.mutate(),
            item_specs.clone(),
        )?;
    }

    game_data.terminate_quest = true;

    Ok(())
}

fn generate_end_quest_rewards(
    master_store: &MasterStore,
    game_data: &GameInstanceData,
) -> QuestRewards {
    let delta_area_level = 1 + game_data
        .area_state
        .read()
        .max_area_level
        .saturating_sub(game_data.area_blueprint.specs.starting_level);

    let mut item_rewards = Vec::new();

    // If enough, generate 2 Maps
    if delta_area_level >= ITEM_REWARDS_MAP_MIN_LEVEL {
        item_rewards.extend((0..2).flat_map(|_| {
            loot_generator::generate_loot(
                &game_data.area_blueprint.loot_table,
                &master_store.items_store,
                &master_store.item_affixes_table,
                &master_store.item_adjectives_table,
                &master_store.item_nouns_table,
                game_data
                    .area_state
                    .read()
                    .max_area_level
                    .saturating_add(game_data.area_blueprint.specs.item_level_modifier),
                false,
                true,
                Some(ItemCategory::Map),
                game_data.area_state.read().loot_rarity,
            )
        }));
    // Otherwise fill with normal items
    } else if delta_area_level >= ITEM_REWARDS_MIN_LEVEL {
        item_rewards.extend((0..2).flat_map(|_| {
            loot_generator::generate_loot(
                &game_data.area_blueprint.loot_table,
                &master_store.items_store,
                &master_store.item_affixes_table,
                &master_store.item_adjectives_table,
                &master_store.item_nouns_table,
                game_data
                    .area_state
                    .read()
                    .max_area_level
                    .saturating_add(game_data.area_blueprint.specs.item_level_modifier),
                false,
                true,
                None,
                game_data.area_state.read().loot_rarity,
            )
        }));
    }

    // Add an extra rarer item
    if delta_area_level >= ITEM_REWARDS_MIN_LEVEL {
        item_rewards.extend((0..1).flat_map(|_| {
            loot_generator::generate_loot(
                &game_data.area_blueprint.loot_table,
                &master_store.items_store,
                &master_store.item_affixes_table,
                &master_store.item_adjectives_table,
                &master_store.item_nouns_table,
                game_data
                    .area_state
                    .read()
                    .max_area_level
                    .saturating_add(game_data.area_blueprint.specs.item_level_modifier),
                true,
                true,
                None,
                game_data.area_state.read().loot_rarity * 5.0,
            )
        }))
    }

    QuestRewards { item_rewards }
}
