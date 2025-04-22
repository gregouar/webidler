use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json;

use futures::future::join_all;
use tokio::fs;

use shared::data::{MonsterSpecs, WorldSpecs};

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
    min_level: u16,
    max_level: u16,
    probability: u32,
    monsters: Vec<MonsterWaveSpawnBlueprint>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterWaveSpawnBlueprint {
    pub monster_bp_path: PathBuf,
    pub min_quantity: u8,
    pub max_quantity: u8,
}

impl WorldBlueprint {
    pub async fn load_from_file(filepath: PathBuf) -> Result<Self> {
        let schema: WorldBlueprintSchema = serde_json::from_slice(&fs::read(&filepath).await?)?;

        let monster_specs_to_load: HashSet<PathBuf> = schema
            .waves
            .iter()
            .flat_map(|wave| {
                wave.monsters
                    .iter()
                    .map(|spawn| spawn.monster_bp_path.clone())
            })
            .collect();

        // let mut monster_specs_loading_tasks = HashMap::new();
        // for monster_specs_path in monster_specs_to_load.into_iter() {
        //     monster_specs_loading_tasks.insert(
        //         monster_specs_path.clone(),
        //         tokio::spawn(async move {
        // Result::<MonsterSpecs>::Ok(serde_json::from_slice(
        //     &fs::read(monster_specs_path).await?,
        // )?)
        //         }),
        //     );
        // }

        // let mut monster_specs = HashMap::new();
        // for (path, task) in monster_specs_loading_tasks {
        //     monster_specs.insert(path, task.await??);
        // }

        let monster_specs = join_all(monster_specs_to_load.iter().map(|path| {
            let path = path.clone();
            async move {
                let monster_specs = serde_json::from_slice(&fs::read(&path).await?)?;
                Result::<(PathBuf, MonsterSpecs)>::Ok((path, monster_specs))
            }
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

// TODO: MonsterPrototypesPool ?
