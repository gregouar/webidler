use std::time::Duration;

use anyhow::Result;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::{Deserialize, Serialize};

use shared_chat::types::UserId;

use crate::{app_state::AppState, chat::chat_state::ChatState};

use super::AppError;

pub fn routes() -> Router<AppState> {
    Router::new().route("/mute/{user_id}", get(mute_user))
}

async fn mute_user(
    State(chat_state): State<ChatState>,
    Path(user_id): Path<UserId>,
    Json(payload): Json<MuteUserRequest>,
) -> Result<Json<()>, AppError> {
    chat_state
        .users_moderation
        .entry(user_id)
        .or_default()
        .mute(payload.duration);
    // TODO: Send private message to user
    Ok(Json(()))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MuteUserRequest {
    #[serde(default = "default_mute_duration")]
    pub duration: Duration,
}

fn default_mute_duration() -> Duration {
    Duration::from_hours(1)
}
