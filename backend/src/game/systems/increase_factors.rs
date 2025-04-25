pub fn exponential_factor(level: f64) -> f64 {
    10f64.powf((level - 1.0).powf(1.5) / 10.0)
}
