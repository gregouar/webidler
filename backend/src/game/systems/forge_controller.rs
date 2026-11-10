use shared::data::{
    forge::{self, MAX_TEMPERED_ITEM_QUALITY},
    item::{AffixRoll, ItemCategory, ItemModifiers, ItemRarity, ItemSpecs},
    item_affix::AffixType,
};

use crate::{
    game::{
        data::items_store::ItemsStore,
        systems::{items_controller, loot_generator},
        utils::rng,
    },
    rest::AppError,
};

pub fn upgrade_item(
    items_store: &ItemsStore,
    item_specs: &ItemSpecs,
) -> Result<ItemSpecs, AppError> {
    let item_base = items_store.get_item_base(item_specs)?;
    let available_upgrade_levels = item_base
        .upgrade_levels
        .iter()
        .filter(|l| **l <= item_specs.modifiers.level)
        .count();

    if available_upgrade_levels <= item_specs.modifiers.upgrade_level as usize {
        return Err(AppError::UserError("maximum empower level reached.".into()));
    }

    Ok(items_controller::create_item_specs(
        items_store,
        item_base,
        ItemModifiers {
            upgrade_level: item_specs.modifiers.upgrade_level.saturating_add(1),
            ..item_specs.modifiers.clone()
        },
        0.0,
    ))
}

pub fn temper_item(
    items_store: &ItemsStore,
    item_specs: &ItemSpecs,
) -> Result<ItemSpecs, AppError> {
    let item_base = items_store.get_item_base(item_specs)?;
    if item_base.ignore_quality {
        return Err(AppError::UserError("that item cannot have quality".into()));
    }
    if item_specs.modifiers.quality >= MAX_TEMPERED_ITEM_QUALITY {
        return Err(AppError::UserError(
            "maximum tempered quality reached for that item".into(),
        ));
    }

    let quality_increase = rng::random_range(1_u8..=5).unwrap_or(1);

    let quality =
        (item_specs.modifiers.quality + f32::from(quality_increase)).min(MAX_TEMPERED_ITEM_QUALITY);

    Ok(items_controller::create_item_specs(
        items_store,
        item_base,
        ItemModifiers {
            quality,
            ..item_specs.modifiers.clone()
        },
        item_specs.gold_price,
    ))
}

pub fn reroll_item(
    items_store: &ItemsStore,
    item_specs: &ItemSpecs,
) -> Result<ItemSpecs, AppError> {
    if !forge::can_reroll(item_specs) {
        return Err(AppError::UserError("invalid item".into()));
    }

    let item_base = items_store.get_item_base(item_specs)?;
    let mut modifiers = item_specs.modifiers.clone();

    modifiers.affixes = loot_generator::roll_base_affixes(&modifiers.base_item_id, item_base)
        .into_iter()
        .chain(modifiers.affixes.into_iter().filter(|affix| {
            matches!(
                affix.affix_type,
                AffixType::Prefix | AffixType::Suffix | AffixType::Soul
            )
        }))
        .map(|affix| AffixRoll {
            value: loot_generator::roll_affix_value(),
            ..affix
        })
        .collect();

    Ok(items_controller::create_item_specs(
        items_store,
        item_base,
        modifiers,
        item_specs.gold_price,
    ))
}

pub fn can_sacrifice(item: &ItemSpecs) -> bool {
    if item.categories.contains(&ItemCategory::Soul) {
        return false;
    }

    if item.categories.contains(&ItemCategory::Remnant) {
        return true;
    }

    item.modifiers.rarity == ItemRarity::Rare && item.modifiers.count_presuffixes() == 4
}

pub fn can_imbue(item: &ItemSpecs) -> bool {
    matches!(
        item.modifiers.rarity,
        ItemRarity::Rare | ItemRarity::Masterwork
    ) && !item.categories.contains(&ItemCategory::Soul)
        && !item.categories.contains(&ItemCategory::Remnant)
}

pub fn is_compatible_soul(items_store: &ItemsStore, item: &ItemSpecs, soul: &ItemSpecs) -> bool {
    can_imbue(item)
        && soul.categories.contains(&ItemCategory::Soul)
        && soul.categories.is_superset(&item.categories)
        && soul.affixes.len() == 1
        && soul.affixes.first().is_some_and(|soul_affix| {
            soul_affix.base_item_id.is_some()
                || !item.affixes.iter().any(|affix| {
                    let affix_family = items_store
                        .affixes_table
                        .get(&affix.affix_id)
                        .map(|a| &a.family);
                    let soul_affix_family = items_store
                        .affixes_table
                        .get(&soul_affix.affix_id)
                        .map(|a| &a.family);

                    affix.affix_type != AffixType::Soul && soul_affix_family == affix_family
                })
        })
    // .is_some_and(|affix| item.can_replace_soul_affix(affix))
}

pub fn sacrifice(items_store: &ItemsStore, item: &ItemSpecs) -> Result<ItemSpecs, AppError> {
    if !can_sacrifice(item) {
        return Err(AppError::UserError(
            "only Remnants or Rare items with four affixes can be sacrificed".to_string(),
        ));
    }

    let extractable_affixes: Vec<_> = item
        .modifiers
        .affixes
        .iter()
        .filter(|roll| {
            matches!(
                roll.affix_type,
                AffixType::Prefix | AffixType::Suffix | AffixType::Soul
            )
        })
        .collect();
    let affix_index = rng::random_range(0..extractable_affixes.len()).ok_or(
        AppError::UserError("failed to select an affix for the Soul".to_string()),
    )?;
    let affix_roll = extractable_affixes[affix_index];
    let affix = item
        .affixes
        .iter()
        .find(|affix| {
            affix.base_item_id == affix_roll.base_item_id && affix.affix_id == affix_roll.affix_id
        })
        .ok_or(AppError::NotFound)?;

    let base_item = affix_roll
        .base_item_id
        .as_ref()
        .and_then(|base_item_id| items_store.items.get(base_item_id));

    let item_affix = affix_roll
        .base_item_id
        .is_none()
        .then(|| items_store.affixes_table.get(&affix_roll.affix_id))
        .flatten();

    let mut soul_categories = item.categories.clone();
    soul_categories.remove(&ItemCategory::Remnant);
    let soul_item_id =
        items_store
            .soul_by_categories
            .get(&soul_categories)
            .ok_or(AppError::UserError(
                "no Soul base exists for this item category".to_string(),
            ))?;

    let is_unique = base_item.is_some()
        || item_affix
            .map(|item_affix| item_affix.affix_type == AffixType::Soul)
            .unwrap_or_default();

    let soul_item = items_controller::init_item_specs_from_store(
        items_store,
        ItemModifiers {
            base_item_id: soul_item_id.clone(),
            name: format!(
                "{} Soul",
                if is_unique {
                    item_affix
                        .map(|item_affix| &item_affix.name)
                        .unwrap_or(&item.modifiers.name)
                } else {
                    &item.modifiers.name
                }
            ),
            rarity: if is_unique {
                ItemRarity::Unique
            } else {
                ItemRarity::Masterwork
            },
            level: affix.item_level,
            affixes: vec![AffixRoll {
                affix_type: AffixType::Soul,
                ..affix_roll.clone()
            }],
            quality: 0.0,
            upgrade_level: 0,
        },
    )
    .ok_or(AppError::NotFound)?;

    Ok(soul_item)
}

pub fn imbue(
    items_store: &ItemsStore,
    item_specs: &ItemSpecs,
    soul_specs: &ItemSpecs,
) -> Result<ItemSpecs, AppError> {
    if !is_compatible_soul(items_store, item_specs, soul_specs) {
        return Err(AppError::UserError(
            "Item and Soul are not compatible".to_string(),
        ));
    }

    let item_base = items_store.get_item_base(item_specs)?;

    let extracted_affix = soul_specs
        .modifiers
        .affixes
        .first()
        .ok_or(AppError::NotFound)?;

    let mut item_modifiers = item_specs.modifiers.clone();
    item_modifiers
        .affixes
        .retain(|roll| roll.affix_type != AffixType::Soul);
    item_modifiers.affixes.push(extracted_affix.clone());
    item_modifiers.rarity = ItemRarity::Masterwork;

    Ok(items_controller::create_item_specs(
        items_store,
        item_base,
        item_modifiers,
        0.0,
    ))
}
