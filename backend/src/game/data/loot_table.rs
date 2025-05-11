use serde::{Deserialize, Serialize};

use shared::data::world::AreaLevel;

use crate::game::utils::json::LoadJsonFromFile;

use super::items_store::ItemId;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LootTable {
    pub entries: Vec<LootTableEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LootTableEntry {
    pub item_id: ItemId,

    pub weight: u64,
    pub min_area_level: Option<AreaLevel>,
    pub max_area_level: Option<AreaLevel>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RarityWeights {
    pub normal: usize,
    pub magic: usize,
    pub rare: usize,
}

impl Default for RarityWeights {
    fn default() -> Self {
        RarityWeights {
            normal: 6,
            magic: 3,
            rare: 1,
        }
    }
}

impl LoadJsonFromFile for LootTable {}
