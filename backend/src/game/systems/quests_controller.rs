use shared::{
    constants::{ITEM_REWARDS_MAP_MIN_LEVEL, ITEM_REWARDS_MIN_LEVEL, ITEM_REWARDS_RARE_FACTOR},
    data::{
        item::{ItemCategory, ItemRarity},
        quest::{QuestRewards, SkillMasteryReward},
    },
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
    reward_picks: Vec<u8>,
) -> Result<(), AppError> {
    if !game_data.end_quest {
        return Err(AppError::UserError("grind not yet ended".into()));
    }

    if game_data.terminate_quest {
        return Err(AppError::UserError("grind already terminated".into()));
    }

    if reward_picks.len() > game_data.area_specs.reward_picks as usize {
        return Err(AppError::UserError("too many reward picks".into()));
    }

    if game_data.player_inventory.read().bag.len() + reward_picks.len()
        > game_data.player_inventory.read().max_bag_size as usize
    {
        return Err(AppError::UserError("not enough space".into()));
    }

    if let Some(quest_rewards) = game_data.quest_rewards.read() {
        for reward_pick in reward_picks.into_iter() {
            if let Some(item_specs) = quest_rewards.item_rewards.get(reward_pick as usize) {
                inventory_controller::store_item_to_bag(
                    game_data.player_inventory.mutate(),
                    item_specs.clone(),
                )?;
            }
        }
    }

    game_data.terminate_quest = true;

    Ok(())
}

fn generate_end_quest_rewards(
    master_store: &MasterStore,
    game_data: &GameInstanceData,
) -> QuestRewards {
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
                item_level,
                0,
                false,
                true,
                false,
                false,
                Some(ItemCategory::Map),
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
                item_level,
                0,
                false,
                true,
                false,
                false,
                None,
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
                    item_level,
                    0,
                    true,
                    false,
                    false,
                    None,
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
                    item_level,
                    0,
                    true,
                    true,
                    false,
                    false,
                    None,
                    *game_data.area_specs.loot_rarity * ITEM_REWARDS_RARE_FACTOR,
                    0.0,
                )
            }
        }))
        .collect();

    let skill_mastery_rewards = game_data
        .player_resources
        .read()
        .skill_masteries_experience
        .iter()
        .filter_map(|(skill_id, experience_gained)| {
            if *experience_gained <= 0.0 {
                return None;
            }

            let previous_mastery = game_data
                .player_base_specs
                .read()
                .skill_masteries
                .masteries
                .get(skill_id)
                .cloned()
                .unwrap_or_default();
            let mut current_mastery = previous_mastery.clone();
            current_mastery.experience += experience_gained;

            Some(SkillMasteryReward {
                skill_id: skill_id.clone(),
                previous_level: previous_mastery.level(),
                current_level: current_mastery.level(),
                experience_gained: *experience_gained,
                current_relative_experience: current_mastery.relative_experience(),
                current_next_level_cost: current_mastery.next_level_cost(),
            })
        })
        .collect();

    QuestRewards {
        item_rewards,
        skill_mastery_rewards,
    }
}
