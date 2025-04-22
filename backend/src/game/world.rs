use anyhow::Result;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::u16;

use serde::{Deserialize, Serialize};

use futures::future::join_all;

use super::data::load_schema;
use shared::data::{MonsterSpecs, WorldSpecs, WorldState};

const MAX_MONSTERS: usize = 6;

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
    min_level: Option<u16>,
    max_level: Option<u16>,
    probability: f64,
    spawns: Vec<MonsterWaveSpawnBlueprint>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MonsterWaveSpawnBlueprint {
    pub path: PathBuf,
    pub min_quantity: u8,
    pub max_quantity: u8,
}

impl WorldBlueprint {
    pub async fn load_from_file(filepath: PathBuf) -> Result<Self> {
        let schema: WorldBlueprintSchema = load_schema(&filepath).await?;

        let monster_specs_to_load: HashSet<PathBuf> = schema
            .waves
            .iter()
            .flat_map(|wave| wave.spawns.iter().map(|spawn| spawn.path.clone()))
            .collect();

        let monster_specs = join_all(monster_specs_to_load.into_iter().map(|path| async move {
            let monster_specs = load_schema(&path).await?;
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

    // TODO: Move to system?
    pub fn generate_monsters_wave(&self, world_state: &WorldState) -> Result<Vec<MonsterSpecs>> {
        let mut rng = rand::rng();

        let waves: Vec<&MonsterWaveBlueprint> = self
            .schema
            .waves
            .iter()
            .filter(|wave| {
                world_state.area_level >= wave.min_level.unwrap_or(0)
                    && world_state.area_level <= wave.max_level.unwrap_or(u16::MAX)
            })
            .collect();

        let total_probability = waves.iter().map(|w| w.probability).sum();
        if total_probability <= 0.0 {
            return Err(anyhow::format_err!(
                "no monsters wave probability available"
            ));
        }

        let p = rng.random_range(0.0..total_probability);

        // TODO
        let wave = waves.first();

        let mut monsters_specs = Vec::with_capacity(MAX_MONSTERS);
        if let Some(wave) = wave {
            for spawn in wave.spawns.iter() {
                if spawn.max_quantity < spawn.min_quantity {
                    return Err(anyhow::format_err!(
                        "monster wave max_quantity below min_quantity"
                    ));
                }
                for _ in 0..rng.random_range(spawn.min_quantity..=spawn.max_quantity) {
                    let specs = self.monster_specs.get(&spawn.path);
                    if let Some(specs) = specs {
                        monsters_specs.push(specs.clone());
                    }
                }
            }
        }
        return Ok(monsters_specs);
    }
}

// TODO: Global MonsterPrototypesPool shared among instances ?
