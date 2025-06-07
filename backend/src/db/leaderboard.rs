use shared::data::world::AreaLevel;
use shared::messages::SessionId;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::FromRow;
use std::time::Duration;

use super::pool::DbPool;

#[derive(Debug, FromRow)]
pub struct LeaderboardEntry {
    pub session_id: SessionId,
    pub player_name: String,
    pub area_level: i32,
    pub time_played_seconds: i32,
    pub created_at: DateTime<Utc>,
    pub comments: String,
}

pub async fn upsert_leaderboard_entry(
    db_pool: &DbPool,
    session_id: &SessionId,
    player_name: &str,
    area_level: AreaLevel,
    time_played: Duration,
    comments: &str,
) -> Result<(), sqlx::Error> {
    let area_level = area_level as i32;
    let time_played_seconds = time_played.as_secs() as i32;
    sqlx::query!(
        r#"
        INSERT INTO leaderboard (session_id, player_name, area_level, time_played_seconds, comments)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT(session_id)
        DO UPDATE SET
            area_level = EXCLUDED.area_level,
            time_played_seconds = EXCLUDED.time_played_seconds,
            created_at = CURRENT_TIMESTAMP,
            comments = EXCLUDED.comments;
        "#,
        session_id,
        player_name,
        area_level,
        time_played_seconds,
        comments
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn get_top_leaderboard(
    db_pool: &DbPool,
    limit: i64,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    let entries = sqlx::query_as(
        r#"
        SELECT *
        FROM leaderboard
        ORDER BY area_level DESC, time_played_seconds ASC,created_at ASC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(db_pool)
    .await?;

    Ok(entries)
}
