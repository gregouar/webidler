use chrono::{DateTime, Utc};
use sqlx::Transaction;
use std::collections::HashMap;

use shared::data::{
    achievements::{AchievementGoal, AchievementReward, AchievementSpecs},
    area::AreaLevel,
    item::ItemSpecs,
    passive::PassivesTreeAscension,
    player::PlayerInventory,
    skill_mastery::PlayerSkillMasteries,
    user::UserId,
};

use crate::{
    app_state::MasterStore,
    db::{self, pool::Database},
    game::data::master_store::AchievementsStore,
    rest::AppError,
};

pub struct AchievementContext<'a> {
    pub area_levels: &'a HashMap<String, AreaLevel>,
    pub power_level: AreaLevel,
    pub player_level: u8,
    pub skill_masteries: &'a PlayerSkillMasteries,
    pub ascension: &'a PassivesTreeAscension,
    pub inventory: &'a PlayerInventory,
}

pub async fn unlock_achievements(
    tx: &mut Transaction<'_, Database>,
    // master_store: &MasterStore,
    user_id: UserId,
    new_achievements: &[String],
) -> Result<(), AppError> {
    for achievement_id in new_achievements.iter() {
        unlock_achievement(
            tx,
            // &master_store.achievements_store,
            user_id,
            achievement_id,
        )
        .await?;
    }

    Ok(())
}

pub fn check_achievements(
    master_store: &MasterStore,
    already_unlocked: &HashMap<String, DateTime<Utc>>,
    context: &AchievementContext<'_>,
) -> Vec<String> {
    master_store
        .achievements_store
        .iter()
        .filter(|(id, specs)| {
            !already_unlocked.contains_key(*id)
                && achievement_satisfied(master_store, specs, context)
        })
        .map(|(id, _)| id.clone())
        .collect()
}

pub fn achievements_progress(
    master_store: &MasterStore,
    context: &AchievementContext<'_>,
) -> HashMap<String, usize> {
    master_store
        .achievements_store
        .iter()
        .map(|(id, specs)| {
            (
                id.clone(),
                achievement_progress(master_store, specs, context),
            )
        })
        .collect()
}

fn achievement_satisfied(
    master_store: &MasterStore,
    specs: &AchievementSpecs,
    context: &AchievementContext<'_>,
) -> bool {
    achievement_progress(master_store, specs, context) >= specs.required_goal_count()
}

fn achievement_progress(
    master_store: &MasterStore,
    specs: &AchievementSpecs,
    context: &AchievementContext<'_>,
) -> usize {
    aggregate_achievement_progress(
        specs,
        specs
            .goals
            .iter()
            .map(|goal| goal_progress(master_store, goal, context)),
    )
}

fn aggregate_achievement_progress(
    specs: &AchievementSpecs,
    goal_progress: impl IntoIterator<Item = usize>,
) -> usize {
    let progress = if specs.goals_amount.is_some() {
        goal_progress
            .into_iter()
            .zip(&specs.goals)
            .filter(|(progress, goal)| *progress >= goal.required_amount())
            .count()
    } else {
        goal_progress.into_iter().sum()
    };

    progress.min(specs.required_goal_count())
}

fn goal_progress(
    master_store: &MasterStore,
    goal: &AchievementGoal,
    context: &AchievementContext<'_>,
) -> usize {
    match goal {
        AchievementGoal::AreaLevel { value, area_id } => area_id
            .as_ref()
            .map(|id| context.area_levels.get(id).copied().unwrap_or_default() >= *value)
            .unwrap_or_else(|| context.area_levels.values().any(|level| level >= value))
            as usize,
        AchievementGoal::PowerLevel(value) => (context.power_level >= *value) as usize,
        AchievementGoal::PlayerLevel(value) => (context.player_level >= *value) as usize,
        AchievementGoal::SkillLevel {
            value,
            skill_id,
            skill_type,
            amount,
        } => {
            let required = usize::from(*amount);
            context
                .skill_masteries
                .masteries
                .iter()
                .filter(|(id, mastery)| {
                    skill_id.as_ref().is_none_or(|required| required == *id)
                        && skill_type.is_none_or(|required| {
                            master_store
                                .skills_store
                                .get(*id)
                                .is_some_and(|skill| skill.skill_type == required)
                        })
                        && master_store
                            .skill_masteries_store
                            .get(*id)
                            .is_some_and(|specs| mastery.level(specs.max_level) >= *value as u16)
                })
                .count()
                .min(required)
        }
        AchievementGoal::PassiveLevel {
            value,
            passive_id,
            amount,
        } => {
            let required = usize::from(*amount);
            context
                .ascension
                .ascended_nodes
                .iter()
                .filter(|(id, level)| {
                    **level >= *value && passive_id.is_none_or(|required| **id == required)
                })
                .count()
                .min(required)
        }
        AchievementGoal::EquippedItem {
            item_id,
            item_slot,
            item_level,
            item_rarity,
            item_category,
            in_passives_tree,
            amount,
        } => {
            let matches = |item: &ItemSpecs| {
                // let Some(item_base) = master_store
                //     .items_store
                //     .items
                //     .get(&item.modifiers.base_item_id)
                // else {
                //     return false;
                // };
                item_level.is_none_or(|required| item.modifiers.level >= required)
                    && item_rarity.is_none_or(|required| item.modifiers.rarity == required)
                    && item_id
                        .as_ref()
                        .is_none_or(|id| item.modifiers.base_item_id == *id)
                    && item_category.is_none_or(|required| item.categories.contains(&required))
            };
            let required = *amount as usize;

            let progress = if *in_passives_tree {
                if item_slot.is_none() {
                    context
                        .ascension
                        .socketed_nodes
                        .values()
                        .filter(|item| matches(item))
                        .count()
                } else {
                    0
                }
            } else if let Some(slot) = *item_slot {
                usize::from(
                    context
                        .inventory
                        .get_equipped_item(slot)
                        .is_some_and(&matches),
                )
            } else {
                context
                    .inventory
                    .equipped_items()
                    .filter(|(_, item)| matches(item))
                    .count()
            };

            progress.min(required)
        }
    }
}

async fn unlock_achievement(
    tx: &mut Transaction<'_, Database>,
    // achievements_store: &AchievementsStore,
    user_id: UserId,
    achievement_id: &str,
) -> Result<bool, AppError> {
    Ok(db::user_unlocks::unlock_achievement(tx, &user_id, achievement_id).await?)

    // let achievement = achievements_store
    //     .get(achievement_id)
    //     .ok_or(AppError::NotFound)?;

    // if !db::user_unlocks::unlock_achievement(tx, &user_id, achievement_id).await? {
    //     return Ok(false);
    // }

    // for reward in achievement.rewards.iter() {
    //     match reward {
    //         AchievementReward::Cosmetic(cosmetic_id) => {
    //             db::user_unlocks::unlock_cosmetic(tx, &user_id, cosmetic_id).await?;
    //         }
    //         AchievementReward::Pet(pet_id) => {
    //             db::user_unlocks::unlock_pet(tx, &user_id, pet_id).await?;
    //         }
    //     }
    // }

    // Ok(true)
}

pub async fn reward_achievement(
    tx: &mut Transaction<'_, Database>,
    achievements_store: &AchievementsStore,
    user_id: UserId,
    achievement_id: &str,
) -> Result<(), AppError> {
    let achievement = achievements_store
        .get(achievement_id)
        .ok_or(AppError::NotFound)?;

    for reward in achievement.rewards.iter() {
        match reward {
            AchievementReward::Cosmetic(cosmetic_id) => {
                db::user_unlocks::unlock_cosmetic(tx, &user_id, cosmetic_id).await?;
            }
            AchievementReward::Pet(pet_id) => {
                db::user_unlocks::unlock_pet(tx, &user_id, pet_id).await?;
            }
        }
    }

    Ok(())
}
