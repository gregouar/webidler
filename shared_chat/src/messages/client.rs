use serde::{Deserialize, Serialize};

use crate::types::{ChatChannel, ChatContent, LinkedItemBytes};

use super::macros::impl_into_message;

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
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct ClientDisconnectMessage {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientPostMessage {
    pub channel: ChatChannel,
    pub content: ChatContent,
    pub linked_item: Option<LinkedItemBytes>,
    // pub linked_item: Option<(LinkedItemBytes, [u8; 32])>,
}
