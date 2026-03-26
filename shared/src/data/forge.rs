use crate::{data::item_affix::AffixType, http::client::ForgeAffixOperation};

pub const MAX_AFFIXES: usize = 5;
pub const PREFIX_PRICE_FACTOR: f64 = 2.0;
pub const SUFFIX_PRICE_FACTOR: f64 = 2.0;

pub fn affix_price(amount: usize) -> Option<f64> {
    match amount {
        0 => Some(1.0),
        1 => Some(3.0),
        2 => Some(5.0),
        3 => Some(10.0),
        4 => Some(50.0),
        _ => None,
    }
}

pub fn remove_price(amount: usize) -> Option<f64> {
    if amount > 0 { Some(10.0) } else { None }
}

pub fn affix_operation_price(operation: ForgeAffixOperation, affixes_amount: usize) -> Option<f64> {
    match operation {
        ForgeAffixOperation::Add(affix_type) => affix_price(affixes_amount).map(|price| {
            price
                * match affix_type {
                    Some(AffixType::Prefix) => PREFIX_PRICE_FACTOR,
                    Some(AffixType::Suffix) => SUFFIX_PRICE_FACTOR,
                    _ => 1.0,
                }
        }),
        ForgeAffixOperation::Remove => (affixes_amount > 0).then_some(10.0),
    }
}
