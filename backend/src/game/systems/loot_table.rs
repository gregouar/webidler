use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use shared::data::world::AreaLevel;

use crate::game::utils::json::load_json;

use super::items_table::ItemId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LootTable {
    pub entries: Vec<LootTableEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LootTableEntry {
    pub item_id: ItemId,

    pub weight: f32,
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

impl LootTable {
    pub async fn load_from_file(filepath: &PathBuf) -> Result<Self> {
        Ok(load_json(filepath).await?)
    }
}
