use anyhow::Result;
use backend_shared::http::users::User;
use chrono::Utc;
use std::ops::ControlFlow;
use tokio::sync::mpsc;
use uuid::Uuid;

use shared_chat::{
    messages::{
        client::{ClientChatMessage, ClientPostMessage},
        server::{ErrorMessage, ErrorType, ServerConnectMessage},
    },
    types::ChatMessage,
};

use crate::{
    chat::chat_state::ChatState,
    websocket::{WebSocketReceiver, WebSocketSender},
};

pub struct ChatSession {
    session_id: Uuid,
    chat_state: ChatState,
    user: User,
    // TODO: ConnectedAt, other?
}

impl ChatSession {
    pub fn new(chat_state: ChatState, user: User) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            chat_state,
            user,
        }
    }

    pub async fn run(
        &self,
        mut ws_sender: WebSocketSender,
        mut ws_receiver: WebSocketReceiver,
    ) -> Result<()> {
        let history = ServerConnectMessage {
            history: self
                .chat_state
                .history
                .lock()
                .unwrap()
                .iter_rev()
                .take(20)
                .map(|m| (**m).clone())
                .collect(),
        }
        .into();

        ws_sender
            .send(&history)
            .await
            .unwrap_or_else(|_| tracing::warn!("failed to send connection message"));

        // Maybe this should be handler outside of this:
        let (direct_tx, mut direct_rx) = mpsc::channel(32);
        self.chat_state
            .reply_map
            .insert(self.session_id, direct_tx.clone());
        let mut broadcast_rx = self.chat_state.outbound_tx.subscribe();
        ///////////////////////////////

        let write_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(msg) = broadcast_rx.recv() => {
                       if let Err(err)=  ws_sender.send_raw(msg).await {
                         tracing::warn!("failed to send message: {}",err);
                         break;
                       }

                    }
                    Some(msg) = direct_rx.recv() => {
                       if let Err(err)=  ws_sender.send(&msg).await {
                         tracing::warn!("failed to send message: {}",err);
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

        // Maybe this should be handler outside of this:
        self.chat_state.reply_map.remove(&self.session_id);

        tracing::debug!("chat session '{}' ended ", self.user.user_id);
        Ok(())
    }

    async fn handle_client_message(&self, msg: ClientChatMessage) -> Option<ErrorMessage> {
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
                if let Err(err) = self.handle_chat_message(*m).await {
                    return Some(ErrorMessage {
                        error_type: ErrorType::Chat,
                        message: err.to_string(),
                        must_disconnect: true,
                    });
                }
            }
        }
        None
    }

    async fn handle_chat_message(&self, msg: ClientPostMessage) -> Result<()> {
        self.chat_state
            .inbound_tx
            .send((
                self.session_id,
                ChatMessage {
                    channel: msg.channel,
                    user_id: Some(self.user.user_id),
                    user_name: Some(self.user.username.clone()),
                    content: msg.content,
                    sent_at: Utc::now(),
                },
            ))
            .await?;

        Ok(())
    }
}
