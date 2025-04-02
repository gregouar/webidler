use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Connect(ServerConnectMessage),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConnectMessage {
    pub greeting: String,
    pub value: i32,
}
