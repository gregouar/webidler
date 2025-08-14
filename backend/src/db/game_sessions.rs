use sqlx::types::chrono::{DateTime, Utc};
use sqlx::FromRow;

use shared::data::user::UserCharacterId;

use super::pool::DbPool;

pub type SessionId = i64;

#[derive(Debug, FromRow)]
pub struct SessionEntry {
    pub session_id: SessionId,

    pub character_id: UserCharacterId,

    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
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
        WHERE ended_at IS NULL
        "#,
    )
    .fetch_one(db_pool)
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
        WHERE character_id = $1 AND ended_at IS NULL
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
        WHERE ended_at IS NULL
    "#,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
