use anyhow::Result;

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
use serde::Serialize;
use std::ops::ControlFlow;
use std::{net::SocketAddr, time::Duration};
use tokio::task::JoinHandle;

use shared::messages::{client::ClientMessage, compression, server::ServerMessage};

type WsSender = SplitSink<WebSocket, Message>;
type SendTask = JoinHandle<Result<WsSender>>;

pub struct WebSocketConnection {
    receiver_rx: mpsc::Receiver<ClientMessage>,
    ws_sender: Option<WsSender>,
    send_task: Option<SendTask>,
    send_buffer: Vec<u8>,
    compression_dictionary: Option<Vec<u8>>,
    compression_enabled: bool,
}

impl WebSocketConnection {
    /// Establish a connection on the socket, starting a listening job in background
    pub fn establish(
        socket: WebSocket,
        addr: SocketAddr,
        timeout: Duration,
        compression_dictionary: Option<Vec<u8>>,
    ) -> Self {
        let (ws_sender, mut ws_receiver) = socket.split();
        let (receiver_tx, receiver_rx) = mpsc::channel(10);
        let receiver_compression_dictionary = compression_dictionary.clone();

        // Start receiving task in background, posting new client messages on channel
        tokio::spawn(async move {
            loop {
                match time::timeout(timeout, ws_receiver.next()).await {
                    Ok(Some(Ok(m))) => {
                        match process_message(m, addr, receiver_compression_dictionary.as_deref()) {
                            ControlFlow::Continue(Some(m)) => {
                                if receiver_tx.send(m).await.is_err() {
                                    // If channel is closed, we can stop
                                    break;
                                }
                            }
                            ControlFlow::Break(_) => break,
                            _ => {}
                        }
                    }
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

        WebSocketConnection {
            receiver_rx,
            ws_sender: Some(ws_sender),
            send_task: None,
            send_buffer: Vec::with_capacity(16 * 1024),
            compression_dictionary,
            compression_enabled: false,
        }
    }

    pub async fn send(&mut self, msg: &ServerMessage) -> Result<()> {
        self.wait_for_pending_send().await?;
        self.start_background_send(msg)?;
        self.wait_for_pending_send().await?;

        Ok(())
    }

    pub fn start_background_send(&mut self, msg: &ServerMessage) -> Result<()> {
        let sender = self
            .ws_sender
            .take()
            .ok_or_else(|| anyhow::format_err!("websocket sender is not ready"))?;
        self.send_task = Some(tokio::spawn(send_task(
            sender,
            into_ws_msg(
                msg,
                &mut self.send_buffer,
                self.compression_enabled,
                self.compression_dictionary.as_deref(),
            )?,
        )));

        Ok(())
    }

    pub fn enable_compression(&mut self) {
        self.compression_enabled = true;
    }

    async fn wait_for_pending_send(&mut self) -> Result<()> {
        if let Some(send_task) = self.send_task.take() {
            self.ws_sender = Some(send_task.await??);
        }

        Ok(())
    }

    pub async fn poll_pending_send(&mut self) -> Result<bool> {
        if self
            .send_task
            .as_ref()
            .is_some_and(|task| !task.is_finished())
        {
            return Ok(false);
        }

        self.wait_for_pending_send().await?;

        Ok(true)
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

    pub async fn block_receive(&mut self) -> ControlFlow<(), ClientMessage> {
        match self.receiver_rx.recv().await {
            Some(m) => ControlFlow::Continue(m),
            None => ControlFlow::Break(()),
        }
    }
}

fn process_message(
    msg: Message,
    who: SocketAddr,
    compression_dictionary: Option<&[u8]>,
) -> ControlFlow<(), Option<ClientMessage>> {
    match msg {
        Message::Binary(b) => match from_ws_msg(&b, compression_dictionary) {
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

fn into_ws_msg(
    message: &ServerMessage,
    send_buffer: &mut Vec<u8>,
    compression_enabled: bool,
    compression_dictionary: Option<&[u8]>,
) -> Result<Message> {
    send_buffer.clear();
    message.serialize(&mut rmp_serde::Serializer::new(&mut *send_buffer))?;

    let bytes = if compression_enabled {
        compression::encode_payload_from_slice(send_buffer, compression_dictionary)?
            .expect("compression should always produce an encoded payload")
    } else {
        std::mem::take(send_buffer)
    };

    Ok(Message::Binary(Bytes::from_owner(bytes)))
}

async fn send_task(mut sender: WsSender, ws_msg: Message) -> Result<WsSender> {
    timeout(Duration::from_secs(2), sender.send(ws_msg)).await??;
    Ok(sender)
}

fn from_ws_msg(message: &Bytes, compression_dictionary: Option<&[u8]>) -> Result<ClientMessage> {
    Ok(rmp_serde::from_slice(
        &compression::decode_payload_with_dictionary(message, compression_dictionary)?,
    )?)
}
