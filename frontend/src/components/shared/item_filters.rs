use std::collections::BTreeSet;

use indexmap::IndexMap;
use leptos::prelude::Read;

use shared::{
    data::item::{ItemCategory, ItemRarity},
    types::ItemName,
};

use crate::components::data_context::DataContext;

pub fn item_name_options(
    data_context: DataContext,
    item_category: Option<ItemCategory>,
    item_rarity: Option<ItemRarity>,
) -> IndexMap<Option<ItemName>, String> {
    let base_rarity = item_rarity.map(|rarity| match rarity {
        ItemRarity::Normal | ItemRarity::Magic | ItemRarity::Rare => ItemRarity::Normal,
        ItemRarity::Masterwork => ItemRarity::Masterwork,
        ItemRarity::Unique => ItemRarity::Unique,
    });

    std::iter::once((None, "All Items".to_string()))
        .chain(
            data_context
                .item_bases
                .read()
                .values()
                .filter(|item| {
                    item_category.is_none_or(|category| item.categories.contains(&category))
                        && base_rarity.is_none_or(|rarity| item.rarity == rarity)
                })
                .map(|item| item.name.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .filter_map(|name| {
                    ItemName::try_new(name.clone())
                        .ok()
                        .map(|key| (Some(key), name))
                }),
        )
        .collect()
}
