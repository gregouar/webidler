use std::collections::HashSet;

use shared::data::{
    character_status::StatusId,
    modifier::{Modifier, compute_more_factor},
    skill::DamageType,
    stat_effect::{EffectsMap, Matchable, StatEffect, StatType, compare_options},
};

pub fn compute_stats_effects_status_value(
    effects_map: &EffectsMap,
    ignore_stat_effects: &HashSet<StatType>,
    status_id: &StatusId,
    status_damage_type: Option<DamageType>,
) -> f64 {
    let mut factor = Factor::new();

    for effect in effects_map.iter() {
        if ignore_stat_effects
            .iter()
            .any(|ignored_stat_effect| ignored_stat_effect.is_match(&effect.stat))
        {
            continue;
        }

        if let StatType::StatusPower {
            status_filter,
            skill_filter: _,
            min_max: _,
        } = &effect.stat
            && status_filter.is_match_with_status(status_id, status_damage_type)
        // && skill_filter.is_match_with_skill() TODO
        {
            factor.apply_effect(&effect);
        }

        if let StatType::Damage {
            skill_filter: _,
            damage_type,
            min_max: _,
            is_hit,
        } = &effect.stat
            && compare_options(is_hit, &Some(false))
            && status_damage_type.is_some()
            && compare_options(&status_damage_type, damage_type)
        {
            factor.apply_effect(&effect);
        }
    }

    factor.evaluate()
}

struct Factor {
    // base: f64,
    more: f64,
    increased: f64,
    decreased: f64,
}

impl Factor {
    fn new() -> Self {
        Self {
            // base: 1.0,
            more: 0.0,
            increased: 0.0,
            decreased: 0.0,
        }
    }

    fn evaluate(self) -> f64 {
        let div = (1.0 - self.decreased * 0.01).max(0.0);
        // let base = if convert {
        //     self.base.multiply_value(1.0 - self.converted * 0.01)
        // } else {
        //     self.base
        // };

        // if base.is_negative() {
        //     return base;
        // }

        // if self.more == -100.0 {
        //     return base.multiply_value(0.0);
        // }

        // base.multiply_value(factor)

        (1.0 + self.more * 0.01)
            * (1.0 + self.increased * 0.01)
            * (if div > 0.0 { 1.0 / div } else { 1.0 })
    }

    fn apply_effect(&mut self, stat_effect: &StatEffect) {
        match stat_effect.modifier {
            Modifier::Increased => {
                if stat_effect.value >= 0.0 {
                    self.increased += stat_effect.value;
                } else {
                    self.decreased += stat_effect.value;
                }
            }
            Modifier::Flat => {}
            Modifier::More => {
                let value = compute_more_factor(stat_effect.value);
                if value == -100.0 || self.more == -100.0 {
                    self.more = -100.0
                } else {
                    self.more = self.more + value + self.more * value * 0.01;
                }
            }
        }
    }
}
