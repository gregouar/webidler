use anyhow::Result;
use backend_shared::http::users::UserDetails;
use chrono::Utc;
use std::ops::ControlFlow;
use tokio::sync::mpsc;
use uuid::Uuid;

use shared_chat::{
    messages::{
        client::{ClientChatMessage, ClientPostMessage},
        server::{ErrorMessage, ErrorType, ServerChatMessage, ServerConnectMessage},
    },
    types::{ChatMessage, UserId},
};

use crate::{
    chat::{character_resolver::CharacterResolver, chat_state::ChatState},
    websocket::{WebSocketReceiver, WebSocketSender},
};

const MAX_OUTBOUND_HISTORY_MESSAGE_SIZE: usize = 7 * 1024;

pub struct ChatSession {
    session_id: Uuid,
    chat_state: ChatState,
    user_details: UserDetails,
    character_resolver: CharacterResolver,
    // TODO: ConnectedAt, other?
}

impl ChatSession {
    pub fn new(
        chat_state: ChatState,
        user_details: UserDetails,
        character_resolver: CharacterResolver,
    ) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            chat_state,
            user_details,
            character_resolver,
        }
    }

    pub async fn run(
        &mut self,
        mut ws_sender: WebSocketSender,
        mut ws_receiver: WebSocketReceiver,
    ) -> Result<()> {
        let mut broadcast_rx = self.chat_state.outbound_tx.subscribe();
        let history: Vec<_> = self
            .chat_state
            .history
            .lock()
            .unwrap()
            .iter_rev()
            .take(30)
            .map(|m| (**m).clone())
            .collect();
        let history_message =
            build_history_connect_message(self.user_details.user.user_id, history);

        ws_sender
            .send(&history_message)
            .await
            .unwrap_or_else(|_| tracing::warn!("failed to send connection message"));

        // Maybe this should be handler outside of this:
        let (direct_tx, mut direct_rx) = mpsc::channel(32);
        self.chat_state
            .reply_map
            .insert(self.session_id, direct_tx.clone());
        self.chat_state
            .users_map
            .entry(self.user_details.user.user_id)
            .or_default()
            .insert(self.session_id);
        self.chat_state
            .usernames_map
            .entry(self.user_details.user.username.to_ascii_lowercase())
            .or_insert((
                self.user_details.user.user_id,
                self.user_details.user.username.clone(),
            ));
        ///////////////////////////////

        let write_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = broadcast_rx.recv() => {
                        match result {
                            Ok(msg) => {
                                if let Err(err) = ws_sender.send_raw(msg).await {
                                    tracing::warn!("failed to send message: {}", err);
                                    break;
                                }
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                                tracing::warn!("chat connection skipped {skipped} broadcast messages");
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                        }
                    }
                    Some(msg) = direct_rx.recv() => {
                        if let Err(err) = ws_sender.send(&msg).await {
                            tracing::warn!("failed to send message: {}", err);
                            break;
                        }
                    }
                    else => break, // This disconnect...
                }
            }
        });

        tokio::pin!(write_task);

        loop {
            tokio::select! {
                res = &mut write_task => {
                    if let Err(e) = res {
                        tracing::warn!("writer task failed: {}", e);
                    }
                    break;
                }
                m = ws_receiver.block_receive() => match m {
                    ControlFlow::Continue(m) => {
                        if let Some(error_message) = self.handle_client_message(m).await
                            && let Err(e) = direct_tx.send(error_message.into()).await
                        {
                            tracing::warn!("failed to send error to client: {}", e)
                        }
                    }
                    ControlFlow::Break(_) => break,
                }
            }
        }

        // direct_tx
        //     .send(
        //         ServerDisconnectMessage {
        //             reason: "session end".into(),
        //         }
        //         .into(),
        //     )
        //     .await
        //     .unwrap_or_else(|_| tracing::warn!("failed to send disconnection message"));

        write_task.abort();

        // Maybe this should be handled outside of this:
        let mut user_entry = self
            .chat_state
            .users_map
            .entry(self.user_details.user.user_id)
            .or_default();
        user_entry.remove(&self.session_id);
        if user_entry.is_empty() {
            self.chat_state
                .usernames_map
                .remove(&self.user_details.user.username.to_ascii_lowercase());
        }
        self.chat_state.reply_map.remove(&self.session_id);

        tracing::debug!("chat session '{}' ended ", self.user_details.user.user_id);
        Ok(())
    }

    async fn handle_client_message(&mut self, msg: ClientChatMessage) -> Option<ErrorMessage> {
        match msg {
            ClientChatMessage::Heartbeat => {}
            ClientChatMessage::Connect(_) => {
                tracing::warn!("received unexpected message: {:?}", msg);
                return Some(ErrorMessage {
                    error_type: ErrorType::Server,
                    message: "unexpected message received from client".to_string(),
                    must_disconnect: true,
                });
            }
            // ClientChatMessage::Disconnect(m) => {}
            ClientChatMessage::PostMessage(m) => {
                if let Err(error_message) = self.handle_chat_message(*m).await {
                    return Some(error_message);
                }
            }
        }
        None
    }

    async fn handle_chat_message(
        &mut self,
        msg: ClientPostMessage,
    ) -> std::result::Result<(), ErrorMessage> {
        // let (linked_item, item_signature) = msg.linked_item.unzip();
        let character_name = self
            .character_resolver
            .resolve(msg.character_id)
            .await
            .map_err(|err| ErrorMessage {
                error_type: ErrorType::Chat,
                message: err.to_string(),
                must_disconnect: false,
            })?;

        self.chat_state
            .inbound_tx
            .send((
                self.session_id,
                ChatMessage {
                    channel: msg.channel,
                    user_id: Some(self.user_details.user.user_id),
                    username: Some(self.user_details.user.username.clone()),
                    character_id: msg.character_id,
                    character_name,
                    chat_badge: self.user_details.chat_badge.clone(),
                    content: msg.content.into_inner(),
                    linked_item: msg.linked_item,
                    // item_signature,
                    sent_at: Utc::now(),
                },
            ))
            .await
            .map_err(|err| {
                tracing::error!("failed to queue chat message: {err}");
                ErrorMessage {
                    error_type: ErrorType::Server,
                    message: "failed to process chat message".to_string(),
                    must_disconnect: true,
                }
            })?;

        Ok(())
    }
}

fn build_history_connect_message(user_id: UserId, history: Vec<ChatMessage>) -> ServerChatMessage {
    let mut message = ServerConnectMessage {
        user_id,
        history: Vec::new(),
    };

    for history_message in history.into_iter() {
        message.history.push(history_message);

        if rmp_serde::to_vec(&message)
            .map(|bytes| bytes.len())
            .unwrap_or(usize::MAX)
            <= MAX_OUTBOUND_HISTORY_MESSAGE_SIZE
        {
            continue;
        }

        message.history.pop();
        break;
    }

    message.into()
}
