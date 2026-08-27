use shared::{
    constants::{ITEM_REWARDS_MAP_MIN_LEVEL, ITEM_REWARDS_MIN_LEVEL, ITEM_REWARDS_RARE_FACTOR},
    data::{
        grind::{GrindRewards, QuestReward},
        item::{ItemRarity, ItemSpecs},
    },
};

use crate::{
    app_state::MasterStore,
    game::{
        data::loot_table::LootTable,
        game_data::GameInstanceData,
        systems::{
            inventory_controller,
            loot_generator::{
                self, DEFAULT_LOOT_TEMPLATE, GenerateLootTemplate, MAP_LOOT_TEMPLATE,
            },
        },
    },
    rest::AppError,
};

pub fn end_grind(master_store: &MasterStore, game_data: &mut GameInstanceData) {
    if !game_data.end_grind {
        game_data.end_grind = true;
        *game_data.grind_rewards.mutate() =
            Some(generate_end_grind_rewards(master_store, game_data));
    }
}

pub fn terminate_grind(
    game_data: &mut GameInstanceData,
    reward_picks: Vec<u8>,
) -> Result<(), AppError> {
    if !game_data.end_grind {
        return Err(AppError::UserError("grind not yet ended".into()));
    }

    if game_data.terminate_grind {
        return Err(AppError::UserError("grind already terminated".into()));
    }

    if reward_picks.len() > game_data.area_specs.reward_picks as usize {
        return Err(AppError::UserError("too many reward picks".into()));
    }

    let quest_rewards_amount = game_data
        .grind_rewards
        .read()
        .as_ref()
        .and_then(|rewards| rewards.quest_reward.as_ref())
        .is_some() as usize;

    if game_data.player_inventory.read().bag.len() + reward_picks.len() + quest_rewards_amount
        > game_data.player_inventory.read().max_bag_size as usize
    {
        return Err(AppError::UserError("not enough space".into()));
    }

    if let Some(grind_rewards) = game_data.grind_rewards.read() {
        if let Some(quest_reward) = &grind_rewards.quest_reward {
            inventory_controller::store_item_to_bag(
                game_data.player_inventory.mutate(),
                quest_reward.clone(),
            )?;
        }
        for reward_pick in reward_picks.into_iter() {
            if let Some(item_specs) = grind_rewards.item_rewards.get(reward_pick as usize) {
                inventory_controller::store_item_to_bag(
                    game_data.player_inventory.mutate(),
                    item_specs.clone(),
                )?;
            }
        }
    }

    game_data.terminate_grind = true;

    Ok(())
}

fn generate_end_grind_rewards(
    master_store: &MasterStore,
    game_data: &GameInstanceData,
) -> GrindRewards {
    let area_level = game_data.area_state.read().max_area_level;

    // Up to 2 rewards are edict, only 1 if only 2 rewards available.
    let rewards_amount = if area_level >= ITEM_REWARDS_MIN_LEVEL && !game_data.area_specs.training {
        game_data.area_specs.reward_slots
    } else {
        0
    };

    let amount_map_rewards = if area_level >= ITEM_REWARDS_MAP_MIN_LEVEL {
        if rewards_amount > 2 { 2 } else { 1 }
    } else {
        0
    };

    let amount_normal_rewards = (2 - amount_map_rewards).min(rewards_amount);
    let amount_rare_rewards = rewards_amount - amount_normal_rewards - amount_map_rewards;

    let item_level = area_level
        .saturating_add(*game_data.area_specs.item_level_modifier)
        .saturating_add(*game_data.area_specs.power_level);

    let item_rewards = (0..amount_map_rewards)
        .flat_map(|_| {
            loot_generator::generate_loot(
                &game_data.area_blueprint.loot_table,
                &master_store.items_store,
                &master_store.item_affixes_table,
                &master_store.item_adjectives_table,
                &master_store.item_nouns_table,
                &MAP_LOOT_TEMPLATE,
                item_level,
                0,
                false,
                *game_data.area_specs.loot_rarity,
                0.0,
            )
        })
        .chain((0..amount_normal_rewards).flat_map(|_| {
            loot_generator::generate_loot(
                &game_data.area_blueprint.loot_table,
                &master_store.items_store,
                &master_store.item_affixes_table,
                &master_store.item_adjectives_table,
                &master_store.item_nouns_table,
                &DEFAULT_LOOT_TEMPLATE,
                item_level,
                0,
                false,
                *game_data.area_specs.loot_rarity,
                0.0,
            )
        }))
        .chain((0..amount_rare_rewards).flat_map(|_| {
            if let Some(reward_loot_table) = &game_data.area_blueprint.reward_loot_table {
                loot_generator::roll_item(
                    reward_loot_table,
                    &master_store.items_store,
                    &master_store.item_affixes_table,
                    &master_store.item_adjectives_table,
                    &master_store.item_nouns_table,
                    &DEFAULT_LOOT_TEMPLATE,
                    item_level,
                    0,
                    true,
                    ItemRarity::Unique,
                    0.0,
                )
            } else {
                loot_generator::generate_loot(
                    &game_data.area_blueprint.loot_table,
                    &master_store.items_store,
                    &master_store.item_affixes_table,
                    &master_store.item_adjectives_table,
                    &master_store.item_nouns_table,
                    &DEFAULT_LOOT_TEMPLATE,
                    item_level,
                    0,
                    true,
                    *game_data.area_specs.loot_rarity * ITEM_REWARDS_RARE_FACTOR,
                    0.0,
                )
            }
        }))
        .collect();

    let quest_reward = generate_quest_reward(master_store, game_data);

    GrindRewards {
        item_rewards,
        quest_reward,
    }
}

fn generate_quest_reward(
    master_store: &MasterStore,
    game_data: &GameInstanceData,
) -> Option<ItemSpecs> {
    let quest = game_data.area_specs.quest.as_ref()?;
    if game_data.quest_completed || game_data.area_state.read().max_area_level < quest.area_level {
        return None;
    }

    let reward = match &quest.reward {
        QuestReward::Item {
            item_id,
            level,
            rarity,
            max_affixes,
        } => {
            let base = master_store.items_store.content.get(item_id)?.clone();
            let rarity = if base.rarity == ItemRarity::Unique {
                ItemRarity::Unique
            } else {
                rarity.unwrap_or(base.rarity)
            };
            Some(loot_generator::roll_item_stats(
                item_id.clone(),
                base,
                rarity,
                *level,
                0,
                &master_store.item_affixes_table,
                &master_store.item_adjectives_table,
                &master_store.item_nouns_table,
                *max_affixes,
                0.0,
            ))
        }
        QuestReward::Loot {
            level,
            loot_tables,
            item_rarity,
            item_category,
            max_base,
            max_affixes,
        } => {
            let custom_loot_table;
            let loot_table = if let Some(loot_table_ids) = loot_tables {
                custom_loot_table = LootTable {
                    area_specific: false,
                    entries: loot_table_ids
                        .iter()
                        .filter_map(|id| master_store.loot_tables_store.get(id))
                        .flat_map(|table| table.entries.iter().cloned())
                        .collect(),
                };
                &custom_loot_table
            } else {
                &game_data.area_blueprint.loot_table
            };
            let template = GenerateLootTemplate {
                allow_unique: true,
                max_base: *max_base,
                max_affixes: *max_affixes,
                filter_category: *item_category,
                prevent_categories: &[],
            };

            match item_rarity {
                Some(rarity) => loot_generator::roll_item(
                    loot_table,
                    &master_store.items_store,
                    &master_store.item_affixes_table,
                    &master_store.item_adjectives_table,
                    &master_store.item_nouns_table,
                    &template,
                    *level,
                    0,
                    true,
                    *rarity,
                    0.0,
                ),
                None => loot_generator::generate_loot(
                    loot_table,
                    &master_store.items_store,
                    &master_store.item_affixes_table,
                    &master_store.item_adjectives_table,
                    &master_store.item_nouns_table,
                    &template,
                    *level,
                    0,
                    true,
                    *game_data.area_specs.loot_rarity,
                    0.0,
                ),
            }
        }
    };

    if reward.is_none() {
        tracing::error!(
            "failed to generate quest reward for area '{}' at quest level {}",
            game_data.area_id,
            quest.area_level
        );
    }
    reward
}
