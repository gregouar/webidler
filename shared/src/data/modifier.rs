use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;

use crate::data::temple::StatEffect;

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default,
)]
pub enum Modifier {
    #[default]
    Multiplier, // Would love to rename to Increased
    Flat,
    More,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ModifiableValue<T> {
    pub base: T,

    pub flat: T,
    pub more: f64,
    pub increased: f64,
    pub decreased: f64,
}

impl<T> ModifiableValue<T>
where
    T: std::ops::Add<Output = T> + BaseModifiableValue + Default + Copy,
{
    pub fn evaluate(&self) -> T {
        let div = (1.0 - self.decreased * 0.01).max(0.0);

        let base = self.base + self.flat;
        if base.is_negative() {
            return base;
        }

        let factor = (1.0 + self.more * 0.01)
            * (1.0 + self.increased * 0.01)
            * (if div > 0.0 { 1.0 / div } else { 1.0 });

        base.multiply_value(factor)
    }

    pub fn compute(&mut self) -> T {
        *self = self.evaluate().into();
        self.base
    }

    pub fn apply_effect(&mut self, effect: &StatEffect) {
        // For retro compatibility
        let modifier = match effect.modifier {
            Modifier::Multiplier if effect.stat.is_multiplicative() => Modifier::More,
            modifier => modifier,
        };

        match modifier {
            Modifier::Multiplier => {
                if effect.value >= 0.0 {
                    self.increased += effect.value
                } else {
                    self.decreased += effect.value
                }
            }
            Modifier::Flat => self.flat = self.flat.add_value(effect.value),
            Modifier::More => {
                let value = compute_more_factor(effect.value);
                self.more = self.more + value + self.more * value * 0.01;
            }
        }
    }

    pub fn apply_negative_effect(&mut self, effect: &StatEffect) {
        self.apply_effect(&StatEffect {
            value: -effect.value,
            ..effect.clone()
        })
    }
}

impl<T> Serialize for ModifiableValue<T>
where
    T: std::ops::Add<Output = T> + BaseModifiableValue + Default + Copy,
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.evaluate().serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for ModifiableValue<T>
where
    T: Deserialize<'de> + Default + Copy,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(T::deserialize(deserializer)?.into())
    }
}

impl<T> From<T> for ModifiableValue<T>
where
    T: Default,
{
    fn from(value: T) -> Self {
        Self {
            base: value,
            ..Default::default()
        }
    }
}

impl<T> PartialOrd for ModifiableValue<T>
where
    T: std::ops::Add<Output = T> + BaseModifiableValue + Default + Copy,
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.evaluate().partial_cmp(&other.evaluate())
    }
}

impl<T> PartialEq for ModifiableValue<T>
where
    T: std::ops::Add<Output = T> + BaseModifiableValue + Default + Copy,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.evaluate() == other.evaluate()
    }
}

pub trait BaseModifiableValue {
    fn multiply_value(&self, value: f64) -> Self;
    fn add_value(&self, value: f64) -> Self;
    fn is_negative(&self) -> bool;
}

impl BaseModifiableValue for f64 {
    fn multiply_value(&self, value: f64) -> f64 {
        self * value
    }

    fn add_value(&self, value: f64) -> f64 {
        self + value
    }

    fn is_negative(&self) -> bool {
        *self < 0.0
    }
}

impl BaseModifiableValue for f32 {
    fn multiply_value(&self, value: f64) -> f32 {
        self * value as f32
    }

    fn add_value(&self, value: f64) -> f32 {
        self + value as f32
    }

    fn is_negative(&self) -> bool {
        *self < 0.0
    }
}

pub fn compute_more_factor(value: f64) -> f64 {
    if value >= 0.0 {
        value
    } else {
        // We want that negative effect are diminishingly interesting
        let div = (1.0 - value * 0.01).max(0.0);

        if value <= -1e300 {
            -100.0
        } else if div != 0.0 {
            value / div
        } else {
            0.0
        }
    }
}
