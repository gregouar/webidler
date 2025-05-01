pub fn exponential_factor(level: f64) -> f64 {
    10f64.powf((level - 1.0).powf(1.5) / 10.0)
}

pub fn linear_factor(level: f64) -> f64 {
    1.0 + (level - 1.0) / 2.0
}
