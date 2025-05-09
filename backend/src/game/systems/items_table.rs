use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use std::collections::HashMap;

use shared::data::item::ItemBase;

use crate::game::utils::json::load_json;

pub type ItemId = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemsTable {
    pub entries: HashMap<ItemId, ItemBase>,
}

impl ItemsTable {
    pub async fn load_from_file(filepath: impl Into<&PathBuf>) -> Result<Self> {
        Ok(load_json(filepath).await?)
    }
}
