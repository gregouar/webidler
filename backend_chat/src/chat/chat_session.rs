use std::ops::ControlFlow;

use anyhow::Result;

use shared::{
    data::user::UserId,
    messages::{
        chat::{ClientChatMessage, ServerDisconnectMessage},
        server::{ErrorMessage, ErrorType},
    },
};
use tokio::task::yield_now;

use crate::websocket::WebSocketConnection;

pub struct ChatSession<'a> {
    client_conn: &'a mut WebSocketConnection,
    user_id: UserId,
    // TODO: ConnectedAt, other?
}

impl<'a> ChatSession<'a> {
    pub fn new(client_conn: &'a mut WebSocketConnection, user_id: UserId) -> Self {
        Self {
            client_conn,
            user_id,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            if self.handle_client_inputs().await.is_break() {
                break;
            }
        }

        self.client_conn
            .send(
                &ServerDisconnectMessage {
                    reason: "session end".into(),
                }
                .into(),
            )
            .await
            .unwrap_or_else(|_| tracing::warn!("failed to send disconnection message"));

        tracing::debug!("chat session '{}' ended ", self.user_id);
        Ok(())
    }

    // TODO:  rework this as blocking, polling doesn't make sense for chat
    async fn handle_client_inputs(&mut self) -> ControlFlow<(), ()> {
        // We limit the amount of events we handle in one loop
        for _ in 1..10 {
            match self.client_conn.poll_receive() {
                ControlFlow::Continue(Some(m)) => {
                    if let Some(error_message) = self.handle_client_message(m)
                        && let Err(e) = self.client_conn.send(&error_message.into()).await
                    {
                        tracing::warn!("failed to send error to client: {}", e)
                    }
                }
                ControlFlow::Continue(None) => return ControlFlow::Continue(()), // No more messages
                ControlFlow::Break(_) => return ControlFlow::Break(()), // Connection closed
            }
            yield_now().await;
        }
        ControlFlow::Continue(())
    }

    fn handle_client_message(&self, msg: ClientChatMessage) -> Option<ErrorMessage> {
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
            ClientChatMessage::Disconnect(m) => {}
            ClientChatMessage::PostMessage(client_post_message) => todo!(),
        }
        None
    }
}
