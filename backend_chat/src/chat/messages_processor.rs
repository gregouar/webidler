use axum::body::Bytes;
use backend_shared::{http::users::UserId, profanities_checker::ProfanitiesChecker};
use dashmap::DashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

use shared_chat::{
    messages::server::{ErrorMessage, ErrorType, ServerChatMessage},
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
                history: Arc::new(Mutex::new(RingBuffer::new(100))),
            },
            profanities_checker,
        }
    }

    pub fn get_chat_state(&self) -> ChatState {
        self.chat_state.clone()
    }
    // pub fn get_sender(&self) -> mpsc::Sender<ChatMessage> {
    //     self.input_tx.clone()
    // }

    pub async fn run(mut self) {
        // tokio::spawn(async move {
        let user_states: DashMap<UserId, UserModerationState> = DashMap::new();

        while let Some((session_id, msg)) = self.inbound_rx.recv().await {
            if msg.channel == ChatChannel::System {
                send_direct_error(&self.chat_state, session_id, "Cannot send to that channel.")
                    .await;
                continue;
            }

            if let Some(user_id) = msg.user_id {
                let mut user_moderation = user_states.entry(user_id).or_default();

                if user_moderation.is_muted() {
                    send_direct_error(&self.chat_state, session_id, "You are muted.").await;
                    continue;
                }

                if !user_moderation.allow_message() {
                    send_direct_error(&self.chat_state, session_id, "Rate limited.").await;
                    continue;
                }
            }

            let content = if self.profanities_checker.contains_profanities(&msg.content) {
                "***"
            } else {
                &msg.content
            };

            if let Ok(content) = ChatContent::try_new(content) {
                let chat_message = ChatMessage { content, ..msg };
                self.chat_state
                    .history
                    .lock()
                    .unwrap()
                    .push(Arc::new(chat_message.clone()));
                if let Ok(ser_message) =
                    rmp_serde::to_vec(&ServerChatMessage::Broadcast(chat_message.into()))
                {
                    let message = Arc::new(Bytes::from(ser_message));
                    let _ = self.chat_state.outbound_tx.send(message);
                }

                // let message: Arc<ServerChatMessage> =
                //     Arc::new(ServerChatMessage::Broadcast(chat_message.into()));
                // self.chat_state
                //     .history
                //     .lock()
                //     .unwrap()
                //     .push(message.clone());
                // let _ = self.chat_state.outbound_tx.send(message);
            }
        }
    }
    //);
    // }
}

async fn send_direct_error(chat_state: &ChatState, session_id: Uuid, msg: &str) {
    if let Some(reply_queue) = chat_state.reply_map.get(&session_id) {
        let _ = reply_queue
            .send(
                ErrorMessage {
                    error_type: ErrorType::Chat,
                    message: msg.into(),
                    must_disconnect: false,
                }
                .into(),
            )
            .await;
    }
}
