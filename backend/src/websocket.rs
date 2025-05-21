use anyhow::Result;

use tokio::sync::mpsc;
use tokio::time;

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

use shared::messages::{client::ClientMessage, server::ServerMessage};

pub struct WebSocketConnection {
    receiver_rx: mpsc::Receiver<ClientMessage>,
    ws_sender: SplitSink<WebSocket, Message>,
}

impl WebSocketConnection {
    /// Establish a connection on the socket, starting a listening job in background
    pub fn establish(socket: WebSocket, who: SocketAddr, timeout: Duration) -> Self {
        let (ws_sender, mut ws_receiver) = socket.split();
        let (receiver_tx, receiver_rx) = mpsc::channel(10);

        // Start receiving task in background, posting new client messages on channel
        tokio::spawn(async move {
            loop {
                match time::timeout(timeout, ws_receiver.next()).await {
                    Ok(Some(Ok(m))) => match process_message(m, who) {
                        ControlFlow::Continue(Some(m)) => {
                            if receiver_tx.send(m).await.is_err() {
                                // If channel is closed, we can stop
                                break;
                            }
                        }
                        ControlFlow::Break(_) => break,
                        _ => {}
                    },
                    Err(_) => {
                        tracing::warn!("client disconnected due to inactivity");
                        break;
                    }
                    _ => {}
                }
            }
        });

        WebSocketConnection {
            receiver_rx,
            ws_sender,
        }
    }

    pub async fn send(&mut self, msg: &ServerMessage) -> Result<()> {
        self.ws_sender.send(into_ws_msg(msg)?).await?;
        Ok(())
    }

    /// Poll new messages. Return
    /// - ControlFlow::Continue(Some(m)) if new message m received, and remove m from the queue
    /// - ControlFlow::Continue(None) if no message available
    /// - ControlFlow::Break on disconnection
    pub fn poll_receive(&mut self) -> ControlFlow<(), Option<ClientMessage>> {
        match self.receiver_rx.try_recv() {
            Ok(m) => ControlFlow::Continue(Some(m)),
            Err(mpsc::error::TryRecvError::Empty) => ControlFlow::Continue(None),
            Err(mpsc::error::TryRecvError::Disconnected) => ControlFlow::Break(()),
        }
    }
}

fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), Option<ClientMessage>> {
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

fn into_ws_msg(message: &ServerMessage) -> Result<Message> {
    Ok(Message::Binary(Bytes::from_owner(rmp_serde::to_vec(
        message,
    )?)))
}

fn from_ws_msg(message: &Bytes) -> Result<ClientMessage> {
    Ok(rmp_serde::from_slice(message)?)
}
