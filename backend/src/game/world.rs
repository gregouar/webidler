use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::u16;

use serde::{Deserialize, Serialize};

use futures::future::join_all;

use super::data::load_json;
use shared::data::{monster::MonsterSpecs, world::WorldSpecs};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WorldBlueprint {
    pub schema: WorldBlueprintSchema,
    pub monster_specs: HashMap<PathBuf, MonsterSpecs>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WorldBlueprintSchema {
    pub specs: WorldSpecs,
    pub waves: Vec<MonsterWaveBlueprint>,
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
    pub async fn load_from_file(filepath: PathBuf) -> Result<Self> {
        let schema: WorldBlueprintSchema = load_json(&filepath).await?;

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

        Ok(WorldBlueprint {
            schema,
            monster_specs,
        })
    }
}
// TODO: Global MonsterPrototypesPool shared among instances ?
