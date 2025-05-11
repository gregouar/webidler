use anyhow::{Context, Result};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::game::utils::json::LoadJsonFromFile;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ManifestCategory {
    Items,
    ItemAffixes,
    ItemAdjectives,
    ItemNouns,
    Loot,
    Monsters,
    Worlds,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Manifest {
    pub meta: ManifestMeta,

    #[serde(default)]
    pub folders: Vec<PathBuf>,

    // TODO: Switch to enum and hashmap?
    #[serde(default)]
    pub items: Vec<PathBuf>,
    #[serde(default)]
    pub item_affixes: Vec<PathBuf>,
    #[serde(default)]
    pub item_adjectives: Vec<PathBuf>,
    #[serde(default)]
    pub item_nouns: Vec<PathBuf>,
    #[serde(default)]
    pub loot: Vec<PathBuf>,
    #[serde(default)]
    pub monsters: Vec<PathBuf>,
    #[serde(default)]
    pub worlds: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ManifestMeta {
    enabled: bool,
}

impl LoadJsonFromFile for Manifest {}

pub async fn load_manifest(folder_path: impl AsRef<Path>) -> Result<Manifest> {
    let folder_path = folder_path.as_ref();

    let local_manifest = Manifest::load_from_file(folder_path.join(".manifest.json"))
        .await
        .context(format!(
            "Failed to load folder manifest from {:?}",
            folder_path
        ))?;

    if !local_manifest.meta.enabled {
        return Ok(Manifest::default());
    }

    // Add parent path to have full paths to resources
    let join_paths = |paths: Vec<PathBuf>| paths.into_iter().map(|f| folder_path.join(f)).collect();
    let manifest = Manifest {
        meta: local_manifest.meta,
        folders: join_paths(local_manifest.folders),
        item_affixes: join_paths(local_manifest.item_affixes),
        item_adjectives: join_paths(local_manifest.item_adjectives),
        item_nouns: join_paths(local_manifest.item_nouns),
        items: join_paths(local_manifest.items),
        loot: join_paths(local_manifest.loot),
        monsters: join_paths(local_manifest.monsters),
        worlds: join_paths(local_manifest.worlds),
    };

    // Load sub-folders
    let sub_manifests: Vec<_> = join_all(
        manifest
            .folders
            .iter()
            .map(|f| async move { Result::<_>::Ok(load_manifest(f).await?) }),
    )
    .await
    .into_iter()
    .collect::<Result<_>>()?;

    // Merge all folders
    Ok(sub_manifests
        .into_iter()
        .fold(manifest, |manifest, sub| manifest.merge(sub)))
}

impl Manifest {
    fn merge(mut self, other: Manifest) -> Self {
        self.folders.extend(other.folders);
        self.item_affixes.extend(other.item_affixes);
        self.item_adjectives.extend(other.item_adjectives);
        self.item_nouns.extend(other.item_nouns);
        self.items.extend(other.items);
        self.loot.extend(other.loot);
        self.monsters.extend(other.monsters);
        self.worlds.extend(other.worlds);
        self
    }
}
