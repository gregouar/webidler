use std::sync::Arc;

use tokio::sync::{broadcast, mpsc};

use shared::{
    messages::{
        chat::{ChatChannel, ChatMessage},
        server::{ErrorMessage, ErrorType},
    },
    types::ChatContent,
};
use uuid::Uuid;

use crate::chat::chat_state::ChatState;

pub struct MessagesProcessor {
    inbound_rx: mpsc::Receiver<(Uuid, ChatMessage)>,

    chat_state: ChatState,
    // TODO: Banned, Muted, SpamBucket in some Moderation thingy?
}

impl MessagesProcessor {
    pub fn new() -> Self {
        let (inbound_tx, inbound_rx) = mpsc::channel(1000);
        let (outbound_tx, _) = broadcast::channel(500);
        Self {
            inbound_rx,
            chat_state: ChatState {
                inbound_tx,
                outbound_tx,
                reply_map: Default::default(),
            },
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
        // let user_states: DashMap<UserId, UserModerationState> = DashMap::new();

        while let Some((session_id, msg)) = self.inbound_rx.recv().await {
            if msg.channel == ChatChannel::System {
                if let Some(reply_queue) = self.chat_state.reply_map.get(&session_id) {
                    let _ = reply_queue
                        .send(
                            ErrorMessage {
                                error_type: ErrorType::Chat,
                                message: "cannot send to that channel".into(),
                                must_disconnect: false,
                            }
                            .into(),
                        )
                        .await;
                }
                continue;
            }

            // let mut entry = user_states
            //     .entry(msg.user_id)
            //     .or_insert_with(UserModerationState::new);

            // if entry.is_muted() {
            //     send_direct_error(&state, msg.user_id, "You are muted.");
            //     continue;
            // }

            // if !entry.allow_message() {
            //     send_direct_error(&state, msg.user_id, "Rate limited.");
            //     continue;
            // }

            let filtered = profanity_filter(&msg.content);

            if let Ok(content) = ChatContent::try_new(filtered) {
                let server_msg = Arc::new(ChatMessage { content, ..msg });
                let _ = self.chat_state.outbound_tx.send(server_msg);
            }
        }
    }
    //);
    // }
}

fn profanity_filter(input: &str) -> String {
    input.replace("pou", "***")
}
