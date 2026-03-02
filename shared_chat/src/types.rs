use chrono::{DateTime, Utc};
use nutype::nutype;
use serde::{Deserialize, Serialize};

pub type UserId = uuid::Uuid;

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

#[nutype(
    validate(not_empty, len_char_max = 200),
    derive(Deserialize, Serialize, Debug, PartialEq, Clone, Deref)
)]
pub struct ChatContent(String);
