use axum::body::Bytes;
use backend_shared::{http::users::UserId, profanities_checker::ProfanitiesChecker};
use dashmap::DashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

use shared_chat::{
    messages::server::{ErrorMessage, ErrorType, ServerChatMessage, ServerWhisperFeedbackMessage},
    ring_buffer::RingBuffer,
    types::{ChatChannel, ChatContent, ChatMessage},
};

use crate::chat::{chat_state::ChatState, user_moderation::UserModerationState};

pub struct MessagesProcessor {
    inbound_rx: mpsc::Receiver<(Uuid, ChatMessage)>,

    chat_state: ChatState,
    profanities_checker: ProfanitiesChecker,
    // TODO: Banned, Muted, SpamBucket in some Moderation thingy?
}

impl MessagesProcessor {
    pub fn new(profanities_checker: ProfanitiesChecker) -> Self {
        let (inbound_tx, inbound_rx) = mpsc::channel(1000);
        let (outbound_tx, _) = broadcast::channel(500);
        Self {
            inbound_rx,
            chat_state: ChatState {
                inbound_tx,
                outbound_tx,
                reply_map: Default::default(),
                users_map: Default::default(),
                usernames_map: Default::default(),
                history: Arc::new(Mutex::new(RingBuffer::new(100))),
            },
            profanities_checker,
        }
    }

    pub fn get_chat_state(&self) -> ChatState {
        self.chat_state.clone()
    }

    pub async fn run(mut self) {
        let user_states: DashMap<UserId, UserModerationState> = DashMap::new();

        while let Some((session_id, msg)) = self.inbound_rx.recv().await {
            if msg.channel == ChatChannel::System {
                send_direct_error(&self.chat_state, session_id, "cannot send to that channel")
                    .await;
                continue;
            }

            if let Some(user_id) = msg.user_id {
                let mut user_moderation = user_states.entry(user_id).or_default();

                if user_moderation.is_muted() {
                    send_direct_error(&self.chat_state, session_id, "you are muted").await;
                    continue;
                }

                if !user_moderation.allow_message() {
                    send_direct_error(&self.chat_state, session_id, "rate limited").await;
                    continue;
                }
            }

            let (content, channel, target_username) =
                if let Some((username, message)) = parse_whisper_message(&msg.content) {
                    if let Some(entry) = self.chat_state.usernames_map.get(&username) {
                        let (user_id, target_username) = entry.value();
                        (
                            message,
                            ChatChannel::Whisper(*user_id),
                            Some(target_username.clone()),
                        )
                    } else {
                        send_direct_error(
                            &self.chat_state,
                            session_id,
                            "unknown user or not connected",
                        )
                        .await;

                        continue;
                    }
                } else {
                    (msg.content, msg.channel, None)
                };

            let content = if self.profanities_checker.contains_profanities(&content) {
                ChatContent::try_new("***").unwrap_or_default()
            } else {
                content
            };

            let chat_message = ChatMessage {
                content,
                channel,
                ..msg
            };
            let server_chat_message = ServerChatMessage::Broadcast(chat_message.clone().into());

            if let ChatChannel::Whisper(user_id) = channel {
                if let Some(targets) = self.chat_state.users_map.get(&user_id)
                    && !targets.is_empty()
                {
                    for target_session_id in targets.iter() {
                        send_direct_message(
                            &self.chat_state,
                            *target_session_id,
                            server_chat_message.clone(),
                        )
                        .await;
                    }
                    send_direct_message(
                        &self.chat_state,
                        session_id,
                        ServerWhisperFeedbackMessage {
                            target_username,
                            target_user_id: user_id,
                            chat_message,
                        }
                        .into(),
                    )
                    .await;
                } else {
                    send_direct_error(
                        &self.chat_state,
                        session_id,
                        "whisper target user not connected",
                    )
                    .await;
                }
            } else {
                self.chat_state
                    .history
                    .lock()
                    .unwrap()
                    .push(Arc::new(chat_message));
                if let Ok(ser_message) = rmp_serde::to_vec(&server_chat_message) {
                    let message = Arc::new(Bytes::from(ser_message));
                    let _ = self.chat_state.outbound_tx.send(message);
                }
            }
        }
    }
}

async fn send_direct_error(chat_state: &ChatState, session_id: Uuid, msg: &str) {
    send_direct_message(
        chat_state,
        session_id,
        ErrorMessage {
            error_type: ErrorType::Chat,
            message: msg.into(),
            must_disconnect: false,
        }
        .into(),
    )
    .await;
}

async fn send_direct_message(chat_state: &ChatState, session_id: Uuid, msg: ServerChatMessage) {
    if let Some(reply_queue) = chat_state.reply_map.get(&session_id) {
        let _ = reply_queue.send(msg).await;
    }
}

fn parse_whisper_message(content: &str) -> Option<(String, ChatContent)> {
    if !content.starts_with('@') {
        return None;
    }

    let mut parts = content[1..].splitn(2, ' ');

    let username = parts.next()?.trim().to_string();
    let message = parts.next()?.trim().to_string();

    if username.is_empty() || message.is_empty() {
        return None;
    }

    Some((
        username.to_ascii_lowercase(),
        ChatContent::try_new(message).ok()?,
    ))
}
