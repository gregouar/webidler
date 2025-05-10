use anyhow::{Context, Result};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use shared::data::monster::MonsterSpecs;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::game::utils::json::LoadJsonFromFile;

use super::{items_table::ItemsTable, loot_table::LootTable, world::WorldBlueprint};

// TODO: Load from zip/dat file and compress at build time for prod release

pub type MonstersSpecsStore = HashMap<PathBuf, MonsterSpecs>;
pub type LootTablesStore = HashMap<PathBuf, LootTable>;
pub type WorldBlueprintStore = HashMap<PathBuf, WorldBlueprint>;

#[derive(Debug, Clone)]
pub struct MasterStore {
    pub items_table: Arc<ItemsTable>,
    // affixes_tables
    pub loot_tables_store: Arc<LootTablesStore>,
    pub monster_specs_store: Arc<MonstersSpecsStore>,
    pub world_blueprints_store: Arc<WorldBlueprintStore>,
}

// TODO: Have manifest more flexible, with instead of entries the different resources: folders, items, loot, monsters, worlds
// and then let it auto-explore
#[derive(Serialize, Deserialize, Debug, Clone)]
struct FolderManifest {
    entries: Vec<PathBuf>,
}

impl LoadJsonFromFile for FolderManifest {}
impl LoadJsonFromFile for MonsterSpecs {}

impl MasterStore {
    pub async fn load_from_folder(folder_path: impl Into<PathBuf>) -> Result<Self> {
        let folder_path: PathBuf = folder_path.into();

        let items_table = load_items_tables(&folder_path.join("items")).await?;
        let loot_tables_store = join_load_and_map(&folder_path.join("loot")).await?;
        let monster_specs_store = join_load_and_map(&folder_path.join("monsters")).await?;
        let world_blueprints_store = join_load_and_map(&folder_path.join("worlds"))
            .await?
            .into_iter()
            .map(|(path, schema)| {
                Ok((
                    path,
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

async fn load_manifest(folder_path: impl AsRef<Path>) -> Result<FolderManifest> {
    Ok(
        FolderManifest::load_from_file(folder_path.as_ref().join(".manifest.json"))
            .await
            .context(format!(
                "Failed to load folder manifest from {:?}",
                folder_path.as_ref()
            ))?,
    )
}

/// Load several files in parallel and store the results in a hash map
async fn join_load_and_map<T: LoadJsonFromFile>(
    folder_path: impl AsRef<Path>,
) -> Result<HashMap<PathBuf, T>> {
    let manifest = load_manifest(&folder_path).await?;

    let folder_path = folder_path.as_ref();
    Ok(join_all(manifest.entries.iter().map(|filename| async move {
        Result::<_>::Ok((
            filename.into(),
            T::load_from_file(folder_path.join(filename)).await?,
        ))
    }))
    .await
    .into_iter()
    .collect::<Result<_>>()?)
}

async fn load_items_tables(folder_path: impl AsRef<Path>) -> Result<ItemsTable> {
    let manifest = load_manifest(&folder_path).await?;

    let folder_path = folder_path.as_ref();
    let items_tables: Vec<_> = join_all(manifest.entries.iter().map(|filename| async move {
        Result::<_>::Ok(ItemsTable::load_from_file(folder_path.join(filename)).await?)
    }))
    .await
    .into_iter()
    .collect::<Result<_>>()?;

    Ok(ItemsTable {
        entries: items_tables.into_iter().flat_map(|x| x.entries).collect(),
    })
}
