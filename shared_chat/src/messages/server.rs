use serde::{Deserialize, Serialize};

use crate::types::ChatMessage;

use super::macros::impl_into_message;

impl_into_message! {
    #[derive(Serialize, Deserialize, Debug, Clone,)]
    pub enum ServerChatMessage {
        Connect(ServerConnectMessage),
        // Disconnect(ServerDisconnectMessage),
        Error(ErrorMessage),

        Broadcast(ChatMessage),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorMessage {
    pub error_type: ErrorType,
    pub message: String,
    pub must_disconnect: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ErrorType {
    Server,
    Chat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConnectMessage {
    pub history: Vec<ChatMessage>,
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct ServerDisconnectMessage {
//     pub reason: String,
// }
