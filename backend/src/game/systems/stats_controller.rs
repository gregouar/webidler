use shared::data::stat_effect::{Modifier, StatEffect};

pub trait ApplyStatModifier {
    fn apply_modifier(&mut self, modifier: Modifier, value: f64);
    fn apply_effect(&mut self, effect: &StatEffect) {
        // We want that negative effect are diminishingly interesting
        let value = match effect.modifier {
            Modifier::Flat => effect.value,
            Modifier::Multiplier => {
                if effect.value >= 0.0 {
                    effect.value
                } else {
                    let div = (1.0 - effect.value).max(0.0);
                    if div != 0.0 {
                        effect.value / div
                    } else {
                        0.0
                    }
                }
            }
        };
        self.apply_modifier(effect.modifier, value);
    }

    fn apply_negative_effect(&mut self, effect: &StatEffect) {
        self.apply_effect(&StatEffect {
            stat: effect.stat,
            modifier: effect.modifier,
            value: -effect.value,
        })
    }
}

impl ApplyStatModifier for f32 {
    fn apply_modifier(&mut self, modifier: Modifier, value: f64) {
        match modifier {
            Modifier::Flat => *self += value as f32,
            Modifier::Multiplier => *self *= (1.0 + value as f32).max(0.0),
        }
    }
}

impl ApplyStatModifier for f64 {
    fn apply_modifier(&mut self, modifier: Modifier, value: f64) {
        match modifier {
            Modifier::Flat => *self += value,
            Modifier::Multiplier => *self *= (1.0 + value).max(0.0),
        }
    }
}
