use std::collections::HashSet;

use shared::data::{
    item::{ItemBase, ItemRarity, ItemSpecs},
    item_affix::{StatEffect, AffixEffectBlueprint, AffixType, ItemAffix, ItemAffixBlueprint},
    world::AreaLevel,
};

use crate::game::{
    data::items_store::{ItemAdjectivesTable, ItemNounsTable},
    utils::rng,
};
use crate::game::{
    data::{
        items_store::{ItemAffixesTable, ItemsStore},
        loot_table::{LootTable, LootTableEntry, RarityWeights},
    },
    utils::rng::RandomWeighted,
};

use super::items_controller;

impl rng::RandomWeighted for &ItemAffixBlueprint {
    fn random_weight(&self) -> u64 {
        self.weight
    }
}

impl RandomWeighted for &LootTableEntry {
    fn random_weight(&self) -> u64 {
        self.weight
    }
}

// TODO: inc magic find (accumulated enemy rarity? player stats? area level?)
pub fn generate_loot(
    level: AreaLevel,
    loot_table: &LootTable,
    items_store: &ItemsStore,
    affixes_table: &ItemAffixesTable,
    adjectives_table: &ItemAdjectivesTable,
    nouns_table: &ItemNounsTable,
) -> Option<ItemSpecs> {
    roll_base_item(loot_table, items_store, level).map(|base| {
        let rarity = roll_rarity(&RarityWeights::default()).max(base.rarity);
        roll_item(
            base,
            rarity,
            level,
            affixes_table,
            adjectives_table,
            nouns_table,
        )
    })
}

fn roll_rarity(weights: &RarityWeights) -> ItemRarity {
    match rng::random_range(0..(weights.normal + weights.magic + weights.rare)).unwrap_or(0) {
        r if r < weights.normal => ItemRarity::Normal,
        r if r < weights.normal + weights.magic => ItemRarity::Magic,
        _ => ItemRarity::Rare,
    }
}

pub fn roll_item(
    base: ItemBase,
    rarity: ItemRarity,
    level: AreaLevel,
    affixes_table: &ItemAffixesTable,
    adjectives_table: &ItemAdjectivesTable,
    nouns_table: &ItemNounsTable,
) -> ItemSpecs {
    let mut affixes: Vec<ItemAffix> = roll_unique_affixes(&base);

    let (prefixes_amount, suffixes_amount) = match rarity {
        ItemRarity::Magic => roll_affixes_amount(1, 2, 0, 1),
        ItemRarity::Rare => roll_affixes_amount(3, 4, 1, 2),
        _ => (0, 0),
    };

    let mut families_in_use: HashSet<String> = HashSet::new();

    let prefixes: Vec<_> = (0..prefixes_amount)
        .filter_map(|_| {
            roll_affix(
                &base,
                level,
                AffixType::Prefix,
                &mut families_in_use,
                affixes_table,
            )
        })
        .collect();
    affixes.extend(prefixes);

    let suffixes: Vec<_> = (0..suffixes_amount)
        .filter_map(|_| {
            roll_affix(
                &base,
                level,
                AffixType::Suffix,
                &mut families_in_use,
                affixes_table,
            )
        })
        .collect();
    affixes.extend(suffixes);

    items_controller::update_item_specs(
        ItemSpecs {
            name: base.name.clone(),
            base,
            rarity,
            level,
            armor_specs: None,
            weapon_specs: None,
            affixes,
        },
        adjectives_table,
        nouns_table,
    )
}

fn roll_base_item(
    loot_table: &LootTable,
    items_store: &ItemsStore,
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

    rng::random_weighted_pick(&items_available)
        .and_then(|loot_entry| items_store.get(&loot_entry.item_id).cloned())
}

fn roll_unique_affixes(base_item: &ItemBase) -> Vec<ItemAffix> {
    base_item
        .affixes
        .iter()
        .map(|e| ItemAffix {
            name: "Unique".to_string(),
            family: base_item.name.clone(),
            tags: HashSet::new(),
            affix_type: AffixType::Unique,
            tier: 1,
            effects: vec![roll_affix_effect(e)],
        })
        .collect()
}

fn roll_affixes_amount(
    min_amount: usize,
    max_amount: usize,
    min_prefixes: usize,
    max_prefixes: usize,
) -> (usize, usize) {
    let amount = rng::random_range(min_amount..=max_amount).unwrap_or(min_amount);
    let prefix_count = rng::random_range(min_prefixes..=max_prefixes).unwrap_or(min_prefixes);
    (prefix_count, amount.checked_sub(prefix_count).unwrap_or(0))
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
            a.restrictions
                .as_ref()
                .map(|r| !r.is_disjoint(&base_item.affix_restrictions))
                .unwrap_or(true)
                && area_level >= a.item_level
                && a.affix_type == affix_type
                && !families_in_use.contains(&a.family)
        })
        .collect();

    rng::random_weighted_pick(&available_affixes).map(|a| {
        families_in_use.insert(a.family.clone());
        ItemAffix {
            name: a.name.clone(),
            family: a.family.clone(),
            tags: a.tags.clone(),
            affix_type,
            tier: a.tier,
            effects: a.effects.iter().map(|e| roll_affix_effect(e)).collect(),
        }
    })
}

fn roll_affix_effect(effect_blueprint: &AffixEffectBlueprint) -> StatEffect {
    StatEffect {
        stat: effect_blueprint.stat,
        modifier: effect_blueprint.modifier,
        value: rng::random_range(effect_blueprint.min..=effect_blueprint.max).unwrap_or_default(),
    }
}

pub fn generate_name(
    item_specs: &ItemSpecs,
    adjectives_table: &ItemAdjectivesTable,
    nouns_table: &ItemNounsTable,
) -> String {
    match item_specs.rarity {
        ItemRarity::Magic => generate_magic_name(item_specs),
        ItemRarity::Rare => generate_rare_name(item_specs, adjectives_table, nouns_table),
        _ => item_specs.base.name.clone(),
    }
}

fn generate_magic_name(item_specs: &ItemSpecs) -> String {
    let mut name = item_specs.base.name.clone();

    let prefixes: Vec<_> = item_specs
        .affixes
        .iter()
        .filter(|a| a.affix_type == AffixType::Prefix)
        .collect();

    if prefixes.len() == 1 {
        name = format!("{} {}", prefixes[0].name, name);
    }

    let suffixes: Vec<_> = item_specs
        .affixes
        .iter()
        .filter(|a| a.affix_type == AffixType::Suffix)
        .collect();

    if suffixes.len() == 1 {
        name = format!("{} {}", name, suffixes[0].name);
    };

    name
}

struct WeightedNamePart<'a> {
    text: &'a str,
    weight: u64,
}

impl<'a> rng::RandomWeighted for WeightedNamePart<'a> {
    fn random_weight(&self) -> u64 {
        self.weight
    }
}

fn generate_rare_name(
    item_specs: &ItemSpecs,
    adjectives_table: &ItemAdjectivesTable,
    nouns_table: &ItemNounsTable,
) -> String {
    let tags: HashSet<_> = item_specs
        .affixes
        .iter()
        .flat_map(|a| a.tags.iter())
        .collect();

    let available_adjectives: Vec<_> = adjectives_table
        .iter()
        .map(|part| WeightedNamePart {
            text: &part.text,
            weight: part.tags.iter().filter(|t| tags.contains(t)).count() as u64,
        })
        .collect();

    let available_nouns: Vec<_> = nouns_table
        .iter()
        .map(|part| WeightedNamePart {
            text: &part.text,
            weight: part
                .restrictions
                .iter()
                .filter(|t| item_specs.base.affix_restrictions.contains(t))
                .count() as u64,
        })
        .collect();

    format!(
        "{} {}",
        rng::random_weighted_pick(&available_adjectives)
            .map(|part| part.text)
            .unwrap_or("Mysterious"),
        rng::random_weighted_pick(&available_nouns)
            .map(|part| part.text)
            .unwrap_or("Artifact")
    )
}
