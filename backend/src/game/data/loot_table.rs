use serde::{Deserialize, Serialize};

use shared::data::area::AreaLevel;

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

    #[serde(default)]
    pub boss_only: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RarityWeights {
    pub normal: usize,
    pub magic: usize,
    pub rare: usize,
    pub unique: usize,
}

impl Default for RarityWeights {
    fn default() -> Self {
        RarityWeights {
            normal: 100,
            magic: 80,
            rare: 20,
            unique: 1,
        }
    }
}

impl LoadJsonFromFile for LootTable {}
