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
    pub normal: f64,
    pub magic: f64,
    pub rare: f64,
    pub unique: f64,
}

impl Default for RarityWeights {
    fn default() -> Self {
        RarityWeights {
            normal: 400.0,
            magic: 320.0,
            rare: 80.0,
            unique: 1.0,
        }
    }
}

impl LoadJsonFromFile for LootTable {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GambleTableBlueprint {
    pub loot_tables: Vec<String>,
    pub item_rarity: f64,
}

#[derive(Debug, Clone)]
pub struct GambleTable {
    pub loot_table: LootTable,
    pub item_rarity: f64,
}

impl LoadJsonFromFile for GambleTableBlueprint {}
