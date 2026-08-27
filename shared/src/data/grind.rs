use serde::{Deserialize, Serialize};

use crate::data::{
    area::AreaLevel,
    item::{ItemCategory, ItemRarity, ItemSpecs},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrindRewards {
    pub item_rewards: Vec<ItemSpecs>,
    #[serde(default)]
    pub quest_reward: Option<ItemSpecs>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct QuestSpecs {
    pub description: String,
    pub area_level: AreaLevel,
    pub reward: QuestReward,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum QuestReward {
    Item {
        item_id: String,
        level: AreaLevel,
        #[serde(default)]
        rarity: Option<ItemRarity>,
        #[serde(default)]
        max_affixes: bool,
    },
    Loot {
        level: AreaLevel,
        #[serde(default)]
        loot_tables: Option<Vec<String>>,
        #[serde(default)]
        item_rarity: Option<ItemRarity>,
        #[serde(default)]
        item_category: Option<ItemCategory>,
        #[serde(default)]
        max_base: bool,
        #[serde(default)]
        max_affixes: bool,
    },
}
