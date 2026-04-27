use chrono::{DateTime, Utc};
use nutype::nutype;
use serde::{Deserialize, Serialize};

pub type UserId = uuid::Uuid;
const MAX_LINKED_ITEM_SIZE: usize = 4096;

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
    pub sent_at: DateTime<Utc>,

    pub user_id: Option<UserId>,
    pub username: Option<String>,
    pub chat_badge: Option<String>,

    pub content: String,
    pub linked_item: Option<LinkedItemBytes>,
    // #[serde(default, skip_serializing, skip_deserializing)]
    // pub item_signature: Option<[u8; 32]>,
}

#[nutype(
    sanitize(with=strip_control_chars),
    validate(len_char_max = 200),
    default="",
    derive(Deserialize, Serialize, Debug, PartialEq, Clone, Deref, Default, Display)
)]
pub struct ChatContent(String);

pub fn strip_control_chars(input: String) -> String {
    input
        .chars()
        .filter(|c| !c.is_control() || *c == '\n')
        .collect()
}

#[nutype(
    validate(
        predicate = |bytes: &Vec<u8>| bytes.len() <= MAX_LINKED_ITEM_SIZE
    ),
    derive(Serialize, Deserialize, Debug, Clone, AsRef)
)]
pub struct LinkedItemBytes(Vec<u8>);
