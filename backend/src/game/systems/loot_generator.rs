use std::collections::HashSet;

use shared::data::{
    item::{ItemBase, ItemRarity, ItemSpecs},
    item_affix::{AffixEffect, AffixEffectBlueprint, AffixType, ItemAffix, ItemAffixBlueprint},
    world::AreaLevel,
};

use crate::game::utils::rng;
use crate::game::{
    data::{
        items_table::{ItemAffixesTable, ItemsTable},
        loot_table::{LootTable, LootTableEntry, RarityWeights},
    },
    utils::rng::RandomWeighted,
};

use super::items_controller;

impl rng::RandomWeighted for ItemAffixBlueprint {
    fn random_weight(&self) -> f64 {
        self.weight as f64
    }
}

impl RandomWeighted for LootTableEntry {
    fn random_weight(&self) -> f64 {
        self.weight as f64
    }
}

// TODO: inc magic find (accumulated enemy rarity? player stats? area level?)
pub fn generate_loot(
    loot_table: &LootTable,
    items_table: &ItemsTable,
    affixes_table: &ItemAffixesTable,
    area_level: AreaLevel,
) -> Option<ItemSpecs> {
    let base_item = match roll_base_item(loot_table, items_table, area_level) {
        Some(i) => i,
        None => return None,
    };

    let rarity = roll_rarity(&RarityWeights::default()).max(base_item.rarity);

    let mut affixes: Vec<ItemAffix> = base_item
        .affixes
        .iter()
        .map(|e| ItemAffix {
            name: "Unique".to_string(),
            family: base_item.name.clone(),
            affix_type: AffixType::Unique,
            tier: 1,
            effects: vec![roll_affix_effect(e)],
        })
        .collect();

    let (min_affixes, max_affixes) = match rarity {
        ItemRarity::Magic => (0, 1),
        ItemRarity::Rare => (1, 2),
        _ => (0, 0),
    };

    let prefixes_amount = rng::random_range(min_affixes..=max_affixes).unwrap_or_default();
    let suffixes_amount = rng::random_range(min_affixes..=max_affixes).unwrap_or_default();

    let mut families_in_use: HashSet<String> = HashSet::new();

    affixes.extend((0..prefixes_amount).filter_map(|_| {
        roll_affix(
            &base_item,
            area_level,
            AffixType::Prefix,
            &mut families_in_use,
            affixes_table,
        )
    }));
    affixes.extend((0..suffixes_amount).filter_map(|_| {
        roll_affix(
            &base_item,
            area_level,
            AffixType::Suffix,
            &mut families_in_use,
            affixes_table,
        )
    }));

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

    rng::random_weighted_pick(items_available)
        .and_then(|loot_entry| items_table.get(&loot_entry.item_id).cloned())
}

fn roll_affix(
    base_item: &ItemBase,
    area_level: AreaLevel,
    affix_type: AffixType,
    families_in_use: &mut HashSet<String>,
    affixes_table: &ItemAffixesTable,
) -> Option<ItemAffix> {
    let available_affixes: Vec<_> = affixes_table
        .iter()
        .filter(|a| {
            a.slots.contains(&base_item.slot)
                && area_level >= a.item_level
                && a.affix_type == affix_type
                && !families_in_use.contains(&a.family)
        })
        .collect();

    rng::random_weighted_pick(available_affixes).map(|a| {
        families_in_use.insert(a.family.clone());
        ItemAffix {
            name: a.name.clone(),
            family: a.family.clone(),
            affix_type,
            tier: a.tier,
            effects: a.effects.iter().map(|e| roll_affix_effect(e)).collect(),
        }
    })
}

fn roll_affix_effect(effect_blueprint: &AffixEffectBlueprint) -> AffixEffect {
    AffixEffect {
        stat: effect_blueprint.stat,
        modifier: effect_blueprint.modifier,
        value: rng::random_range(effect_blueprint.min..=effect_blueprint.max).unwrap_or_default(),
    }
}
