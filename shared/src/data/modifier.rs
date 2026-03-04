use serde::{Deserialize, Serialize};
use std::ops::Deref;
#[cfg(feature = "modifiable")]
use std::ops::DerefMut;

#[cfg(feature = "modifiable")]
use {
    crate::data::stat_effect::StatEffect,
    serde::{Deserializer, Serializer},
    std::cmp::Ordering,
};

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default,
)]
pub enum Modifier {
    #[default]
    Increased,
    Flat,
    More,
}

#[cfg(not(feature = "modifiable"))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
#[serde(transparent)]
pub struct ModifiableValue<T>(T);

#[cfg(not(feature = "modifiable"))]
impl<T> From<T> for ModifiableValue<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[cfg(not(feature = "modifiable"))]
impl<T> Deref for ModifiableValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(not(feature = "modifiable"))]
impl<T> AsRef<T> for ModifiableValue<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

#[cfg(feature = "modifiable")]
#[derive(Debug, Clone, Copy, Default)]
pub struct ModifiableValue<T> {
    more: f64,
    increased: f64,
    decreased: f64,

    converted: f64,

    base: T,
    evaluated: T,
}

#[cfg(feature = "modifiable")]
impl<T> ModifiableValue<T>
where
    T: BaseModifiableValue + Default + Copy,
{
    fn evaluate(&self, convert: bool) -> T {
        let div = (1.0 - self.decreased * 0.01).max(0.0);
        let base = if convert {
            self.base.multiply_value(1.0 - self.converted * 0.01)
        } else {
            self.base
        };

        if base.is_negative() {
            return base;
        }

        let factor = (1.0 + self.more * 0.01)
            * (1.0 + self.increased * 0.01)
            * (if div > 0.0 { 1.0 / div } else { 1.0 });

        base.multiply_value(factor)
    }

    fn compute(&mut self) {
        self.evaluated = self.evaluate(true);
    }

    pub fn apply_modifier(&mut self, value: f64, modifier: Modifier) {
        match modifier {
            Modifier::Increased => {
                if value >= 0.0 {
                    self.increased += value
                } else {
                    self.decreased += value
                }
            }
            Modifier::Flat => self.base = self.base.add_value(value),
            Modifier::More => {
                let value = compute_more_factor(value);
                self.more = self.more + value + self.more * value * 0.01;
            }
        }
        self.compute();
    }

    pub fn apply_effect(&mut self, effect: &StatEffect) {
        self.apply_modifier(effect.value, effect.modifier);
    }

    pub fn apply_negative_effect(&mut self, effect: &StatEffect) {
        self.apply_modifier(-effect.value, effect.modifier);
    }

    pub fn convert_value(&mut self, percent: f64, is_extra: bool, only_base: bool) -> T {
        let mut percent = percent;
        if !is_extra {
            percent = percent.min(100.0 - self.converted);
            self.converted += percent;
        }

        self.compute();

        (if only_base {
            self.base
        } else {
            self.evaluate(false)
        })
        .multiply_value(percent * 0.01)
    }
}

#[cfg(feature = "modifiable")]
impl<T> Deref for ModifiableValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.evaluated
    }
}
#[cfg(feature = "modifiable")]
impl<T> DerefMut for ModifiableValue<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.evaluated
    }
}

#[cfg(feature = "modifiable")]
impl<T> AsRef<T> for ModifiableValue<T> {
    fn as_ref(&self) -> &T {
        &self.evaluated
    }
}

#[cfg(feature = "modifiable")]
impl<T> Serialize for ModifiableValue<T>
where
    T: Serialize + Copy,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.evaluated.serialize(serializer)
    }
}

#[cfg(feature = "modifiable")]
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

#[cfg(feature = "modifiable")]
impl<T> From<T> for ModifiableValue<T>
where
    T: Default + Copy,
{
    fn from(value: T) -> Self {
        Self {
            base: value,
            evaluated: value,
            ..Default::default()
        }
    }
}

#[cfg(feature = "modifiable")]
impl<T> PartialOrd for ModifiableValue<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.evaluated.partial_cmp(&other.evaluated)
    }
}

#[cfg(feature = "modifiable")]
impl<T> PartialEq for ModifiableValue<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.evaluated == other.evaluated
    }
}

pub trait BaseModifiableValue {
    fn multiply_value(&self, value: f64) -> Self;
    fn add_value(&self, value: f64) -> Self;
    fn is_negative(&self) -> bool;
    fn round(&self) -> Self;
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

    fn round(&self) -> f64 {
        (*self).round()
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

    fn round(&self) -> f32 {
        (*self).round()
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
