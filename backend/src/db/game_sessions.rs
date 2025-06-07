use shared::messages::{SessionId, UserId};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::FromRow;

use super::pool::DbPool;

#[derive(Debug, FromRow)]
pub struct SessionEntry {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

pub async fn create_session(db_pool: &DbPool, user_id: &UserId) -> Result<SessionId, sqlx::Error> {
    sqlx::query_scalar!(
        "
        INSERT INTO game_sessions (user_id)
        VALUES ($1)
        RETURNING session_id
        ",
        user_id
    )
    .fetch_one(db_pool)
    .await
}

pub async fn is_user_in_session(db_pool: &DbPool, user_id: &UserId) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
        SELECT session_id
        FROM game_sessions
        WHERE user_id = $1 AND ended_at IS NULL
        "#,
        user_id
    )
    .fetch_optional(db_pool)
    .await?
    .is_some())
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

pub async fn end_session(db_pool: &DbPool, session_id: &SessionId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE game_sessions
        SET ended_at = CURRENT_TIMESTAMP
        WHERE session_id = $1
        "#,
        session_id,
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
