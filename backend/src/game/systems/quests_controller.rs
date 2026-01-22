use shared::data::quest::QuestRewards;

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

    if let Some(ref quest_rewards) = game_data.quest_rewards.read() {
        if let Some(item_specs) = item_index
            .map(|item_index| quest_rewards.item_rewards.get(item_index as usize))
            .flatten()
        {
            inventory_controller::store_item_to_bag(
                game_data.player_inventory.mutate(),
                item_specs.clone(),
            )?;
        }
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
        .max_area_level_ever
        .saturating_sub(game_data.area_blueprint.specs.starting_level);

    if delta_area_level >= 100 {}

    // TODO: If completed X levels only
    // TODO: Try to generate first 2 edicts and then complete until 3 items
    let item_rewards = (0..3)
        .into_iter()
        .flat_map(|_| {
            loot_generator::generate_loot(
                &game_data.area_blueprint.loot_table,
                &master_store.items_store,
                &master_store.item_affixes_table,
                &master_store.item_adjectives_table,
                &master_store.item_nouns_table,
                game_data
                    .area_state
                    .read()
                    .area_level
                    .saturating_add(game_data.area_blueprint.specs.item_level_modifier),
                false,
                true,
                game_data.area_state.read().loot_rarity,
            )
        })
        .collect();

    QuestRewards { item_rewards }
}
