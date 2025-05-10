use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use shared::data::item::ItemBase;

use crate::game::utils::json::LoadJsonFromFile;

pub type ItemId = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemsTable {
    pub entries: HashMap<ItemId, ItemBase>,
}

impl LoadJsonFromFile for ItemsTable {}
