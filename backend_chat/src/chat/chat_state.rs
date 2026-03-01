use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::{broadcast, mpsc};

use shared::messages::chat::{ChatMessage, ServerChatMessage};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ChatState {
    pub inbound_tx: mpsc::Sender<(Uuid, ChatMessage)>,
    pub outbound_tx: broadcast::Sender<Arc<ServerChatMessage>>,
    pub reply_map: DashMap<Uuid, mpsc::Sender<ServerChatMessage>>,
}
