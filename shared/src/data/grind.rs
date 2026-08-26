use serde::{Deserialize, Serialize};

use crate::data::item::ItemSpecs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrindRewards {
    pub item_rewards: Vec<ItemSpecs>,
}
