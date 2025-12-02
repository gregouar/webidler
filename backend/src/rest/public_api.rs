use std::time::Duration;

use anyhow::Result;

use axum::{extract::State, routing::get, Json, Router};

use shared::{
    data::area::AreaLevel,
    http::server::{LeaderboardEntry, LeaderboardResponse, NewsResponse, PlayersCountResponse},
};

use crate::{
    app_state::{AppState, DbPool},
    db,
    integration::discord::DiscordState,
};

use super::AppError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/leaderboard", get(get_leaderboard))
        .route("/players", get(get_players_count))
        .route("/news", get(get_news))
}

async fn get_news(State(discord): State<DiscordState>) -> Result<Json<NewsResponse>, AppError> {
    Ok(Json(NewsResponse {
        entries: discord.get_news().await?,
    }))
}

async fn get_players_count(
    State(db_pool): State<db::DbPool>,
) -> Result<Json<PlayersCountResponse>, AppError> {
    let value = db::game_sessions::count_active_sessions(&db_pool).await?;
    let glimpse = db::game_sessions::glimpse_active_sessions(&db_pool, 10)
        .await?
        .into_iter()
        .map(|entry| LeaderboardEntry {
            user_id: entry.user_id,
            username: entry.username.unwrap_or("Hidden User".into()),
            character_id: entry.character_id,
            character_name: entry.character_name,
            area_id: entry.area_id,
            area_level: entry.area_level as u16,
            created_at: entry.created_at.into(),
            elapsed_time: None,
            comments: "".into(),
        })
        .collect();
    Ok(Json(PlayersCountResponse { value, glimpse }))
}

async fn get_leaderboard(
    State(db_pool): State<DbPool>,
) -> Result<Json<LeaderboardResponse>, AppError> {
    Ok(Json(LeaderboardResponse {
        entries: db::leaderboard::get_leaderboard(&db_pool, 10)
            .await?
            .into_iter()
            .map(|entry| entry.into())
            .collect(),
    }))
}

impl From<db::leaderboard::LeaderboardEntry> for LeaderboardEntry {
    fn from(val: db::leaderboard::LeaderboardEntry) -> Self {
        LeaderboardEntry {
            user_id: val.user_id,
            username: val.username.unwrap_or_default(),
            character_id: val.character_id,
            character_name: val.character_name,
            area_id: val.area_id,
            area_level: val.area_level as AreaLevel,
            created_at: val.created_at.into(),
            elapsed_time: val.elapsed_time.map(Duration::from_secs_f64),
            comments: "".to_string(),
        }
    }
}
