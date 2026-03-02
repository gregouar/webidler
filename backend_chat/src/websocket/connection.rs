use anyhow::Result;

use shared_chat::messages::server::ServerChatMessage;
use tokio::time;
use tokio::{sync::mpsc, time::timeout};

use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocket},
};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, StreamExt},
};
use std::ops::ControlFlow;
use std::{net::SocketAddr, time::Duration};

use shared_chat::messages::client::ClientChatMessage;

pub struct WebSocketSender {
    ws_sender: SplitSink<WebSocket, Message>,
}

pub struct WebSocketReceiver {
    receiver_rx: mpsc::Receiver<ClientChatMessage>,
}

/// Establish a connection on the socket, starting a listening job in background
pub fn establish(
    socket: WebSocket,
    addr: SocketAddr,
    timeout: Duration,
) -> (WebSocketSender, WebSocketReceiver) {
    let (ws_sender, mut ws_receiver) = socket.split();
    let (receiver_tx, receiver_rx) = mpsc::channel(10);

    // Start receiving task in background, posting new client messages on channel
    tokio::spawn(async move {
        loop {
            match time::timeout(timeout, ws_receiver.next()).await {
                Ok(Some(Ok(m))) => match process_message(m, addr) {
                    ControlFlow::Continue(Some(m)) => {
                        if receiver_tx.send(m).await.is_err() {
                            // If channel is closed, we can stop
                            break;
                        }
                    }
                    ControlFlow::Break(_) => break,
                    _ => {}
                },
                Ok(Some(Err(e))) => {
                    tracing::error!("connection error: {}", e);
                    break;
                }
                Ok(None) => break, // Connection dropped
                Err(_) => {
                    tracing::warn!("client disconnected due to inactivity");
                    break;
                }
            }
        }
    });

    (
        WebSocketSender { ws_sender },
        WebSocketReceiver { receiver_rx },
    )
}

impl WebSocketSender {
    pub async fn send(&mut self, msg: &ServerChatMessage) -> Result<()> {
        timeout(
            Duration::from_secs(5),
            self.ws_sender.send(into_ws_msg(msg)?),
        )
        .await??;
        Ok(())
    }
}

impl WebSocketReceiver {
    pub async fn block_receive(&mut self) -> ControlFlow<(), ClientChatMessage> {
        match self.receiver_rx.recv().await {
            Some(m) => ControlFlow::Continue(m),
            None => ControlFlow::Break(()),
        }
    }
}

fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), Option<ClientChatMessage>> {
    match msg {
        Message::Binary(b) => match from_ws_msg(&b) {
            Ok(m) => ControlFlow::Continue(Some(m)),
            Err(_) => {
                tracing::warn!(">>> {} sent invalid message, closing", who);
                ControlFlow::Break(())
            }
        },
        Message::Text(_) => {
            tracing::warn!(">>> {} sent str instead of bytes, closing", who);
            ControlFlow::Break(())
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                tracing::debug!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who,
                    cf.code,
                    cf.reason
                );
            } else {
                tracing::debug!(">>> {} somehow sent close message without CloseFrame", who);
            }
            tracing::info!("client disconnected");
            ControlFlow::Break(())
        }
        _ => ControlFlow::Continue(None), // Ignore ping pong
    }
}

fn into_ws_msg(message: &ServerChatMessage) -> Result<Message> {
    Ok(Message::Binary(Bytes::from_owner(rmp_serde::to_vec(
        message,
    )?)))
}

fn from_ws_msg(message: &Bytes) -> Result<ClientChatMessage> {
    Ok(rmp_serde::from_slice(message)?)
}
