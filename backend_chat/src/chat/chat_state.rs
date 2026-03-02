use axum::body::Bytes;
use dashmap::DashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

use shared_chat::{
    messages::server::ServerChatMessage, ring_buffer::RingBuffer, types::ChatMessage,
};

#[derive(Debug, Clone)]
pub struct ChatState {
    pub inbound_tx: mpsc::Sender<(Uuid, ChatMessage)>,
    pub outbound_tx: broadcast::Sender<Arc<Bytes>>,
    pub reply_map: DashMap<Uuid, mpsc::Sender<ServerChatMessage>>,

    pub history: Arc<Mutex<RingBuffer<Arc<ChatMessage>>>>,
}
