use sqlx::FromRow;

use shared::data::user::{UserCharacterId, UserId};

use crate::db::utc_datetime::UtcDateTime;

use super::pool::DbPool;

pub type SessionId = i64;

#[derive(Debug, FromRow)]
pub struct SessionEntry {
    pub session_id: SessionId,

    pub character_id: UserCharacterId,

    pub created_at: UtcDateTime,
    pub ended_at: UtcDateTime,
}

#[derive(Debug, FromRow)]
pub struct SessionGlimpse {
    pub user_id: UserId,
    pub username: Option<String>,
    pub character_id: UserCharacterId,
    pub character_name: String,
    pub created_at: UtcDateTime,
    pub area_id: String,
    pub area_level: i32,
}

/// Return Ok(None) if session already exist
pub async fn create_session(
    db_pool: &DbPool,
    character_id: &UserCharacterId,
) -> Result<Option<SessionId>, sqlx::Error> {
    let res = sqlx::query_scalar!(
        "
        INSERT INTO game_sessions (character_id)
        VALUES ($1)
        RETURNING session_id
        ",
        character_id
    )
    .fetch_one(db_pool)
    .await;

    match res {
        Ok(session_id) => Ok(Some(session_id)),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => Ok(None),
        Err(e) => Err(e),
    }
}

pub async fn count_active_sessions(db_pool: &DbPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!" 
        FROM game_sessions
        WHERE ended_at = '9999-01-01 23:59:59'
        "#,
    )
    .fetch_one(db_pool)
    .await
}

pub async fn glimpse_active_sessions(
    db_pool: &DbPool,
    limit: i64,
) -> Result<Vec<SessionGlimpse>, sqlx::Error> {
    sqlx::query_as!(
        SessionGlimpse,
        r#"
        SELECT
            users.user_id as "user_id: UserId",
            users.username,
            characters.character_id as "character_id: UserCharacterId",
            characters.character_name,
            game_sessions.created_at,
            area_id,
            area_level as "area_level: i32"
        FROM game_sessions
        INNER JOIN characters ON characters.character_id = game_sessions.character_id
        INNER JOIN users ON characters.user_id = users.user_id
        INNER JOIN saved_game_instances ON saved_game_instances.character_id = game_sessions.character_id
        WHERE ended_at = '9999-01-01 23:59:59'
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(db_pool)
    .await
}

pub async fn end_session(
    db_pool: &DbPool,
    character_id: &UserCharacterId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE game_sessions
        SET ended_at = CURRENT_TIMESTAMP
        WHERE character_id = $1 AND ended_at = '9999-01-01 23:59:59'
        "#,
        character_id,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn clean_all_sessions(db_pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE game_sessions
        SET ended_at = CURRENT_TIMESTAMP
        WHERE ended_at = '9999-01-01 23:59:59'
    "#,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
