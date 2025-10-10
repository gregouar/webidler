use anyhow::{Context, Result};
use std::{collections::HashMap, path::Path};

use serde::de::DeserializeOwned;
use serde_ignored;
use serde_json::de::Deserializer;
use tokio::fs;

// pub async fn load_json<S>(filepath: impl AsRef<Path>) -> Result<S>
// where
//     S: DeserializeOwned,
// {
//     serde_json::from_slice(
//         &fs::read(&filepath)
//             .await
//             .with_context(|| format!("Failed to read file: {:?}", filepath.as_ref()))?,
//     )
//     .with_context(|| format!("Failed to parse json from: {:?}", filepath.as_ref()))
// }

pub async fn load_json<S>(filepath: impl AsRef<Path>) -> Result<S>
where
    S: DeserializeOwned,
{
    let bytes = fs::read(&filepath)
        .await
        .with_context(|| format!("Failed to read file: {:?}", filepath.as_ref()))?;

    let mut de = Deserializer::from_slice(&bytes);

    let mut unknown = Vec::new();
    let value = serde_ignored::deserialize(&mut de, |path| {
        unknown.push(path.to_string());
    })
    .with_context(|| format!("Failed to parse json from: {:?}", filepath.as_ref()))?;

    if !unknown.is_empty() {
        anyhow::bail!(
            "Unknown fields in {}: {}",
            filepath.as_ref().display(),
            unknown.join(", ")
        );
    }

    Ok(value)
}

pub trait LoadJsonFromFile
where
    Self: Sized + DeserializeOwned + Send + Sync + 'static,
{
    fn load_from_file<P>(filepath: P) -> impl std::future::Future<Output = Result<Self>> + Send
    where
        P: AsRef<Path> + Send + Sync + 'static,
    {
        async { load_json(filepath).await }
    }
}

impl<T: LoadJsonFromFile> LoadJsonFromFile for HashMap<String, T> {}
impl<T: LoadJsonFromFile> LoadJsonFromFile for Vec<T> {}
