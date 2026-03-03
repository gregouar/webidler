use anyhow::{Result, anyhow};

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::post,
};
use backend_shared::http::users::UserId;
use chrono::Utc;
use shared_chat::{
    messages::{client::ClientPostMessage, server::ServerChatMessage},
    types::ChatMessage,
};

use crate::{app_state::AppState, chat::chat_state::ChatState};

use super::AppError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/message/{user_id}", post(post_private_message))
        .route("/message", post(post_broadcast_message))
}

async fn post_private_message(
    State(chat_state): State<ChatState>,
    Path(user_id): Path<UserId>,
    Json(payload): Json<ClientPostMessage>,
) -> Result<Json<()>, AppError> {
    let user_sessions = chat_state
        .users_map
        .get(&user_id)
        .ok_or(AppError::NotFound)?;

    let server_chat_message = ServerChatMessage::Broadcast(
        ChatMessage {
            channel: payload.channel,
            sent_at: Utc::now(),
            user_id: None,
            username: None,
            content: payload.content,
            linked_item: payload.linked_item,
        }
        .into(),
    );

    for session_id in user_sessions.iter() {
        if let Some(reply_queue) = chat_state.reply_map.get(session_id) {
            let _ = reply_queue.send(server_chat_message.clone()).await;
        }
    }

    Ok(Json(()))
}

async fn post_broadcast_message(
    State(chat_state): State<ChatState>,
    Json(payload): Json<ClientPostMessage>,
) -> Result<Json<()>, AppError> {
    chat_state
        .inbound_tx
        .send((
            Default::default(),
            ChatMessage {
                channel: payload.channel,
                sent_at: Utc::now(),
                user_id: None,
                username: None,
                content: payload.content,
                linked_item: payload.linked_item,
            },
        ))
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(Json(()))
}
