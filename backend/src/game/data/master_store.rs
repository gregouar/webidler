use anyhow::{anyhow, Result};
use futures::future::join_all;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use shared::data::{monster::MonsterSpecs, passive::PassivesTreeSpecs, skill::BaseSkillSpecs};

use super::{
    area::AreaBlueprint,
    items_store::{ItemAdjectivesTable, ItemAffixesTable, ItemNounsTable, ItemsStore},
    loot_table::LootTable,
    manifest,
    monster::BaseMonsterSpecs,
};

use crate::game::{data::manifest::ManifestCategory, utils::json::LoadJsonFromFile};

// TODO: Load from zip/dat file and compress at build time for prod release

pub type PassivesStore = HashMap<String, PassivesTreeSpecs>;
pub type SkillsStore = HashMap<String, BaseSkillSpecs>;
pub type MonstersSpecsStore = HashMap<String, BaseMonsterSpecs>;
pub type LootTablesStore = HashMap<String, LootTable>;
pub type AreaBlueprintStore = HashMap<String, AreaBlueprint>;

#[derive(Debug, Clone)]
pub struct MasterStore {
    pub passives_store: Arc<PassivesStore>,
    pub skills_store: Arc<SkillsStore>,
    pub items_store: Arc<ItemsStore>,
    pub item_affixes_table: Arc<ItemAffixesTable>,
    pub item_adjectives_table: Arc<ItemAdjectivesTable>,
    pub item_nouns_table: Arc<ItemNounsTable>,
    pub loot_tables_store: Arc<LootTablesStore>,
    pub monster_specs_store: Arc<MonstersSpecsStore>,
    pub area_blueprints_store: Arc<AreaBlueprintStore>,
}

impl LoadJsonFromFile for MonsterSpecs {}
impl LoadJsonFromFile for BaseSkillSpecs {}
impl LoadJsonFromFile for PassivesTreeSpecs {}

impl MasterStore {
    pub async fn load_from_folder(folder_path: impl AsRef<Path>) -> Result<Self> {
        let manifest = manifest::load_manifest(folder_path).await?;

        let (
            passives_store,
            skills_store,
            items_store,
            item_affixes_table,
            item_adjectives_table,
            item_nouns_table,
            loot_tables_store,
            monster_specs_store,
        ) = tokio::join!(
            join_load_and_merge_tables(manifest.get_resources(ManifestCategory::Passives)),
            join_load_and_merge_tables(manifest.get_resources(ManifestCategory::Skills)),
            join_load_and_merge_tables(manifest.get_resources(ManifestCategory::Items)),
            join_load_and_merge_tables(manifest.get_resources(ManifestCategory::ItemAffixes)),
            join_load_and_merge_tables(manifest.get_resources(ManifestCategory::ItemAdjectives)),
            join_load_and_merge_tables(manifest.get_resources(ManifestCategory::ItemNouns)),
            join_load_and_map(manifest.get_resources(ManifestCategory::Loot)),
            join_load_and_map(manifest.get_resources(ManifestCategory::Monsters)),
        );

        let loot_tables_store = loot_tables_store?;

        let area_blueprints_store =
            join_load_and_map(manifest.get_resources(ManifestCategory::Areas))
                .await?
                .into_iter()
                .map(|(f, schema)| {
                    Ok((
                        f,
                        AreaBlueprint::populate_from_schema(schema, &loot_tables_store)?,
                    ))
                })
                .collect::<Result<_>>()?;

        let master_store = MasterStore {
            passives_store: Arc::new(passives_store?),
            skills_store: Arc::new(skills_store?),
            items_store: Arc::new(items_store?),
            item_affixes_table: Arc::new(item_affixes_table?),
            item_adjectives_table: Arc::new(item_adjectives_table?),
            item_nouns_table: Arc::new(item_nouns_table?),
            loot_tables_store: Arc::new(loot_tables_store),
            monster_specs_store: Arc::new(monster_specs_store?),
            area_blueprints_store: Arc::new(area_blueprints_store),
        };

        verify_store_integrity(&master_store)?;

        Ok(master_store)
    }
}

/// Load several files in parallel and store the results in a hash map
async fn join_load_and_map<T: LoadJsonFromFile>(paths: Vec<PathBuf>) -> Result<HashMap<String, T>> {
    join_all(paths.into_iter().map(|f| async move {
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
    .collect::<Result<_>>()
}

async fn join_load_and_merge_tables<T>(paths: Vec<PathBuf>) -> Result<T>
where
    T: LoadJsonFromFile
        + IntoIterator
        + std::iter::FromIterator<<T as std::iter::IntoIterator>::Item>,
{
    let table: Vec<_> = join_all(
        paths
            .into_iter()
            .map(|f| async move { T::load_from_file(f).await }),
    )
    .await
    .into_iter()
    .collect::<Result<_>>()?;

    Ok(table.into_iter().flatten().collect())
}

fn verify_store_integrity(master_store: &MasterStore) -> Result<()> {
    let mut errors = Vec::new();

    for loot in master_store
        .loot_tables_store
        .values()
        .flat_map(|t| &t.entries)
    {
        if master_store.items_store.get(&loot.item_id).is_none() {
            errors.push(anyhow!("Missing item '{}' from store", loot.item_id));
        }
    }

    for spawn in master_store.area_blueprints_store.values().flat_map(|w| {
        w.bosses
            .iter()
            .flat_map(|b| &b.spawns)
            .chain(w.waves.iter().flat_map(|w| &w.spawns))
    }) {
        if master_store
            .monster_specs_store
            .get(&spawn.monster)
            .is_none()
        {
            errors.push(anyhow!("Missing monster '{}' from store", spawn.monster));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(anyhow!(
            "Store integrity check failed:\n{}",
            errors
                .iter()
                .map(|e| format!(" - {e}"))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_integrity() -> Result<(), Box<dyn std::error::Error>> {
        MasterStore::load_from_folder("../data").await?;
        Ok(())
    }
}
