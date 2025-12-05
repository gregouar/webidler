use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::data::{item::ItemSpecs, user::UserCharacterId};

pub type StashId = uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StashItem {
    pub stash_id: StashId,
    pub item_id: usize,

    pub character_id: UserCharacterId,
    pub character_name: String,

    pub item_specs: ItemSpecs,

    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum StashType {
    User,
    Market,
}
