use crate::{
    constants::{SKILL_COST_INCREASE_FACTOR, XP_INCREASE_FACTOR},
    data::{area::AreaLevel, player::PlayerSpecs, skill::SkillSpecs},
};

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

pub fn skill_cost_increase(skill_specs: &SkillSpecs) -> f64 {
    skill_specs.next_upgrade_cost
        + (10.0 * exponential(skill_specs.upgrade_level, SKILL_COST_INCREASE_FACTOR)).round()
}

pub fn player_level_up_cost(player_specs: &PlayerSpecs) -> f64 {
    (20.0 * exponential(player_specs.level as AreaLevel, XP_INCREASE_FACTOR)).round()
}
