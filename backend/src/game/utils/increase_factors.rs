use shared::data::area::AreaLevel;

pub fn exponential(level: AreaLevel, factor: f64) -> f64 {
    10f64.powf(level.saturating_sub(1) as f64 * factor)
}

// for armor physical damage decrease
pub fn diminishing(amount: f64, factor: f64) -> f64 {
    if amount < 0.0 {
        return 0.0;
    }
    amount / (amount + factor)
}
