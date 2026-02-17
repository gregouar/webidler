use std::{cmp::Ordering, collections::HashMap};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use shared::{
    computations::compute_more_factor,
    data::{
        chance::{Chance, ChanceRange},
        skill::DamageType,
        stat_effect::{DamageMap, Modifier, StatEffect},
    },
};

#[derive(Debug, Clone, Copy, Default)]
pub struct ModifiableValue<T> {
    base: T,

    more: f64,
    increased: f64,
    decreased: f64,

    converted: f64,
}

impl<T> ModifiableValue<T>
where
    T: std::ops::Add<Output = T> + BaseModifiableValue + Default + Copy,
{
    pub fn evaluate(&self) -> T {
        let div = (1.0 - self.decreased * 0.01).max(0.0);
        let base = self.base.multiply_value(1.0 - self.converted * 0.01);

        if base.is_negative() {
            return base;
        }

        let factor = (1.0 + self.more * 0.01)
            * (1.0 + self.increased * 0.01)
            * (if div > 0.0 { 1.0 / div } else { 1.0 });

        base.multiply_value(factor)
    }

    // pub fn compute(&mut self) -> T {
    //     *self = self.evaluate().into();
    //     self.base
    // }

    pub fn apply_modifier(&mut self, value: f64, modifier: Modifier) {
        match modifier {
            Modifier::Multiplier => {
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
    }

    pub fn apply_effect(&mut self, effect: &StatEffect) {
        // For retro compatibility
        let modifier = match effect.modifier {
            Modifier::Multiplier if effect.stat.is_multiplicative() => Modifier::More,
            modifier => modifier,
        };

        self.apply_modifier(effect.value, modifier);
    }

    pub fn apply_negative_effect(&mut self, effect: &StatEffect) {
        self.apply_effect(&StatEffect {
            value: -effect.value,
            ..effect.clone()
        })
    }

    pub fn convert_value(&mut self, percent: f64, is_extra: bool, only_base: bool) -> T {
        let mut percent = percent;
        if !is_extra {
            percent = percent.max(100.0 - self.converted);
            self.converted += percent;
        }

        (if only_base {
            self.base
        } else {
            self.evaluate()
        })
        .multiply_value(percent * 0.01)
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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ModifiableChance {
    pub value: ModifiableValue<f32>,
    pub lucky_chance: ModifiableValue<f32>,
}

impl ModifiableChance {
    pub fn evaluate(self) -> Chance {
        Chance {
            value: self.value.evaluate(),
            lucky_chance: self.lucky_chance.evaluate(),
        }
    }
}

impl From<&Chance> for ModifiableChance {
    fn from(value: &Chance) -> Self {
        ModifiableChance {
            value: value.value.into(),
            lucky_chance: value.lucky_chance.into(),
        }
    }
}

impl From<Chance> for ModifiableChance {
    fn from(value: Chance) -> Self {
        ModifiableChance {
            value: value.value.into(),
            lucky_chance: value.lucky_chance.into(),
        }
    }
}

// impl From<ModifiableChance> for Chance {
//     fn from(value: ModifiableChance) -> Self {
//         Self {
//             value: value.value.evaluate(),
//             lucky_chance: value.lucky_chance.evaluate(),
//         }
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ModifiableChanceRange<T> {
    pub min: T,
    pub max: T,
    pub lucky_chance: ModifiableValue<f32>,
}

// impl<T> ModifiableChanceRange<T> {
//     pub fn evaluate(self) -> ChanceRange<T> {
//         ChanceRange {
//             min: self.min,
//             max: self.max,
//             lucky_chance: self.lucky_chance.evaluate(),
//         }
//     }
// }

impl<T> ModifiableChanceRange<ModifiableValue<T>>
where
    T: std::ops::Add<Output = T> + BaseModifiableValue + Default + Copy,
{
    pub fn evaluate(self) -> ChanceRange<T> {
        ChanceRange {
            min: self.min.evaluate(),
            max: self.max.evaluate(),
            lucky_chance: self.lucky_chance.evaluate(),
        }
    }
}

impl<T> From<ChanceRange<T>> for ModifiableChanceRange<ModifiableValue<T>>
where
    T: Default,
{
    fn from(value: ChanceRange<T>) -> Self {
        ModifiableChanceRange {
            min: value.min.into(),
            max: value.max.into(),
            lucky_chance: value.lucky_chance.into(),
        }
    }
}

pub type ModifiableDamageMap = HashMap<DamageType, ModifiableChanceRange<ModifiableValue<f64>>>;

pub fn to_modifiable_damage_map(damage_map: &DamageMap) -> ModifiableDamageMap {
    damage_map
        .iter()
        .map(|(d, c)| {
            (
                *d,
                ModifiableChanceRange {
                    min: c.min.into(),
                    max: c.max.into(),
                    lucky_chance: c.lucky_chance.into(),
                },
            )
        })
        .collect()
}
