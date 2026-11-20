use chrono::{DateTime, Utc};
use sqlx::Transaction;
use std::collections::HashMap;

use shared::data::{achievements::AchievementCounter, user::UserCharacterId};

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

pub async fn read_counters<'c>(
    executor: impl DbExecutor<'c>,
    character_id: &UserCharacterId,
) -> Result<HashMap<AchievementCounter, u64>, sqlx::Error> {
    sqlx::query!(
        r#"
        SELECT
            counter_id,
            amount
        FROM character_achievements_counters
        WHERE character_id = $1
        "#,
        character_id
    )
    .fetch_all(executor)
    .await?
    .into_iter()
    .map(|row| {
        let counter = serde_plain::from_str::<AchievementCounter>(&row.counter_id)
            .map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
        let amount =
            u64::try_from(row.amount).map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
        Ok((counter, amount))
    })
    .collect::<Result<HashMap<_, _>, sqlx::Error>>()
}

pub async fn increase_counter(
    tx: &mut Transaction<'_, Database>,
    character_id: &UserCharacterId,
    counter: AchievementCounter,
    amount: u64,
) -> Result<bool, sqlx::Error> {
    let counter_id =
        serde_plain::to_string(&counter).map_err(|error| sqlx::Error::Encode(Box::new(error)))?;
    let amount = i64::try_from(amount).map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
    Ok(sqlx::query!(
        "INSERT INTO character_achievements_counters (character_id, counter_id, amount)
         VALUES ($1, $2, $3)
         ON CONFLICT(character_id, counter_id) DO UPDATE
         SET
            amount = character_achievements_counters.amount + EXCLUDED.amount,
            updated_at = CURRENT_TIMESTAMP;
        ",
        character_id,
        counter_id,
        amount,
    )
    .execute(&mut **tx)
    .await?
    .rows_affected()
        > 0)
}

pub async fn write_counters(
    tx: &mut Transaction<'_, Database>,
    character_id: &UserCharacterId,
    counters: &HashMap<AchievementCounter, u64>,
) -> Result<(), sqlx::Error> {
    for (counter, amount) in counters.iter() {
        let counter_id = serde_plain::to_string(counter)
            .map_err(|error| sqlx::Error::Encode(Box::new(error)))?;
        let amount =
            i64::try_from(*amount).map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
        sqlx::query!(
            "INSERT INTO character_achievements_counters (character_id, counter_id, amount)
            VALUES ($1, $2, $3)
            ON CONFLICT(character_id, counter_id) DO UPDATE
            SET
                amount = EXCLUDED.amount,
                updated_at = CURRENT_TIMESTAMP;
            ",
            character_id,
            counter_id,
            amount,
        )
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}
