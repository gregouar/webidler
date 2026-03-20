use backend_shared::signature::HmacKey;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use shared::data::{
    item::{ItemBase, ItemCategory},
    item_affix::{AffixTag, ItemAffixBlueprint},
};

use crate::game::utils::json::LoadJsonFromFile;

pub type ItemId = String;

pub type ItemsStoreContent = HashMap<ItemId, ItemBase>;

#[derive(Debug, Clone)]
pub struct ItemsStore {
    pub content: ItemsStoreContent,
    pub signature_key: HmacKey,
}

pub type ItemAffixesTable = Vec<ItemAffixBlueprint>;

impl LoadJsonFromFile for ItemBase {}
impl LoadJsonFromFile for ItemAffixBlueprint {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Adjective {
    pub text: String,
    pub tags: HashSet<AffixTag>,
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
