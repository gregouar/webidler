use backend_shared::signature::HmacKey;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};

use shared::data::{
    item::{BaseItemId, ItemBase, ItemCategory, ItemSpecs},
    item_affix::{AffixTag, ItemAffixBlueprint},
};

use crate::{
    game::{data::indexed_store::IndexedStore, utils::json::LoadJsonFromFile},
    rest::AppError,
};

pub type ItemsStoreContent = IndexedStore<String, ItemBase>;
pub type SoulBasesByCategories = HashMap<BTreeSet<ItemCategory>, BaseItemId>;

#[derive(Debug, Clone)]
pub struct ItemsStore {
    pub items: ItemsStoreContent,
    pub affixes_table: ItemAffixesTable,
    pub adjectives_table: ItemAdjectivesTable,
    pub nouns_table: ItemNounsTable,

    pub soul_by_categories: SoulBasesByCategories,
    pub signature_key: HmacKey,
}

impl ItemsStore {
    pub fn new(
        items: ItemsStoreContent,
        affixes_table: ItemAffixesTable,
        adjectives_table: ItemAdjectivesTable,
        nouns_table: ItemNounsTable,
        signature_key: HmacKey,
    ) -> Self {
        let items = items.into_indexed_keys();
        let soul_by_categories = items
            .iter()
            .filter(|(_, base)| {
                base.categories.contains(&ItemCategory::Soul)
                    && base.rarity != shared::data::item::ItemRarity::Unique
            })
            .map(|(item_id, base)| {
                let mut categories = base.categories.clone();
                categories.remove(&ItemCategory::Soul);
                (categories, item_id.clone())
            })
            .collect();

        Self {
            items,
            affixes_table,
            adjectives_table,
            nouns_table,
            soul_by_categories,
            signature_key,
        }
    }

    pub fn get_item_base(&self, item_specs: &ItemSpecs) -> Result<&ItemBase, AppError> {
        self.items
            .get(&item_specs.modifiers.base_item_id)
            .ok_or(AppError::NotFound)
    }
}

pub type ItemAffixesTable = HashMap<String, ItemAffixBlueprint>;

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
