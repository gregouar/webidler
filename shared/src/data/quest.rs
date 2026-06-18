use serde::{Deserialize, Serialize};

use crate::data::item::ItemSpecs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuestRewards {
    pub item_rewards: Vec<ItemSpecs>,
}
