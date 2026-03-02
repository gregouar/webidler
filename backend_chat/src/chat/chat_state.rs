use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

use shared_chat::{messages::server::ServerChatMessage, types::ChatMessage};

#[derive(Debug, Clone)]
pub struct ChatState {
    pub inbound_tx: mpsc::Sender<(Uuid, ChatMessage)>,
    pub outbound_tx: broadcast::Sender<Arc<ServerChatMessage>>,
    pub reply_map: DashMap<Uuid, mpsc::Sender<ServerChatMessage>>,
}
