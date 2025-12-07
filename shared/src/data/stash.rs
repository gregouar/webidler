use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::data::{
    item::ItemSpecs,
    user::{UserCharacterId, UserId},
};

pub type StashId = uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub enum StashType {
    #[default]
    User,
    Market,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Stash {
    pub stash_id: StashId,
    pub user_id: UserId,

    pub stash_type: StashType,
    pub title: Option<String>,

    pub items_amount: usize,
    pub max_items: usize,
    pub resource_gems: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StashItem {
    pub stash_id: StashId,
    pub stash_item_id: usize,

    pub user_id: UserId,
    pub character_id: Option<UserCharacterId>,
    pub character_name: Option<String>,

    pub item_specs: ItemSpecs,

    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StashPrice {
    pub start_price: f64,
    pub start_size: usize,

    pub upgrade_price: f64,
    pub upgrade_size: usize,
}
