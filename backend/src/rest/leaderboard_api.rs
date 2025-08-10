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
    // TODO: new leaderboard from characters table
    Ok(Json(LeaderboardResponse { entries: vec![] }))
}
