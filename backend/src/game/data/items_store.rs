use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use shared::data::{
    item::{ItemBase, ItemCategory},
    item_affix::{AffixTag, ItemAffixBlueprint},
};

use crate::game::utils::json::LoadJsonFromFile;

pub type ItemId = String;

pub type ItemsStore = HashMap<ItemId, ItemBase>;
pub type ItemAffixesTable = Vec<ItemAffixBlueprint>;

impl LoadJsonFromFile for ItemBase {}
impl LoadJsonFromFile for ItemAffixBlueprint {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Adjective {
    pub text: String,
    pub tags: Vec<AffixTag>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Noun {
    pub text: String,
    pub restrictions: Vec<ItemCategory>,
}

pub type ItemAdjectivesTable = Vec<Adjective>;
pub type ItemNounsTable = Vec<Noun>;

impl LoadJsonFromFile for Adjective {}
impl LoadJsonFromFile for Noun {}
