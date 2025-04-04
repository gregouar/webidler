use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    Heartbeat,
    Connect(ClientConnectMessage),
    Test(TestMessage),
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

impl Default for ClientMessage {
    fn default() -> Self {
        ClientMessage::Heartbeat
    }
}
