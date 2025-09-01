use serde::{Deserialize, Serialize};

use crate::data::item::ItemSpecs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketItem {
    pub index: usize,
    pub item_specs: ItemSpecs,
    pub price: f64,
}
