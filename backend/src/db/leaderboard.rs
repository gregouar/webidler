use shared::data::world::AreaLevel;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::FromRow;
use std::time::Duration;

use super::pool::DbPool;

#[derive(Debug, FromRow)]
pub struct LeaderboardEntry {
    pub id: i32,
    pub player_name: String,
    pub area_level: i32,
    pub time_played_seconds: i32,
    pub created_at: DateTime<Utc>,
    pub comments: String,
}

pub async fn insert_leaderboard_entry(
    pool: &DbPool,
    player_name: &str,
    area_level: AreaLevel,
    time_played: Duration,
    comments: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO leaderboard (player_name, area_level, time_played_seconds, comments)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(player_name)
    .bind(area_level as i32)
    .bind(time_played.as_secs() as i32)
    .bind(comments)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_top_leaderboard(
    pool: &DbPool,
    limit: i64,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    let entries = sqlx::query_as(
        r#"
        SELECT id, player_name, area_level, time_played_seconds, created_at,comments
        FROM leaderboard
        ORDER BY area_level DESC, time_played_seconds ASC,created_at ASC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(entries)
}
