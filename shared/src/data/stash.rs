use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::data::{item::ItemSpecs, user::UserCharacterId};

pub type StashId = uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StashItem {
    pub stash_id: StashId,
    pub stash_item_id: usize,

    pub character_id: Option<UserCharacterId>,
    pub character_name: Option<String>,

    pub item_specs: ItemSpecs,

    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum StashType {
    User,
    Market,
}
