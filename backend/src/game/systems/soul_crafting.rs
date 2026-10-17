use shared::data::{
    item::{ItemCategory, ItemModifiers, ItemRarity, ItemSpecs},
    item_affix::AffixType,
};

use crate::{
    game::{data::items_store::ItemsStore, systems::items_controller, utils::rng},
    rest::AppError,
};

pub fn can_sacrifice(item: &ItemSpecs) -> bool {
    item.modifiers.rarity == ItemRarity::Rare
        && !item.base.categories.contains(&ItemCategory::Soul)
        && item.modifiers.count_nonunique_affixes() == 4
}

pub fn can_imbue(item: &ItemSpecs) -> bool {
    matches!(
        item.modifiers.rarity,
        ItemRarity::Rare | ItemRarity::Masterwork
    ) && !item.base.categories.contains(&ItemCategory::Soul)
}

pub fn is_compatible_soul(item: &ItemSpecs, soul: &ItemSpecs) -> bool {
    can_imbue(item)
        && soul.base.categories.is_superset(&item.base.categories)
        && soul.modifiers.affixes.len() == 1
        && soul
            .modifiers
            .affixes
            .first()
            .is_some_and(|affix| item.modifiers.can_replace_soul_affix(affix))
}

pub fn sacrifice(item: &ItemSpecs, items_store: &ItemsStore) -> Result<ItemSpecs, AppError> {
    if !can_sacrifice(item) {
        return Err(AppError::UserError(
            "only Rare items with four affixes can be sacrificed".to_string(),
        ));
    }

    let normal_affixes: Vec<_> = item
        .modifiers
        .affixes
        .iter()
        .filter(|affix| matches!(affix.affix_type, AffixType::Prefix | AffixType::Suffix))
        .collect();
    let affix_index = rng::random_range(0..normal_affixes.len()).ok_or(AppError::UserError(
        "failed to select an affix for the Soul".to_string(),
    ))?;
    let mut affix = normal_affixes[affix_index].clone();
    affix.affix_type = AffixType::Soul;

    let soul_item_id = items_store
        .soul_by_categories
        .get(&item.base.categories)
        .ok_or(AppError::UserError(
            "no Soul base exists for this item category".to_string(),
        ))?;
    let soul_item = items_controller::init_item_specs_from_store(
        items_store,
        ItemModifiers {
            base_item_id: soul_item_id.clone(),
            name: format!("{} Soul", item.modifiers.name),
            rarity: ItemRarity::Masterwork,
            level: affix.item_level,
            affixes: vec![affix],
            quality: 0.0,
            upgrade_level: 0,
        },
    )
    .ok_or(AppError::NotFound)?;

    Ok(soul_item)
}

pub fn imbue(item: &mut ItemSpecs, mut soul: ItemSpecs) -> Result<(), AppError> {
    if !is_compatible_soul(item, &soul) {
        return Err(AppError::UserError(
            "Item and Soul are not compatible".to_string(),
        ));
    }

    item.modifiers
        .affixes
        .retain(|affix| affix.affix_type != AffixType::Soul);
    item.modifiers
        .affixes
        .push(soul.modifiers.affixes.pop().ok_or(AppError::NotFound)?);
    item.modifiers.rarity = ItemRarity::Masterwork;

    Ok(())
}
