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

impl Into<LeaderboardEntry> for db::leaderboard::LeaderboardEntry {
    fn into(self) -> LeaderboardEntry {
        LeaderboardEntry {
            user_id: self.user_id,
            username: self.username.unwrap_or_default(),
            character_id: self.character_id,
            character_name: self.character_name,
            area_id: self.area_id,
            area_level: self.area_level,
            created_at: self.created_at.into(),
            comments: "".to_string(),
        }
    }
}
