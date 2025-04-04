use serde::{Deserialize, Serialize};

use super::macros::impl_into_message;

impl_into_message! {
    #[derive(Serialize, Deserialize, Debug, Clone,)]
    pub enum ServerMessage {
        Connect(ConnectMessage),
        Update(UpdateMessage),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectMessage {
    pub greeting: String,
    pub value: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateMessage {
    pub value: i32,
}
