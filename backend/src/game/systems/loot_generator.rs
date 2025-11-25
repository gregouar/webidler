use std::collections::HashSet;

use shared::{
    constants::{MAX_ITEM_QUALITY, MAX_ITEM_QUALITY_PER_LEVEL},
    data::{
        area::AreaLevel,
        chance::ChanceRange,
        forge::MAX_AFFIXES,
        item::{ItemBase, ItemModifiers, ItemRarity, ItemSpecs},
        item_affix::{AffixEffect, AffixEffectBlueprint, AffixType, ItemAffix, ItemAffixBlueprint},
        stat_effect::StatEffect,
    },
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
    utils::rng::{RandomWeighted, Rollable},
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
#[allow(clippy::too_many_arguments)]
pub fn generate_loot(
    level: AreaLevel,
    is_boss_level: bool,
    loot_table: &LootTable,
    items_store: &ItemsStore,
    affixes_table: &ItemAffixesTable,
    adjectives_table: &ItemAdjectivesTable,
    nouns_table: &ItemNounsTable,
    allow_unique: bool,
) -> Option<ItemSpecs> {
    let mut rarity = roll_rarity(&RarityWeights::default());
    if !allow_unique {
        rarity = rarity.min(ItemRarity::Rare);
    }
    roll_base_item(
        loot_table,
        items_store,
        level,
        is_boss_level,
        rarity == ItemRarity::Unique,
    )
    .map(|(base_item_id, base)| {
        if base.rarity != ItemRarity::Unique {
            rarity = rarity.min(ItemRarity::Rare);
        }
        rarity = rarity.max(base.rarity);
        roll_item(
            base_item_id,
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
    match rng::random_range(0..=(weights.normal + weights.magic + weights.rare + weights.unique))
        .unwrap_or(0)
    {
        r if r < weights.normal => ItemRarity::Normal,
        r if r < weights.normal + weights.magic => ItemRarity::Magic,
        r if r < weights.normal + weights.magic + weights.rare => ItemRarity::Rare,
        _ => ItemRarity::Unique,
    }
}

pub fn roll_item(
    base_item_id: String,
    base: ItemBase,
    rarity: ItemRarity,
    level: AreaLevel,
    affixes_table: &ItemAffixesTable,
    adjectives_table: &ItemAdjectivesTable,
    nouns_table: &ItemNounsTable,
) -> ItemSpecs {
    let quality = roll_quality(base.min_area_level, level);

    let mut modifiers = ItemModifiers {
        base_item_id,
        name: base.name.clone(),
        rarity: match rarity {
            ItemRarity::Unique => ItemRarity::Unique,
            _ => ItemRarity::Normal,
        },
        level,
        quality,
        affixes: roll_unique_affixes(&base, quality),
    };

    let affixes_amount = match rarity {
        ItemRarity::Magic => ChanceRange {
            min: 1,
            max: 2,
            ..Default::default()
        },
        ItemRarity::Rare => ChanceRange {
            min: 3,
            max: 4,
            ..Default::default()
        },
        _ => ChanceRange::default(),
    }
    .roll();

    for _ in 0..affixes_amount {
        add_affix(
            &base,
            &mut modifiers,
            None,
            affixes_table,
            adjectives_table,
            nouns_table,
        );
    }

    items_controller::create_item_specs(base, modifiers, false)
}

fn roll_quality(min_item_level: AreaLevel, level: AreaLevel) -> f32 {
    (rng::random_range(0..=level.saturating_sub(min_item_level)).unwrap_or_default() as f32
        * MAX_ITEM_QUALITY_PER_LEVEL)
        .min(MAX_ITEM_QUALITY)
}

fn roll_base_item(
    loot_table: &LootTable,
    items_store: &ItemsStore,
    area_level: AreaLevel,
    is_boss_level: bool,
    is_unique: bool,
) -> Option<(String, ItemBase)> {
    let items_available: Vec<_> = loot_table
        .entries
        .iter()
        .filter(|l| {
            area_level
                >= l.min_area_level.unwrap_or(
                    items_store
                        .get(&l.item_id)
                        .map(|i| i.min_area_level)
                        .unwrap_or(AreaLevel::MIN),
                )
                && area_level <= l.max_area_level.unwrap_or(AreaLevel::MAX)
                && (!l.boss_only || is_boss_level)
            // && (is_unique
            //     == items_store
            //         .get(&l.item_id)
            //         .map(|base| base.rarity == ItemRarity::Unique)
            //         .unwrap_or_default())
        })
        .collect();

    let items_available: Vec<_> = if is_unique {
        let uniques_available: Vec<_> = items_available
            .iter()
            .filter(|l| {
                items_store
                    .get(&l.item_id)
                    .map(|base| base.rarity == ItemRarity::Unique)
                    .unwrap_or_default()
            })
            .cloned()
            .collect();
        if uniques_available.is_empty() {
            items_available
        } else {
            uniques_available
        }
    } else {
        items_available
            .into_iter()
            .filter(|l| {
                items_store
                    .get(&l.item_id)
                    .map(|base| base.rarity != ItemRarity::Unique)
                    .unwrap_or_default()
            })
            .collect()
    };

    if items_available.is_empty() {
        tracing::warn!("No base items available for level {}", area_level);
    }

    rng::random_weighted_pick(&items_available).and_then(|loot_entry| {
        items_store
            .get(&loot_entry.item_id)
            .cloned()
            .map(|item_base| (loot_entry.item_id.clone(), item_base))
    })
}

fn roll_unique_affixes(base_item: &ItemBase, quality: f32) -> Vec<ItemAffix> {
    base_item
        .affixes
        .iter()
        .map(|e: &AffixEffectBlueprint| {
            let quality_factor = 1.0
                + if e.ignore_quality {
                    0.0
                } else {
                    quality as f64 * 0.01
                };
            let mut effect = roll_affix_effect(e);
            effect.stat_effect.value *= quality_factor;

            ItemAffix {
                name: "Unique".to_string(),
                family: base_item.name.clone(),
                tags: HashSet::new(),
                affix_type: AffixType::Unique,
                tier: 1,
                item_level: base_item.min_area_level,
                effects: vec![effect],
            }
        })
        .collect()
}

pub fn add_affix(
    base: &ItemBase,
    modifiers: &mut ItemModifiers,
    affix_type: Option<AffixType>,
    affixes_table: &ItemAffixesTable,
    adjectives_table: &ItemAdjectivesTable,
    nouns_table: &ItemNounsTable,
) -> bool {
    if base.rarity == ItemRarity::Unique {
        return false;
    }

    let prefixes_amount = modifiers.count_affixes(AffixType::Prefix);
    let suffixes_amount = modifiers.count_affixes(AffixType::Suffix);

    if prefixes_amount + suffixes_amount >= MAX_AFFIXES {
        return false;
    }

    let affix_type = match affix_type {
        Some(AffixType::Prefix) => {
            if prefixes_amount <= suffixes_amount {
                AffixType::Prefix
            } else {
                return false;
            }
        }
        Some(AffixType::Suffix) => {
            if suffixes_amount <= prefixes_amount {
                AffixType::Suffix
            } else {
                return false;
            }
        }
        _ => {
            if prefixes_amount < suffixes_amount {
                AffixType::Prefix
            } else if suffixes_amount < prefixes_amount {
                AffixType::Suffix
            } else if rng::flip_coin() {
                AffixType::Prefix
            } else {
                AffixType::Suffix
            }
        }
    };

    if let Some(affix) = roll_affix(
        base,
        modifiers.level,
        affix_type,
        &mut modifiers.get_families(),
        affixes_table,
    ) {
        modifiers.affixes.push(affix);
    } else {
        return false;
    }

    let affixes_amount = prefixes_amount + suffixes_amount + 1;
    let new_rarity = if affixes_amount <= 2 {
        ItemRarity::Magic
    } else if affixes_amount <= 4 {
        ItemRarity::Rare
    } else {
        ItemRarity::Masterwork
    };

    match modifiers.rarity {
        ItemRarity::Normal | ItemRarity::Magic => {
            modifiers.name = generate_name(
                base,
                new_rarity,
                &modifiers.affixes,
                adjectives_table,
                nouns_table,
            );
        }
        _ => {}
    };

    modifiers.rarity = new_rarity;

    true
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
                .map(|r| !r.is_disjoint(&base_item.categories))
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
            item_level: a.item_level,
            effects: a.effects.iter().map(roll_affix_effect).collect(),
        }
    })
}

fn roll_affix_effect(effect_blueprint: &AffixEffectBlueprint) -> AffixEffect {
    AffixEffect {
        stat_effect: StatEffect {
            stat: effect_blueprint.stat.clone(),
            modifier: effect_blueprint.modifier,
            value: effect_blueprint.value.roll().round(),
            bypass_ignore: false,
            // ignore_quality: effect_blueprint.ignore_quality,
        },
        scope: effect_blueprint.scope,
    }
}

pub fn generate_name(
    base: &ItemBase,
    rarity: ItemRarity,
    affixes: &[ItemAffix],
    adjectives_table: &ItemAdjectivesTable,
    nouns_table: &ItemNounsTable,
) -> String {
    match rarity {
        ItemRarity::Magic => generate_magic_name(base, affixes),
        ItemRarity::Rare => generate_rare_name(base, affixes, adjectives_table, nouns_table),
        _ => base.name.clone(),
    }
}

fn generate_magic_name(base: &ItemBase, affixes: &[ItemAffix]) -> String {
    let mut name = base.name.clone();

    let prefixes: Vec<_> = affixes
        .iter()
        .filter(|a| a.affix_type == AffixType::Prefix)
        .collect();

    if prefixes.len() == 1 {
        name = format!("{} {}", prefixes[0].name, name);
    }

    let suffixes: Vec<_> = affixes
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

impl rng::RandomWeighted for WeightedNamePart<'_> {
    fn random_weight(&self) -> u64 {
        self.weight
    }
}

fn generate_rare_name(
    base: &ItemBase,
    affixes: &[ItemAffix],
    adjectives_table: &ItemAdjectivesTable,
    nouns_table: &ItemNounsTable,
) -> String {
    let tags: HashSet<_> = affixes.iter().flat_map(|a| a.tags.iter()).collect();

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
                .filter(|t| base.categories.contains(t))
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
