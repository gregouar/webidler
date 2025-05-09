use anyhow::{Context, Result};
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use tokio::fs;

pub async fn load_json<S>(filepath: impl Into<&PathBuf>) -> Result<S>
where
    S: DeserializeOwned,
{
    let file_path = PathBuf::from("./data").join(filepath.into());
    Ok(serde_json::from_slice(
        &fs::read(&file_path)
            .await
            .with_context(|| format!("Failed to read file: {:?}", file_path))?,
    )
    .with_context(|| format!("Failed to parse json from: {:?}", file_path))?)
}
