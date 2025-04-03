use anyhow::Result;

use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocket},
};
use std::net::SocketAddr;
use std::ops::ControlFlow;

use shared::{client_messages::ClientMessage, server_messages::ServerMessage};

pub struct WebSocketConnection {
    socket: WebSocket,
    who: SocketAddr,
}

impl WebSocketConnection {
    pub fn new(socket: WebSocket, who: SocketAddr) -> Self {
        WebSocketConnection { socket, who }
    }

    pub async fn send(&mut self, msg: &ServerMessage) -> Result<()> {
        self.socket.send(into_ws_msg(&msg)?).await?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<ControlFlow<(), Option<ClientMessage>>> {
        match self.socket.recv().await {
            Some(m) => Ok(self.process_message(m?)),
            None => Ok(ControlFlow::Break(())),
        }
    }

    fn process_message(&self, msg: Message) -> ControlFlow<(), Option<ClientMessage>> {
        return match msg {
            Message::Binary(b) => match from_ws_msg(&b) {
                Ok(m) => ControlFlow::Continue(Some(m)),
                Err(_) => {
                    log::info!(">>> {} sent invalid message, closing", self.who);
                    ControlFlow::Break(())
                }
            },
            Message::Text(_) => {
                log::info!(">>> {} sent str instead of bytes, closing", self.who);
                ControlFlow::Break(())
            }
            Message::Close(c) => {
                if let Some(cf) = c {
                    log::info!(
                        ">>> {} sent close with code {} and reason `{}`",
                        self.who,
                        cf.code,
                        cf.reason
                    );
                } else {
                    log::info!(
                        ">>> {} somehow sent close message without CloseFrame",
                        self.who
                    );
                }
                ControlFlow::Break(())
            }
            _ => ControlFlow::Continue(None), // Ignore ping pong
        };
    }
}

fn into_ws_msg(message: &ServerMessage) -> Result<Message> {
    Ok(Message::Binary(Bytes::from_owner(rmp_serde::to_vec(
        message,
    )?)))
}

fn from_ws_msg(message: &Bytes) -> Result<ClientMessage> {
    Ok(rmp_serde::from_slice(message)?)
}
