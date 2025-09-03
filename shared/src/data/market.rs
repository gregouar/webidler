use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::data::{
    item::{ItemRarity, ItemSpecs},
    user::UserCharacterId,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketItem {
    pub item_id: usize,

    pub owner_id: UserCharacterId,
    pub owner_name: String,

    pub recipient: Option<(UserCharacterId, String)>,
    pub rejected: bool,

    pub price: f64,

    pub item_specs: ItemSpecs,

    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MarketFilters {
    pub item_rarity: Option<ItemRarity>,
}
