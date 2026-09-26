use chrono::{DateTime, Utc};
use sqlx::Transaction;
use std::collections::{HashMap, HashSet};

use shared::data::user::UserId;

use crate::db::{
    pool::{Database, DbExecutor},
    utc_datetime::UtcDateTime,
};

pub async fn read_achievements<'c>(
    executor: impl DbExecutor<'c>,
    user_id: &UserId,
) -> Result<HashMap<String, DateTime<Utc>>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
        SELECT
            achievement_id,
            unlocked_at as "unlocked_at: UtcDateTime"
        FROM user_achievements
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_all(executor)
    .await?
    .into_iter()
    .map(|row| (row.achievement_id, row.unlocked_at.into()))
    .collect())
}

pub async fn read_cosmetics<'c>(
    executor: impl DbExecutor<'c>,
    user_id: &UserId,
) -> Result<HashSet<String>, sqlx::Error> {
    Ok(sqlx::query_scalar!(
        "SELECT cosmetic_id FROM user_cosmetics WHERE user_id = $1",
        user_id
    )
    .fetch_all(executor)
    .await?
    .into_iter()
    .collect())
}

pub async fn read_pets<'c>(
    executor: impl DbExecutor<'c>,
    user_id: &UserId,
) -> Result<HashSet<String>, sqlx::Error> {
    Ok(
        sqlx::query_scalar::<_, String>("SELECT pet_id FROM user_pets WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(executor)
            .await?
            .into_iter()
            .collect(),
    )
}

pub async fn unlock_achievement(
    tx: &mut Transaction<'_, Database>,
    user_id: &UserId,
    achievement_id: &str,
) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query!(
            "INSERT INTO user_achievements (user_id, achievement_id) VALUES ($1, $2) ON CONFLICT(user_id, achievement_id) DO NOTHING",
            user_id,
            achievement_id,
        )
        .execute(&mut **tx)
        .await?
        .rows_affected()
            > 0)
}

pub async fn unlock_cosmetic(
    tx: &mut Transaction<'_, Database>,
    user_id: &UserId,
    cosmetic_id: &str,
) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query!(
            "INSERT INTO user_cosmetics (user_id, cosmetic_id) VALUES ($1, $2) ON CONFLICT(user_id, cosmetic_id) DO NOTHING",
            user_id,
            cosmetic_id,
        )
        .execute(&mut **tx)
        .await?
        .rows_affected()
            > 0)
}

pub async fn unlock_pet(
    tx: &mut Transaction<'_, Database>,
    user_id: &UserId,
    pet_id: &str,
) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query!(
            "INSERT INTO user_pets (user_id, pet_id) VALUES ($1, $2) ON CONFLICT(user_id, pet_id) DO NOTHING",
            user_id,
            pet_id,
        )
        .execute(&mut **tx)
        .await?
        .rows_affected()
            > 0)
}
