use std::collections::HashMap;

use shared::data::{item::ItemBase, item_affix::ItemAffixBlueprint};

use crate::game::utils::json::LoadJsonFromFile;

pub type ItemId = String;

pub type ItemsTable = HashMap<ItemId, ItemBase>;
pub type ItemAffixesTable = Vec<ItemAffixBlueprint>;

impl LoadJsonFromFile for ItemBase {}
impl LoadJsonFromFile for ItemAffixBlueprint {}
