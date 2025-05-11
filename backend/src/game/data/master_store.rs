use anyhow::Result;
use futures::future::join_all;
use shared::data::monster::MonsterSpecs;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::game::utils::json::LoadJsonFromFile;

use super::{items_table::ItemsTable, loot_table::LootTable, manifest, world::WorldBlueprint};

// TODO: Load from zip/dat file and compress at build time for prod release

pub type MonstersSpecsStore = HashMap<String, MonsterSpecs>;
pub type LootTablesStore = HashMap<String, LootTable>;
pub type WorldBlueprintStore = HashMap<String, WorldBlueprint>;

#[derive(Debug, Clone)]
pub struct MasterStore {
    pub items_table: Arc<ItemsTable>,
    // affixes_tables
    pub loot_tables_store: Arc<LootTablesStore>,
    pub monster_specs_store: Arc<MonstersSpecsStore>,
    pub world_blueprints_store: Arc<WorldBlueprintStore>,
}

impl LoadJsonFromFile for MonsterSpecs {}

impl MasterStore {
    pub async fn load_from_folder(folder_path: impl AsRef<Path>) -> Result<Self> {
        let manifest = manifest::load_manifest(folder_path).await?;

        // TODO join
        let items_table = load_items_tables(manifest.items).await?;
        let loot_tables_store = join_load_and_map(manifest.loot).await?;
        let monster_specs_store = join_load_and_map(manifest.monsters).await?;
        let world_blueprints_store = join_load_and_map(manifest.worlds)
            .await?
            .into_iter()
            .map(|(f, schema)| {
                Ok((
                    f,
                    WorldBlueprint::populate_from_schema(schema, &loot_tables_store)?,
                ))
            })
            .collect::<Result<_>>()?;

        Ok(MasterStore {
            items_table: Arc::new(items_table),
            loot_tables_store: Arc::new(loot_tables_store),
            monster_specs_store: Arc::new(monster_specs_store),
            world_blueprints_store: Arc::new(world_blueprints_store),
        })
    }
}

/// Load several files in parallel and store the results in a hash map
async fn join_load_and_map<T: LoadJsonFromFile>(paths: Vec<PathBuf>) -> Result<HashMap<String, T>> {
    Ok(join_all(paths.into_iter().map(|f| async move {
        Result::<_>::Ok((
            f.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            T::load_from_file(f).await?,
        ))
    }))
    .await
    .into_iter()
    .collect::<Result<_>>()?)
}

async fn load_items_tables(paths: Vec<PathBuf>) -> Result<ItemsTable> {
    let items_tables: Vec<_> = join_all(
        paths
            .into_iter()
            .map(|f| async move { Result::<_>::Ok(ItemsTable::load_from_file(f).await?) }),
    )
    .await
    .into_iter()
    .collect::<Result<_>>()?;

    Ok(ItemsTable {
        entries: items_tables.into_iter().flat_map(|x| x.entries).collect(),
    })
}
