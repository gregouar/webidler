use anyhow::Result;
use serde::{Deserialize, Serialize};

use shared::data::world::WorldSpecs;

use crate::game::utils::json::LoadJsonFromFile;

use super::{loot_table::LootTable, master_store::LootTablesStore};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldBlueprintSchema {
    pub specs: WorldSpecs,
    pub waves: Vec<MonsterWaveBlueprint>,
    pub loot_tables: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldBlueprint {
    pub specs: WorldSpecs,
    pub waves: Vec<MonsterWaveBlueprint>,
    pub loot_table: LootTable,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MonsterWaveBlueprint {
    pub min_level: Option<u16>,
    pub max_level: Option<u16>,
    pub weight: f64,
    pub spawns: Vec<MonsterWaveSpawnBlueprint>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MonsterWaveSpawnBlueprint {
    pub monster: String,
    pub min_quantity: u8,
    pub max_quantity: u8,
}

impl LoadJsonFromFile for WorldBlueprintSchema {}

impl WorldBlueprint {
    pub fn populate_from_schema(
        schema: WorldBlueprintSchema,
        loot_tables_store: &LootTablesStore,
    ) -> Result<Self> {
        let loot_tables: Vec<_> = schema
            .loot_tables
            .iter()
            .map(|t| {
                loot_tables_store
                    .get(t)
                    .ok_or(anyhow::format_err!("missing loot table '{:?}'", t))
            })
            .collect::<Result<_>>()?;

        Ok(Self {
            loot_table: LootTable {
                entries: loot_tables
                    .into_iter()
                    .flat_map(|t| t.entries.clone())
                    .collect(),
            },
            specs: schema.specs,
            waves: schema.waves,
        })
    }
}
