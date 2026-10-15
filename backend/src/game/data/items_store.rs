use backend_shared::signature::HmacKey;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};

use shared::data::{
    item::{ItemBase, ItemCategory},
    item_affix::{AffixTag, ItemAffixBlueprint},
};

use crate::game::utils::json::LoadJsonFromFile;

pub type ItemId = String;

pub type ItemsStoreContent = HashMap<ItemId, ItemBase>;
pub type SoulBasesByCategories = HashMap<BTreeSet<ItemCategory>, ItemId>;

#[derive(Debug, Clone)]
pub struct ItemsStore {
    pub content: ItemsStoreContent,
    pub soul_by_categories: SoulBasesByCategories,
    pub signature_key: HmacKey,
}

impl ItemsStore {
    pub fn new(content: ItemsStoreContent, signature_key: HmacKey) -> Self {
        let soul_by_categories = content
            .iter()
            .filter(|(_, base)| base.categories.contains(&ItemCategory::Soul))
            .map(|(item_id, base)| {
                let mut categories = base.categories.clone();
                categories.remove(&ItemCategory::Soul);
                (categories, item_id.clone())
            })
            .collect();

        Self {
            content,
            soul_by_categories,
            signature_key,
        }
    }
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
