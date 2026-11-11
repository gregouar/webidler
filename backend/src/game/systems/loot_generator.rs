use std::collections::HashSet;

use shared::{
    computations,
    constants::{MAX_ITEM_QUALITY, MAX_ITEM_QUALITY_PER_LEVEL},
    data::{
        area::AreaLevel,
        chance::ChanceRange,
        forge::MAX_AFFIXES,
        item::{
            AffixRoll, BaseItemId, ItemBase, ItemCategory, ItemModifiers, ItemRarity, ItemSpecs,
        },
        item_affix::AffixType,
    },
};

use crate::game::{
    data::{
        items_store::{ItemAffixesTable, ItemsStore},
        loot_table::{LootTable, LootTableEntry, RarityWeights},
    },
    utils::rng::{self, RandomWeighted, Rollable, flip_coin},
};

use super::items_controller;

pub struct GenerateLootTemplate {
    pub allow_unique: bool,
    pub max_base: bool,
    pub max_affixes: bool,
    pub filter_category: Option<ItemCategory>,
    pub prevent_categories: &'static [ItemCategory],
}

pub const DEFAULT_LOOT_TEMPLATE: GenerateLootTemplate = GenerateLootTemplate {
    allow_unique: true,
    max_base: false,
    max_affixes: false,
    filter_category: None,
    prevent_categories: &[],
};

pub const REPEATED_LOOT_TEMPLATE: GenerateLootTemplate = GenerateLootTemplate {
    allow_unique: false,
    max_base: false,
    max_affixes: false,
    filter_category: None,
    prevent_categories: &[ItemCategory::Rune, ItemCategory::Remnant],
};

pub const MAP_LOOT_TEMPLATE: GenerateLootTemplate = GenerateLootTemplate {
    allow_unique: true,
    max_base: false,
    max_affixes: false,
    filter_category: Some(ItemCategory::Map),
    prevent_categories: &[],
};

#[allow(clippy::too_many_arguments)]
pub fn generate_loot(
    loot_table: &LootTable,
    items_store: &ItemsStore,
    loot_template: &GenerateLootTemplate,
    level: AreaLevel,
    power_level_modifier: AreaLevel,
    is_boss_level: bool,
    loot_rarity: f64,
    gold_find: f64,
    prevent_roll: Option<&HashSet<BaseItemId>>,
) -> Option<ItemSpecs> {
    let mut rarity = roll_rarity(&RarityWeights::default(), loot_rarity);
    if !loot_template.allow_unique {
        rarity = rarity.min(ItemRarity::Rare);
    }
    roll_item(
        loot_table,
        items_store,
        loot_template,
        level,
        power_level_modifier,
        is_boss_level,
        rarity,
        gold_find,
        prevent_roll,
    )
}

fn roll_rarity(weights: &RarityWeights, loot_rarity: f64) -> ItemRarity {
    let factor = loot_rarity * 0.01;

    match rng::random_range(0.0..(weights.normal + weights.magic + weights.rare + weights.unique))
        .unwrap_or(0.0)
    {
        r if r < (weights.unique) * factor => ItemRarity::Unique,
        r if r < (weights.unique + weights.rare) * factor => ItemRarity::Rare,
        r if r < (weights.unique + weights.rare + weights.magic) * factor => ItemRarity::Magic,
        _ => ItemRarity::Normal,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn roll_item(
    loot_table: &LootTable,
    items_store: &ItemsStore,
    loot_template: &GenerateLootTemplate,
    level: AreaLevel,
    power_level_modifier: AreaLevel,
    is_boss_level: bool,
    rarity: ItemRarity,
    gold_find: f64,
    prevent_roll: Option<&HashSet<BaseItemId>>,
) -> Option<ItemSpecs> {
    roll_base_item(
        loot_table,
        items_store,
        loot_template,
        level,
        is_boss_level,
        rarity == ItemRarity::Unique,
        prevent_roll,
    )
    .map(|(base_item_id, base)| {
        let rarity = if base.rarity != ItemRarity::Unique {
            rarity.clamp(base.rarity, ItemRarity::Masterwork)
        } else {
            ItemRarity::Unique
        };
        roll_item_stats(
            items_store,
            base_item_id,
            base,
            rarity,
            level,
            power_level_modifier,
            loot_template.max_affixes,
            gold_find,
            // &items_store.signature_key,
        )
    })
}

#[allow(clippy::too_many_arguments)]
pub fn roll_item_stats(
    items_store: &ItemsStore,
    base_item_id: BaseItemId,
    item_base: &ItemBase,
    rarity: ItemRarity,
    level: AreaLevel,
    power_level_modifier: AreaLevel,
    max_affixes: bool,
    gold_find: f64,
    // signature_key: &HmacSignature,
) -> ItemSpecs {
    let quality = if item_base.ignore_quality {
        0.0
    } else {
        roll_quality(item_base.min_area_level, level)
    };

    let affixes = roll_base_affixes(&base_item_id, item_base);
    let mut modifiers = ItemModifiers {
        base_item_id,
        name: item_base.name.clone(),
        rarity: match rarity {
            ItemRarity::Unique => ItemRarity::Unique,
            _ => ItemRarity::Normal,
        },
        level,
        quality,
        affixes,
        upgrade_level: 0,
    };

    let affixes_amount = if max_affixes {
        match rarity {
            ItemRarity::Magic => 2,
            ItemRarity::Rare | ItemRarity::Masterwork => 4,
            _ => 0,
        }
    } else {
        match rarity {
            ItemRarity::Magic => ChanceRange {
                min: 1,
                max: 2,
                ..Default::default()
            },
            ItemRarity::Rare | ItemRarity::Masterwork => ChanceRange {
                min: 3,
                max: 4,
                ..Default::default()
            },
            _ => ChanceRange::default(),
        }
        .roll()
    };

    for _ in 0..affixes_amount {
        add_affix(items_store, item_base, &mut modifiers, None);
    }

    if rarity == ItemRarity::Masterwork {
        add_affix(
            items_store,
            item_base,
            &mut modifiers,
            Some(AffixType::Soul),
        );
    }

    let gold_price = computations::item_gold_price(
        modifiers.level.saturating_sub(power_level_modifier),
        modifiers.rarity,
    ) * gold_find
        * 0.01;

    items_controller::create_item_specs(items_store, item_base, modifiers, gold_price)
}

fn roll_quality(min_item_level: AreaLevel, level: AreaLevel) -> f32 {
    (rng::random_range(0..=level.saturating_sub(min_item_level)).unwrap_or_default() as f32
        * MAX_ITEM_QUALITY_PER_LEVEL)
        .min(MAX_ITEM_QUALITY)
        .round()
}

fn roll_base_item<'a>(
    loot_table: &LootTable,
    items_store: &'a ItemsStore,
    loot_template: &GenerateLootTemplate,
    area_level: AreaLevel,
    is_boss_level: bool,
    is_unique: bool,
    prevent_roll: Option<&HashSet<BaseItemId>>,
) -> Option<(BaseItemId, &'a ItemBase)> {
    let items_available: Vec<_> = loot_table
        .entries
        .iter()
        .filter(|l| {
            let Some(item_specs) = items_store.items.get(&l.item_id) else {
                return false;
            };
            area_level >= l.min_area_level.unwrap_or(item_specs.min_area_level)
                && area_level <= l.max_area_level.unwrap_or(AreaLevel::MAX)
                && (!l.boss_only || is_boss_level)
                && (loot_template
                    .filter_category
                    .map(|category| item_specs.categories.contains(&category))
                    .unwrap_or(true))
                && loot_template
                    .prevent_categories
                    .iter()
                    .all(|category| !item_specs.categories.contains(category))
                && !prevent_roll.is_some_and(|prevent_roll| prevent_roll.contains(&l.item_id))
        })
        .collect();

    let items_available: Vec<_> = if is_unique {
        let uniques_available: Vec<_> = items_available
            .iter()
            .filter(|l| {
                items_store
                    .items
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
                    .items
                    .get(&l.item_id)
                    .map(|base| base.rarity != ItemRarity::Unique)
                    .unwrap_or_default()
            })
            .collect()
    };

    let items_available = if loot_template.max_base && !is_unique {
        let max_level = items_available
            .iter()
            .map(|l| {
                items_store
                    .items
                    .get(&l.item_id)
                    .filter(|base| !base.categories.contains(&ItemCategory::Remnant))
                    .map(|base| base.min_area_level)
                    .unwrap_or_default()
            })
            .max()
            .unwrap_or_default();
        items_available
            .into_iter()
            .filter(|l| {
                items_store
                    .items
                    .get(&l.item_id)
                    .map(|base| base.min_area_level >= max_level)
                    .unwrap_or_default()
            })
            .collect()
    } else {
        items_available
    };

    if items_available.is_empty() {
        tracing::warn!("No base items available for level {}", area_level);
    }

    rng::random_weighted_pick(&items_available).and_then(|loot_entry| {
        items_store
            .items
            .get(&loot_entry.item_id)
            .map(|item_base| (loot_entry.item_id.clone(), item_base))
    })
}

pub fn roll_base_affixes(base_item_id: &BaseItemId, base_item: &ItemBase) -> Vec<AffixRoll> {
    base_item
        .affixes
        .iter()
        .map(|(effect_id, effect_blueprint)| AffixRoll {
            affix_type: if base_item.categories.contains(&ItemCategory::Soul) {
                AffixType::Soul
            } else if effect_blueprint.is_unique {
                AffixType::Unique
            } else {
                AffixType::Base
            },
            base_item_id: Some(base_item_id.clone()),
            affix_id: effect_id.clone(),
            value: roll_affix_value(),
        })
        .collect()
}

pub fn add_affix(
    items_store: &ItemsStore,
    base: &ItemBase,
    modifiers: &mut ItemModifiers,
    affix_type: Option<AffixType>,
) -> bool {
    if base.rarity == ItemRarity::Unique || base.categories.contains(&ItemCategory::Soul) {
        return false;
    }

    let prefixes_amount = modifiers.count_affixes(AffixType::Prefix);
    let suffixes_amount = modifiers.count_affixes(AffixType::Suffix);

    if prefixes_amount + suffixes_amount >= MAX_AFFIXES && affix_type != Some(AffixType::Soul) {
        return false;
    }

    let affix_type = match affix_type {
        Some(AffixType::Soul) => AffixType::Soul,
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

    if let Some((affix_id, value)) = roll_affix(
        base,
        modifiers.level,
        affix_type,
        get_families(&items_store.affixes_table, modifiers),
        &items_store.affixes_table,
    ) {
        modifiers.affixes.push(AffixRoll {
            affix_type,
            base_item_id: None,
            affix_id,
            value,
        });
    } else {
        return false;
    }

    let affixes_amount = prefixes_amount + suffixes_amount + 1;
    update_rarity(items_store, base, modifiers, affixes_amount);

    true
}

pub fn remove_affix(
    items_store: &ItemsStore,
    base: &ItemBase,
    modifiers: &mut ItemModifiers,
) -> bool {
    if base.rarity == ItemRarity::Unique {
        return false;
    }

    let prefixes_amount = modifiers.count_affixes(AffixType::Prefix);
    let suffixes_amount = modifiers.count_affixes(AffixType::Suffix);

    if prefixes_amount == 0 && suffixes_amount == 0 {
        return false;
    }

    let affix_type = if prefixes_amount > suffixes_amount {
        AffixType::Prefix
    } else if suffixes_amount > prefixes_amount {
        AffixType::Suffix
    } else if flip_coin() {
        AffixType::Prefix
    } else {
        AffixType::Suffix
    };

    let affixes_amount = match affix_type {
        AffixType::Prefix => prefixes_amount,
        AffixType::Suffix => suffixes_amount,
        AffixType::Base | AffixType::Unique | AffixType::Upgrade | AffixType::Soul => 0,
    };

    let affix_subindex = rng::random_range(0..affixes_amount).unwrap_or_default();

    let affix_index = modifiers
        .affixes
        .iter()
        .enumerate()
        .filter(|(_, affix)| affix.affix_type == affix_type)
        .nth(affix_subindex)
        .map(|(idx, _)| idx)
        .unwrap_or_default();

    modifiers.affixes.remove(affix_index);

    let affixes_amount = prefixes_amount + suffixes_amount - 1;
    update_rarity(items_store, base, modifiers, affixes_amount);

    true
}

fn update_rarity(
    items_store: &ItemsStore,
    base: &ItemBase,
    modifiers: &mut ItemModifiers,
    affixes_amount: usize,
) {
    let new_rarity = if modifiers.count_affixes(AffixType::Soul) > 0 {
        ItemRarity::Masterwork
    } else if affixes_amount > 2 {
        ItemRarity::Rare
    } else if affixes_amount > 0 {
        ItemRarity::Magic
    } else {
        ItemRarity::Normal
    };

    if (new_rarity != modifiers.rarity
        && new_rarity != ItemRarity::Masterwork
        && modifiers.rarity != ItemRarity::Masterwork)
        || new_rarity == ItemRarity::Magic
    {
        modifiers.name = generate_name(items_store, base, new_rarity, &modifiers.affixes);
    }

    modifiers.rarity = new_rarity;
}

struct TweakedItemAffixBlueprint<'a> {
    affix_id: &'a str,
    weight: u64,
}

impl<'a> rng::RandomWeighted for TweakedItemAffixBlueprint<'a> {
    fn random_weight(&self) -> u64 {
        self.weight
    }
}

fn roll_affix(
    base_item: &ItemBase,
    area_level: AreaLevel,
    affix_type: AffixType,
    families_in_use: HashSet<String>,
    affixes_table: &ItemAffixesTable,
) -> Option<(String, u8)> {
    let available_affixes: Vec<_> = affixes_table
        .iter()
        .filter(|(_, a)| {
            !a.restrictions.is_disjoint(&base_item.categories)
                && area_level >= a.item_level
                && a.affix_type == affix_type
                && !families_in_use.contains(&a.family)
        })
        .map(|(affix_id, affix_blueprint)| TweakedItemAffixBlueprint {
            affix_id,
            weight: tweak_affix_weight(
                affix_blueprint.weight,
                affix_blueprint.item_level,
                area_level,
            ),
        })
        .collect();

    rng::random_weighted_pick(&available_affixes)
        .map(|a| (a.affix_id.to_string(), roll_affix_value()))
}

fn tweak_affix_weight(base_weight: u64, tier_level: AreaLevel, area_level: AreaLevel) -> u64 {
    let delta = area_level.saturating_sub(tier_level) as f64;
    let factor = 1.0 + delta * (tier_level as f64).powf(1.5) / 10_000.0;
    (base_weight as f64 * factor) as u64
}

pub fn roll_affix_value() -> u8 {
    rng::random_range(0..=100).unwrap_or(50) as u8
}

fn get_families(affixes_table: &ItemAffixesTable, modifiers: &ItemModifiers) -> HashSet<String> {
    modifiers
        .affixes
        .iter()
        .filter(|affix| affix.base_item_id.is_none())
        .filter_map(|affix| affixes_table.get(&affix.affix_id))
        .map(|affix| affix.family.clone())
        .collect()
}

pub fn generate_name(
    items_store: &ItemsStore,
    base: &ItemBase,
    rarity: ItemRarity,
    affixes: &[AffixRoll],
) -> String {
    match rarity {
        ItemRarity::Magic => generate_magic_name(items_store, base, affixes),
        ItemRarity::Rare => generate_rare_name(items_store, base, affixes),
        _ => base.name.clone(),
    }
}

fn generate_magic_name(items_store: &ItemsStore, base: &ItemBase, affixes: &[AffixRoll]) -> String {
    let mut name = base.name.clone();

    if let Some(prefix_name) = affixes
        .iter()
        .find(|a| a.affix_type == AffixType::Prefix)
        .and_then(|a| items_store.affixes_table.get(&a.affix_id))
        .map(|a| &a.name)
    {
        name = format!("{} {}", prefix_name, name);
    }

    if let Some(suffix_name) = affixes
        .iter()
        .find(|a| a.affix_type == AffixType::Suffix)
        .and_then(|a| items_store.affixes_table.get(&a.affix_id))
        .map(|a| &a.name)
    {
        name = format!("{} {}", name, suffix_name);
    }

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

fn generate_rare_name(items_store: &ItemsStore, base: &ItemBase, affixes: &[AffixRoll]) -> String {
    let tags: HashSet<_> = affixes
        .iter()
        .filter_map(|a| items_store.affixes_table.get(&a.affix_id))
        .flat_map(|a| a.tags.iter().copied())
        .collect();

    let available_adjectives: Vec<_> = items_store
        .adjectives_table
        .iter()
        .filter(|part| part.tags.is_subset(&tags))
        .map(|part| WeightedNamePart {
            text: &part.text,
            weight: part.tags.len() as u64,
        })
        .collect();

    let available_nouns: Vec<_> = items_store
        .nouns_table
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

impl RandomWeighted for &LootTableEntry {
    fn random_weight(&self) -> u64 {
        self.weight
    }
}
