use serde::{Deserialize, Serialize};

use super::macros::impl_into_message;

impl_into_message! {
    #[derive(Serialize, Deserialize, Debug, Clone,)]
    pub enum ClientMessage {
        Heartbeat,
        Connect(ClientConnectMessage),
        Test(TestMessage),
    }
}

// Use default to generate heartbeats
impl Default for ClientMessage {
    fn default() -> Self {
        ClientMessage::Heartbeat
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientConnectMessage {
    pub bearer: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestMessage {
    pub greeting: String,
    pub value: i32,
}
