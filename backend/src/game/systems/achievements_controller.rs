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

pub async fn update_achievements(
    tx: &mut Transaction<'_, Database>,
    master_store: &MasterStore,
    user_id: UserId,
    already_unlocked: &mut HashMap<String, DateTime<Utc>>,
    context: &AchievementContext<'_>,
) -> Result<Vec<String>, AppError> {
    let new_achievements = check_achievements(master_store, already_unlocked, context);

    for achievement_id in new_achievements.iter() {
        unlock_achievement(
            tx,
            &master_store.achievements_store,
            user_id,
            achievement_id,
        )
        .await?;
        already_unlocked.insert(achievement_id.clone(), Utc::now());
    }

    Ok(new_achievements)
}

fn check_achievements(
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

fn achievement_satisfied(
    master_store: &MasterStore,
    specs: &AchievementSpecs,
    context: &AchievementContext<'_>,
) -> bool {
    let required = specs
        .goals_amount
        .map(usize::from)
        .unwrap_or(specs.goals.len());
    specs
        .goals
        .iter()
        .filter(|goal| goal_satisfied(master_store, goal, context))
        .count()
        >= required
}

fn goal_satisfied(
    master_store: &MasterStore,
    goal: &AchievementGoal,
    context: &AchievementContext<'_>,
) -> bool {
    match goal {
        AchievementGoal::AreaLevel { value, area_id } => area_id
            .as_ref()
            .map(|id| context.area_levels.get(id).copied().unwrap_or_default() >= *value)
            .unwrap_or_else(|| context.area_levels.values().any(|level| level >= value)),
        AchievementGoal::PowerLevel(value) => context.power_level >= *value,
        AchievementGoal::PlayerLevel(value) => context.player_level >= *value,
        AchievementGoal::SkillLevel {
            value,
            skill_id,
            skill_type,
            amount,
        } => {
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
                >= *amount as usize
        }
        AchievementGoal::PassiveLevel {
            value,
            passive_id,
            amount,
        } => {
            context
                .ascension
                .ascended_nodes
                .iter()
                .filter(|(id, level)| {
                    **level >= *value && passive_id.is_none_or(|required| **id == required)
                })
                .count()
                >= *amount as usize
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
                item_level.is_none_or(|required| item.modifiers.level >= required)
                    && item_rarity.is_none_or(|required| item.modifiers.rarity == required)
                    && item_id
                        .as_ref()
                        .is_none_or(|id| item.modifiers.base_item_id == *id)
                    && item_category.is_none_or(|required| item.base.categories.contains(&required))
            };
            let required = *amount as usize;

            if *in_passives_tree {
                item_slot.is_none()
                    && context
                        .ascension
                        .socketed_nodes
                        .values()
                        .filter(|item| matches(item))
                        .count()
                        >= required
            } else if let Some(slot) = *item_slot {
                required == 1
                    && context
                        .inventory
                        .get_equipped_item(slot)
                        .is_some_and(|item_specs| matches(item_specs))
            } else {
                context
                    .inventory
                    .equipped_items()
                    .filter(|(slot, item)| matches(item))
                    .count()
                    >= required
            }
        }
    }
}

async fn unlock_achievement(
    tx: &mut Transaction<'_, Database>,
    achievements_store: &AchievementsStore,
    user_id: UserId,
    achievement_id: &str,
) -> Result<(), AppError> {
    let achievement = achievements_store
        .get(achievement_id)
        .ok_or(AppError::NotFound)?;

    if !db::user_unlocks::unlock_achievement(tx, &user_id, achievement_id).await? {
        return Ok(());
    }

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
