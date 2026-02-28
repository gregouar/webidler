use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{data::user::UserId, messages::server::ErrorMessage, types::ChatContent};

use super::macros::impl_into_message;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub enum ChatChannel {
    System,
    Global,
    Trade,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub channel: ChatChannel,
    pub user_id: Option<UserId>,
    pub user_name: Option<String>,
    pub content: ChatContent,
    pub sent_at: DateTime<Utc>,
}

impl_into_message! {
    #[derive(Serialize, Deserialize, Debug, Clone,)]
    pub enum ClientChatMessage {
        Heartbeat,

        Connect(ClientConnectMessage),
        // Disconnect(ClientDisconnectMessage),
        PostMessage(ClientPostMessage),
    }
}

// Use default to generate heartbeats
#[allow(clippy::derivable_impls)]
impl Default for ClientChatMessage {
    fn default() -> Self {
        ClientChatMessage::Heartbeat
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientConnectMessage {
    pub jwt: String,
    pub user_id: UserId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientDisconnectMessage {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientPostMessage {
    pub channel: ChatChannel,
    pub content: ChatContent,
}

impl_into_message! {
    #[derive(Serialize, Deserialize, Debug, Clone,)]
    pub enum ServerChatMessage {
        Connect(ServerConnectMessage),
        Disconnect(ServerDisconnectMessage),
        Error(ErrorMessage),

        Broadcast(ServerBroadcastMessage),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConnectMessage {
    // TODO:
    // pub history: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerDisconnectMessage {
    pub reason: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerBroadcastMessage {
    pub chat_message: Arc<ChatMessage>,
}
