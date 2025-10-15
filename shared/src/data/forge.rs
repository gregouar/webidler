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
