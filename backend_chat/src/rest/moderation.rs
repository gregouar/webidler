use anyhow::Result;

use axum::{Json, Router, routing::get};

use crate::app_state::AppState;

use super::AppError;

pub fn routes() -> Router<AppState> {
    Router::new().route("/mute", get(mute_user))
}

async fn mute_user() -> Result<Json<()>, AppError> {
    Ok(Json(()))
}
