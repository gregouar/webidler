use shared::data::{
    modifier::Modifier,
    skill::ApplyStatusEffect,
    stat_effect::{EffectsMap, Matchable, StatEffect, StatType},
};

pub fn compute_stats_effects_status_value(
    effects_map: &EffectsMap,
    status_effect: &ApplyStatusEffect,
) -> f64 {
    let mut factor = Factor::new();

    let status_power = StatType::StatusPower {
        status_type: Some((&status_effect.status_type).into()),
        skill_filter: Default::default(), // TODO
        min_max: None,
    };

    for effect in effects_map.iter() {
        if effect.stat.is_match(&status_power) {
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
                    self.decreased -= stat_effect.value;
                }
            }
            Modifier::Flat => {}
            Modifier::More => {
                self.more *= stat_effect.value;
            }
        }
    }
}
