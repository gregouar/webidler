use std::time::Duration;

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
        entries: db::leaderboard::get_top_leaderboard(&db_pool, 10)
            .await?
            .into_iter()
            .map(|x| LeaderboardEntry {
                player_name: x.player_name,
                area_level: x.area_level as u16,
                time_played: Duration::from_secs(x.time_played_seconds as u64),
                created_at: x.created_at,
                comments: x.comments,
            })
            .collect(),
    }))
}
