use shared::data::world::AreaLevel;

pub const MONSTER_INCREASE_FACTOR: f64 = 0.125;
pub const XP_INCREASE_FACTOR: f64 = 0.4;
pub const ARMOR_FACTOR: f64 = 100.0;

pub fn exponential(level: AreaLevel, factor: f64) -> f64 {
    10f64.powf((level - 1) as f64 * factor)
}

// for armor physical damage decrease
pub fn diminishing(amount: f64, factor: f64) -> f64 {
    if amount < 0.0 {
        return 0.0;
    }
    amount / (amount + factor)
}
