use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::data::{item::ItemSpecs, user::UserCharacterId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketItem {
    pub item_id: usize,

    pub seller: UserCharacterId,
    pub private_sale: Option<UserCharacterId>,

    pub price: f64,

    pub item_specs: ItemSpecs,

    pub created_at: DateTime<Utc>,
}
