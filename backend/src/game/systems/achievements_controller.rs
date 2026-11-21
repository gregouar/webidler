use chrono::{DateTime, Utc};
use sqlx::Transaction;
use std::collections::HashMap;

use shared::data::{
    achievements::{AchievementCounter, AchievementGoal, AchievementReward, AchievementSpecs},
    area::AreaLevel,
    item::ItemSpecs,
    monster::MonsterRarity,
    passive::{PassivesTreeAscension, PurchasedNodes},
    player::{PlayerInventory, PlayerState},
    skill::TargetType,
    skill_mastery::PlayerSkillMasteries,
    user::{UserCharacterId, UserId},
};

use crate::{
    app_state::MasterStore,
    db::{self, characters::CharacterAreaEntry, pool::Database},
    game::data::master_store::AchievementsStore,
    rest::AppError,
};

pub struct AchievementContext<'a> {
    pub counters: &'a HashMap<AchievementCounter, u64>,
    pub area_levels: &'a HashMap<String, AreaLevel>,
    pub power_level: AreaLevel,
    pub player_level: u8,
    pub skill_masteries: &'a PlayerSkillMasteries,
    pub ascension: &'a PassivesTreeAscension,
    pub purchased_nodes: Option<&'a PurchasedNodes>,
    pub inventory: &'a PlayerInventory,
    pub area_statistics: AreaAchievementStatistics<'a>,
    pub player_state: Option<&'a PlayerState>,
}

#[derive(Clone, Copy)]
struct AreaAchievementStatisticsEntry<'a> {
    pub area_id: &'a str,
    pub monsters_killed: i64,
    pub champion_monsters_killed: i64,
    pub bosses_killed: i64,
    pub areas_completed: i64,
    pub edicts_spent: i64,
    pub gems_obtained: i64,
}

impl<'a> From<&'a CharacterAreaEntry> for AreaAchievementStatisticsEntry<'a> {
    fn from(area: &'a CharacterAreaEntry) -> Self {
        Self {
            area_id: &area.area_id,
            monsters_killed: area.monsters_killed,
            champion_monsters_killed: area.champion_monsters_killed,
            bosses_killed: area.bosses_killed,
            areas_completed: area.areas_completed,
            edicts_spent: area.edicts_spent,
            gems_obtained: area.gems_obtained,
        }
    }
}

#[derive(Clone, Copy)]
pub struct AreaAchievementStatistics<'a> {
    persisted: &'a [CharacterAreaEntry],
    current: Option<AreaAchievementStatisticsEntry<'a>>,
}

impl<'a> AreaAchievementStatistics<'a> {
    pub fn persisted(persisted: &'a [CharacterAreaEntry]) -> Self {
        Self {
            persisted,
            current: None,
        }
    }

    pub fn with_current_grind(
        persisted: &'a [CharacterAreaEntry],
        area_id: &'a str,
        game_stats: &shared::data::game_stats::GameStats,
        gems_obtained: f64,
        with_edict: bool,
    ) -> Self {
        let to_i64 = |value: u64| value.min(i64::MAX as u64) as i64;
        Self {
            persisted,
            current: Some(AreaAchievementStatisticsEntry {
                area_id,
                monsters_killed: to_i64(game_stats.monsters_killed),
                champion_monsters_killed: to_i64(game_stats.champion_monsters_killed),
                bosses_killed: to_i64(game_stats.bosses_killed),
                areas_completed: to_i64(game_stats.areas_completed),
                edicts_spent: i64::from(with_edict),
                gems_obtained: gems_obtained.min(i64::MAX as f64) as i64,
            }),
        }
    }

    fn iter(&self) -> impl Iterator<Item = AreaAchievementStatisticsEntry<'_>> + '_ {
        self.persisted
            .iter()
            .map(AreaAchievementStatisticsEntry::from)
            .chain(self.current)
    }
}

pub async fn unlock_achievements(
    tx: &mut Transaction<'_, Database>,
    user_id: UserId,
    character_id: UserCharacterId,
    new_achievements: &[String],
) -> Result<(), AppError> {
    for achievement_id in new_achievements.iter() {
        db::user_unlocks::unlock_achievement(tx, &user_id, achievement_id).await?;
        db::character_achievements::unlock_achievement(tx, &character_id, achievement_id).await?;
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
) -> HashMap<String, u64> {
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
) -> u64 {
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
    goal_progress: impl IntoIterator<Item = u64>,
) -> u64 {
    let progress = if specs.goals_amount.is_some() {
        goal_progress
            .into_iter()
            .zip(&specs.goals)
            .filter(|(progress, goal)| *progress >= goal.required_amount())
            .count() as u64
    } else {
        goal_progress.into_iter().sum()
    };

    progress.min(specs.required_goal_count())
}

fn goal_progress(
    master_store: &MasterStore,
    goal: &AchievementGoal,
    context: &AchievementContext<'_>,
) -> u64 {
    match goal {
        AchievementGoal::Counter { counter, value: _ } => {
            context.counters.get(counter).copied().unwrap_or_default()
        }
        AchievementGoal::AreaLevel { value, area_id } => area_id
            .as_ref()
            .map(|id| context.area_levels.get(id).copied().unwrap_or_default() >= *value)
            .unwrap_or_else(|| context.area_levels.values().any(|level| level >= value))
            as u64,
        AchievementGoal::PowerLevel(value) => (context.power_level >= *value) as u64,
        AchievementGoal::PlayerLevel(value) => (context.player_level >= *value) as u64,
        AchievementGoal::SkillLevel {
            value,
            skill_id,
            skill_type,
            amount,
        } => {
            let required = u64::from(*amount);
            (context
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
                .count() as u64)
                .min(required)
        }
        AchievementGoal::PassiveLevel {
            value,
            passive_id,
            amount,
        } => {
            let required = u64::from(*amount);
            (context
                .ascension
                .ascended_nodes
                .iter()
                .filter(|(id, level)| {
                    **level >= *value && passive_id.is_none_or(|required| **id == required)
                })
                .count() as u64)
                .min(required)
        }
        AchievementGoal::PassiveAllocated { amount, socket } => {
            let required = u64::from(*amount);
            (context
                .purchased_nodes
                .map(|purchased_nodes| {
                    if *socket {
                        purchased_nodes
                            .iter()
                            .filter(|passive_id| {
                                context.ascension.socketed_nodes.contains_key(*passive_id)
                            })
                            .count()
                    } else {
                        purchased_nodes.len()
                    }
                })
                .unwrap_or_default() as u64)
                .min(required)
        }
        AchievementGoal::EquippedItem {
            item_id,
            item_slot,
            item_level,
            item_quality,
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
                    && item_quality.is_none_or(|required| item.modifiers.quality >= required)
                    && item_rarity.is_none_or(|required| item.modifiers.rarity == required)
                    && item_id
                        .as_ref()
                        .is_none_or(|id| item.modifiers.base_item_id == *id)
                    && item_category.is_none_or(|required| item.categories.contains(&required))
            };
            let required = *amount as u64;

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

            (progress as u64).min(required)
        }
        AchievementGoal::MonstersKilled {
            monster_rarity,
            area_id,
            ..
        } => match monster_rarity {
            Some(rarity) => match rarity {
                MonsterRarity::Normal => {
                    sum_area_stat(context, area_id.as_deref(), |area| area.monsters_killed)
                }
                MonsterRarity::Champion => sum_area_stat(context, area_id.as_deref(), |area| {
                    area.champion_monsters_killed
                }),
                MonsterRarity::Boss => {
                    sum_area_stat(context, area_id.as_deref(), |area| area.bosses_killed)
                }
            },
            None => sum_area_stat(context, area_id.as_deref(), |area| area.monsters_killed),
        },
        AchievementGoal::AreaCompleted {
            area_id,
            with_edict,
            ..
        } => sum_area_stat(context, area_id.as_deref(), |area| {
            if *with_edict {
                area.edicts_spent
            } else {
                area.areas_completed
            }
        }),
        AchievementGoal::GemsObtained { area_id, .. } => {
            sum_area_stat(context, area_id.as_deref(), |area| area.gems_obtained)
        }
        AchievementGoal::CharacterStatusStacks {
            value: _,
            status_id,
            target,
        } => match target {
            TargetType::Enemy => todo!(),
            TargetType::Friend | TargetType::Me => context
                .player_state
                .map(|player_state| {
                    player_state
                        .character_state
                        .statuses
                        .get(status_id)
                        .map(|status_stacks| status_stacks.len() as u64)
                })
                .flatten()
                .unwrap_or_default(),
        },
    }
}

fn sum_area_stat(
    context: &AchievementContext<'_>,
    area_id: Option<&str>,
    value: impl Fn(AreaAchievementStatisticsEntry<'_>) -> i64,
) -> u64 {
    context
        .area_statistics
        .iter()
        .filter(|area| area_id.is_none_or(|id| area.area_id == id))
        .fold(0_u64, |total, area| {
            total.saturating_add(value(area) as u64)
        })
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
