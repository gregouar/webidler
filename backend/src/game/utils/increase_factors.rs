pub fn exponential(level: f64) -> f64 {
    10f64.powf((level - 1.0).powf(1.5) / 10.0)
}

pub fn linear(level: f64) -> f64 {
    1.0 + (level - 1.0) / 2.0
}

// for armor physical damage decrease
pub fn diminishing(amount: f64, factor: f64) -> f64 {
    if amount < 0.0 {
        return 0.0;
    }
    amount / (amount + factor)
}
