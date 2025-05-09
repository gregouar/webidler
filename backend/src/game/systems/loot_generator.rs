use shared::data::{
    item::{ItemBase, ItemRarity, ItemSpecs},
    world::AreaLevel,
};

use crate::game::utils::rng;

use super::{
    items_controller,
    items_table::ItemsTable,
    loot_table::{LootTable, RarityWeights},
};

// TODO: inc magic find (accumulated enemy rarity? player stats? area level?)
pub fn generate_loot(
    loot_table: &LootTable,
    items_table: &ItemsTable,
    // affixes_table: &AffixesTable,
    area_level: AreaLevel,
) -> Option<ItemSpecs> {
    let base_item = match roll_base_item(loot_table, items_table, area_level) {
        Some(i) => i,
        None => return None,
    };

    let rarity = roll_rarity(&RarityWeights::default()).max(base_item.rarity);

    // TODO: roll_affixes_effects(base_item.affixes)
    let affixes = vec![];
    // TODO:
    // let affixes = roll_affixes(base_item, affixes_table, rarity);

    Some(items_controller::update_item_specs(ItemSpecs {
        base: base_item,
        rarity: rarity,
        level: area_level,
        armor_specs: None,
        weapon_specs: None,
        affixes: affixes,
    }))
}

fn roll_rarity(weights: &RarityWeights) -> ItemRarity {
    match rng::random_range(0..(weights.normal + weights.magic + weights.rare)).unwrap_or(0) {
        r if r < weights.normal => ItemRarity::Normal,
        r if r < weights.normal + weights.magic => ItemRarity::Magic,
        _ => ItemRarity::Rare,
    }
}

fn roll_base_item(
    loot_table: &LootTable,
    items_table: &ItemsTable,
    area_level: AreaLevel,
) -> Option<ItemBase> {
    let items_available: Vec<_> = loot_table
        .entries
        .iter()
        .filter(|l| {
            area_level >= l.min_area_level.unwrap_or(AreaLevel::MIN)
                && area_level <= l.max_area_level.unwrap_or(AreaLevel::MAX)
        })
        .collect();

    rng::random_range(
        0.0..items_available
            .iter()
            .map(|loot_entry| loot_entry.weight)
            .sum(),
    )
    .and_then(|p| {
        items_available
            .iter()
            .scan(0.0, |cumul_prob, &loot_entry| {
                *cumul_prob += loot_entry.weight;
                Some((*cumul_prob, loot_entry))
            })
            .find(|(max_prob, loot_entry)| p >= *max_prob - loot_entry.weight && p < *max_prob)
            .map(|(_, loot_entry)| loot_entry)
    })
    .and_then(|loot_entry| items_table.entries.get(&loot_entry.item_id).cloned())
}
