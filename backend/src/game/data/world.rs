use anyhow::Result;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use shared::data::{monster::MonsterSpecs, world::WorldSpecs};

use crate::game::utils::json::load_json;

use super::loot_table::LootTable;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldBlueprint {
    pub schema: WorldBlueprintSchema,
    pub monster_specs: HashMap<PathBuf, MonsterSpecs>,
    pub loot_table: LootTable,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WorldBlueprintSchema {
    pub specs: WorldSpecs,
    pub waves: Vec<MonsterWaveBlueprint>,
    pub loot_tables: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterWaveBlueprint {
    pub min_level: Option<u16>,
    pub max_level: Option<u16>,
    pub probability: f64,
    pub spawns: Vec<MonsterWaveSpawnBlueprint>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterWaveSpawnBlueprint {
    pub path: PathBuf,
    pub min_quantity: u8,
    pub max_quantity: u8,
}

impl WorldBlueprint {
    pub async fn load_from_file(filepath: impl Into<&PathBuf>) -> Result<Self> {
        let schema: WorldBlueprintSchema = load_json(filepath).await?;

        let monster_specs_to_load: HashSet<PathBuf> = schema
            .waves
            .iter()
            .flat_map(|wave| wave.spawns.iter().map(|spawn| spawn.path.clone()))
            .collect();

        let monster_specs = join_all(monster_specs_to_load.into_iter().map(|path| async move {
            let monster_specs = load_json(&path).await?;
            Result::<(PathBuf, MonsterSpecs)>::Ok((path, monster_specs))
        }))
        .await
        .into_iter()
        .collect::<Result<_>>()?;

        let loot_tables: Vec<_> = join_all(schema.loot_tables.iter().map(|filepath| async move {
            Result::<_>::Ok(LootTable::load_from_file(filepath).await?)
        }))
        .await
        .into_iter()
        .collect::<Result<_>>()?;

        let loot_table = LootTable {
            entries: loot_tables
                .into_iter()
                .map(|x| x.entries)
                .flatten()
                .collect(),
        };

        Ok(WorldBlueprint {
            schema,
            monster_specs,
            loot_table,
        })
    }
}
// TODO: Global MonsterPrototypesPool shared among instances ?
// Also need some kind of DataStore that would zip all data together.
