use axum::body::Bytes;
use backend_shared::http::users::UserId;
use dashmap::DashMap;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

use shared_chat::{
    messages::server::ServerChatMessage, ring_buffer::RingBuffer, types::ChatMessage,
};

#[derive(Debug, Clone)]
pub struct ChatState {
    pub inbound_tx: mpsc::Sender<(Uuid, ChatMessage)>,
    pub outbound_tx: broadcast::Sender<Arc<Bytes>>,

    pub reply_map: Arc<DashMap<Uuid, mpsc::Sender<ServerChatMessage>>>,
    pub users_map: Arc<DashMap<UserId, HashSet<Uuid>>>,
    pub usernames_map: Arc<DashMap<String, (UserId, String)>>,

    pub history: Arc<Mutex<RingBuffer<Arc<ChatMessage>>>>,
}
