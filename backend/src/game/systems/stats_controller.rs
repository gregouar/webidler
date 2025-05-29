use shared::data::stat_effect::{Modifier, StatEffect};

pub trait ApplyStatModifier {
    fn apply_modifier(&mut self, modifier: Modifier, value: f64);
    /// Use for applying decreased cooldown from increased speed
    fn apply_inverse_modifier(&mut self, modifier: Modifier, value: f64) {
        self.apply_modifier(
            modifier,
            match modifier {
                Modifier::Flat => -value,
                Modifier::Multiplier => {
                    if value != -1.0 {
                        -(value / (1.0 + value))
                    } else {
                        0.0
                    }
                }
            },
        )
    }
    fn apply_effect(&mut self, effect: &StatEffect) {
        self.apply_modifier(effect.modifier, effect.value);
    }
    fn apply_inverse_effect(&mut self, effect: &StatEffect) {
        self.apply_inverse_modifier(effect.modifier, effect.value);
    }
}

impl ApplyStatModifier for f32 {
    fn apply_modifier(&mut self, modifier: Modifier, value: f64) {
        match modifier {
            Modifier::Flat => *self += value as f32,
            Modifier::Multiplier => *self *= 1.0 + value as f32,
        }
    }
}

impl ApplyStatModifier for f64 {
    fn apply_modifier(&mut self, modifier: Modifier, value: f64) {
        match modifier {
            Modifier::Flat => *self += value,
            Modifier::Multiplier => *self *= 1.0 + value,
        }
    }
}

// pub fn apply_modifier(stat: &mut f32, modifier: EffectModifier, value: f64) {
//     match modifier {
//         EffectModifier::Flat => {
//             *stat += value as f32;
//         }
//         EffectModifier::Multiplier => {
//             *stat *= 1.0 + value as f32;
//         }
//     }
// }

// pub fn apply_modifier(stat: &mut f64, modifier: EffectModifier, value: f64) {
//     match modifier {
//         EffectModifier::Flat => {
//             *stat += value;
//         }
//         EffectModifier::Multiplier => {
//             *stat *= 1.0 + value;
//         }
//     }
// }
