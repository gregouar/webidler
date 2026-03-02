use serde::{Deserialize, Serialize};

use crate::types::{ChatMessage, UserId};

use super::macros::impl_into_message;

impl_into_message! {
    #[derive(Serialize, Deserialize, Debug, Clone,)]
    pub enum ServerChatMessage {
        Connect(ServerConnectMessage),
        // Disconnect(ServerDisconnectMessage),
        Error(ErrorMessage),

        Broadcast(ChatMessage),
        WhisperFeedback(ServerWhisperFeedbackMessage),
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerWhisperFeedbackMessage {
    pub target_username: Option<String>,
    pub target_user_id: UserId,
    pub chat_message: ChatMessage,
}
