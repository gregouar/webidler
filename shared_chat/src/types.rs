use chrono::{DateTime, Utc};
use nutype::nutype;
use serde::{Deserialize, Serialize};

pub type UserId = uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub enum ChatChannel {
    System,
    Global,
    Trade,
    Whisper(UserId),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub channel: ChatChannel,
    pub user_id: Option<UserId>,
    pub username: Option<String>,
    pub content: ChatContent,
    pub sent_at: DateTime<Utc>,
}

#[nutype(
    sanitize(with=strip_control_chars),
    validate(not_empty, len_char_max = 200),
    derive(Deserialize, Serialize, Debug, PartialEq, Clone, Deref)
)]
pub struct ChatContent(String);

pub fn strip_control_chars(input: String) -> String {
    input
        .chars()
        .filter(|c| !c.is_control() || *c == '\n')
        .collect()
}
