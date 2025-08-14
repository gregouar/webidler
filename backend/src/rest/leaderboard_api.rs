use anyhow::Result;

use axum::{extract::State, routing::get, Json, Router};

use shared::http::server::{LeaderboardEntry, LeaderboardResponse};

use crate::{
    app_state::{AppState, DbPool},
    db,
};

use super::AppError;

pub fn routes() -> Router<AppState> {
    Router::new().route("/leaderboard", get(get_leaderboard))
}

pub async fn get_leaderboard(
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
            area_level: val.area_level,
            created_at: val.created_at.into(),
            comments: "".to_string(),
        }
    }
}
