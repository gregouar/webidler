use chrono::{DateTime, Utc};
use sqlx::Transaction;
use std::collections::HashMap;

use shared::data::user::UserCharacterId;

use crate::{
    constants::DATA_VERSION,
    db::{
        pool::{Database, DbExecutor},
        utc_datetime::UtcDateTime,
    },
};

pub async fn read_achievements<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
) -> Result<HashMap<String, DateTime<Utc>>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
        SELECT
            achievement_id,
            unlocked_at as "unlocked_at: UtcDateTime"
        FROM character_achievements
        WHERE character_id = $1
        "#,
        character_id
    )
    .fetch_all(executor)
    .await?
    .into_iter()
    .map(|row| (row.achievement_id, row.unlocked_at.into()))
    .collect())
}

pub async fn unlock_achievement(
    tx: &mut Transaction<'_, Database>,
    character_id: &UserCharacterId,
    achievement_id: &str,
) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query!(
        "INSERT INTO character_achievements (character_id, achievement_id, data_version)
         VALUES ($1, $2, $3)
         ON CONFLICT(character_id, achievement_id) DO NOTHING",
        character_id,
        achievement_id,
        DATA_VERSION,
    )
    .execute(&mut **tx)
    .await?
    .rows_affected()
        > 0)
}
