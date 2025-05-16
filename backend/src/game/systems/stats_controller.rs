use shared::data::{effect::EffectModifier, item_affix::AffixEffect};

pub trait ApplyStatModifier {
    fn apply_modifier(&mut self, modifier: EffectModifier, value: f64);
    fn apply_effect(&mut self, effect: &AffixEffect) {
        self.apply_modifier(effect.modifier, effect.value);
    }
}

impl ApplyStatModifier for f32 {
    fn apply_modifier(&mut self, modifier: EffectModifier, value: f64) {
        match modifier {
            EffectModifier::Flat => *self += value as f32,
            EffectModifier::Multiplier => *self *= 1.0 + value as f32,
        }
    }
}

impl ApplyStatModifier for f64 {
    fn apply_modifier(&mut self, modifier: EffectModifier, value: f64) {
        match modifier {
            EffectModifier::Flat => *self += value,
            EffectModifier::Multiplier => *self *= 1.0 + value,
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
