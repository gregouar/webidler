use anyhow::{Context, Result};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::game::utils::json::LoadJsonFromFile;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ManifestCategory {
    Passives,
    Benedictions,
    Skills,
    Items,
    ItemAffixes,
    ItemAdjectives,
    ItemNouns,
    Loot,
    Monsters,
    Areas,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Manifest {
    pub meta: ManifestMeta,

    #[serde(default)]
    pub folders: Vec<PathBuf>,

    #[serde(default)]
    pub resources: HashMap<ManifestCategory, Vec<PathBuf>>,
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
            "Failed to load folder manifest from {folder_path:?}"
        ))?;

    if !local_manifest.meta.enabled {
        return Ok(Manifest::default());
    }

    // Add parent path to have full paths to resources
    let join_paths = |paths: Vec<PathBuf>| paths.into_iter().map(|f| folder_path.join(f)).collect();
    let manifest = Manifest {
        meta: local_manifest.meta,
        folders: join_paths(local_manifest.folders),
        resources: local_manifest
            .resources
            .into_iter()
            .map(|(k, v)| (k, join_paths(v)))
            .collect(),
    };

    // Load sub-folders
    let sub_manifests: Vec<_> = join_all(
        manifest
            .folders
            .iter()
            .map(|f| async move { load_manifest(f).await }),
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
    pub fn get_resources(&self, category: ManifestCategory) -> Vec<PathBuf> {
        self.resources.get(&category).cloned().unwrap_or_default()
    }

    fn merge(mut self, other: Manifest) -> Self {
        self.folders.extend(other.folders);
        for (k, v) in other.resources.into_iter() {
            self.resources.entry(k).or_default().extend(v);
        }
        self
    }
}
