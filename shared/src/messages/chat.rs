use serde::{Deserialize, Serialize};

use crate::{data::user::UserId, messages::server::ErrorMessage, types::Username};

use super::macros::impl_into_message;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub enum ChatChannel {
    System,
    Global,
    Trade,
}

impl_into_message! {
    #[derive(Serialize, Deserialize, Debug, Clone,)]
    pub enum ClientChatMessage {
        Heartbeat,

        Connect(ClientConnectMessage),
        Disconnect(ClientDisconnectMessage),
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
    pub content: String,
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
pub struct ServerConnectMessage {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerDisconnectMessage {
    pub reason: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerBroadcastMessage {
    pub user_id: UserId,
    pub user_name: Username,

    pub channel: ChatChannel,
    pub content: String,
}
