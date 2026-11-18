use crate::{
    data::{
        area::AreaLevel,
        item::{ItemBase, ItemCategory, ItemModifiers, ItemRarity, ItemSpecs},
        item_affix::{AffixType, ItemAffix},
        values::NonNegative,
    },
    http::client::ForgeAffixOperation,
};

pub const MAX_AFFIXES: usize = 4;
pub const PREFIX_PRICE_FACTOR: f64 = 2.0;
pub const SUFFIX_PRICE_FACTOR: f64 = 2.0;
pub const MAX_TEMPERED_ITEM_QUALITY: f32 = 50.0;

fn affix_price(amount: usize) -> Option<f64> {
    match amount {
        0 => Some(1.0),
        1 => Some(2.0),
        2 => Some(3.0),
        3 => Some(4.0),
        _ => None,
    }
}

fn remove_price(amount: usize) -> Option<NonNegative> {
    if amount > 0 { Some(5.0.into()) } else { None }
}

pub fn affix_operation_price(
    operation: ForgeAffixOperation,
    affixes_amount: usize,
    item_specs: &ItemSpecs,
) -> Option<NonNegative> {
    if item_specs.categories.contains(&ItemCategory::Soul)
        || item_specs.categories.contains(&ItemCategory::Remnant)
    {
        return None;
    }

    let price_base_factor = 1.0 + item_specs.modifiers.level as f64 * 0.005;

    match operation {
        ForgeAffixOperation::Add(affix_type) => affix_price(affixes_amount).map(|price| {
            price_base_factor
                * price
                * match affix_type {
                    Some(AffixType::Prefix) => PREFIX_PRICE_FACTOR,
                    Some(AffixType::Suffix) => SUFFIX_PRICE_FACTOR,
                    _ => 1.0,
                }
        }),
        ForgeAffixOperation::Remove => {
            remove_price(affixes_amount).map(|price| price_base_factor * price.get())
        }
    }
    .map(|x| x.round().into())
}

pub fn gamble_price(item_level: AreaLevel) -> NonNegative {
    ((item_level as f64 / 10.0).floor() + 10.0).into()
}

pub fn sacrifice_price(affixes: &[ItemAffix]) -> NonNegative {
    (affixes
        .iter()
        .filter(|affix| {
            matches!(
                affix.affix_type,
                AffixType::Prefix | AffixType::Suffix | AffixType::Soul
            )
        })
        .map(|affix| affix.item_level as f64)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or_default()
        / 5.0)
        .round()
        .into()
}

pub fn can_reroll(item: &ItemSpecs) -> bool {
    item.modifiers.rarity != ItemRarity::Normal
        && !item.categories.contains(&ItemCategory::Soul)
        && !item.categories.contains(&ItemCategory::Remnant)
}

pub fn reroll_price(affixes: &[ItemAffix]) -> NonNegative {
    (affixes
        .iter()
        .map(|affix| affix.item_level as f64)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or_default()
        / 20.0)
        .round()
        .into()
}

pub fn upgrade_item_price(
    item_base: &ItemBase,
    item_modifiers: &ItemModifiers,
) -> Option<NonNegative> {
    item_base
        .upgrade_levels
        .get(item_modifiers.upgrade_level as usize)
        .and_then(|next_upgrade_level| {
            (*next_upgrade_level <= item_modifiers.level)
                .then_some((*next_upgrade_level / 2 + item_base.min_area_level) as f64)
                .map(Into::into)
        })
}

pub fn temper_item_price(
    item_base: &ItemBase,
    item_modifiers: &ItemModifiers,
) -> Option<NonNegative> {
    let quality = item_modifiers.quality.clamp(0.0, MAX_TEMPERED_ITEM_QUALITY);
    (!item_base.ignore_quality && quality < MAX_TEMPERED_ITEM_QUALITY)
        .then(|| ((f64::from(quality) / 5.0).floor() + 1.0).powi(2))
        .map(Into::into)
}
