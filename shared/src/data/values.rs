use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::data::modifier::BaseModifiableValue;

// Compile-time bounds

macro_rules! bounded_value {
    (
        $vis:vis struct $name:ident ($inner:ty);
        $(min = $min:expr;)?
        $(max = $max:expr;)?
    ) => {
        #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
        $vis struct $name(pub $inner);

        impl $name {
            pub fn new(v: $inner) -> Self {
                Self(v)
            }

            pub fn get(&self) -> $inner {
                let mut val = self.0;

                #[allow(clippy::manual_clamp)]
                $( if val < $min { val = $min; } )?
                $( if val > $max { val = $max; } )?

                val
            }
        }

        impl From<$inner> for $name {
            fn from(v: $inner) -> Self {
                Self::new(v)
            }
        }

        impl From<$name> for $inner {
            fn from(v: $name) -> $inner {
                v.get()
            }
        }

        impl std::ops::Add for $name {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self::new(self.0 + rhs.0)
            }
        }

        impl std::ops::Sub for $name {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                Self::new(self.0 - rhs.0)
            }
        }

        impl std::ops::Mul<f64> for $name {
            type Output = Self;
            fn mul(self, rhs: f64) -> Self {
                Self::new(self.0 * rhs as $inner)
            }
        }

        impl std::ops::AddAssign for $name {
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        impl std::ops::SubAssign for $name {
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }

        impl BaseModifiableValue for $name {
            fn multiply_value(&self, factor: f64) -> Self {
                Self::new(self.0 * factor as $inner)
            }

            fn add_value(&self, value: f64) -> Self {
                Self::new(self.0 + value as $inner)
            }

            fn is_negative(&self) -> bool {
                self.0 < 0.0
            }

            fn round(&self) -> Self {
                Self::new(self.0.round() as $inner)
            }
        }
    };
}

bounded_value! {
    pub struct Percent(f32);
    min = 0.0;
    max = 100.0;
}

bounded_value! {
    pub struct Luck(f32);
    min = -100.0;
    max = 100.0;
}

bounded_value! {
    pub struct Damage(f64);
    min = 0.0;
}

bounded_value! {
    pub struct NonNegative(f64);
    min = 0.0;
}

bounded_value! {
    pub struct Cooldown(f64);
    min = 0.0;
    max = 1.0;
}

bounded_value! {
    pub struct AtLeastOne(f64);
    min = 1.0;
}

// Runtime bounds

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundedValue<T> {
    value: T,
    min: Option<T>,
    max: Option<T>,
}

impl<T> Default for BoundedValue<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            value: T::default(),
            min: None,
            max: None,
        }
    }
}

impl<T> BoundedValue<T>
where
    T: Copy + PartialOrd,
{
    pub fn new(value: T, min: Option<T>, max: Option<T>) -> Self {
        Self { value, min, max }
    }

    pub fn get(&self) -> T {
        let mut v = self.value;

        if let Some(m) = self.min
            && v < m
        {
            v = m;
        }

        if let Some(m) = self.max
            && v > m
        {
            v = m;
        }

        v
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
    }

    pub fn set_bounds(&mut self, min: Option<T>, max: Option<T>) {
        self.min = min;
        self.max = max;
    }

    pub fn bounds(&self) -> (Option<T>, Option<T>) {
        (self.min, self.max)
    }
}

impl<T> Serialize for BoundedValue<T>
where
    T: Serialize + PartialOrd + Copy,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.get().serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for BoundedValue<T>
where
    T: Deserialize<'de> + Default + Copy,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Ok(Self {
            value,
            min: None,
            max: None,
        })
    }
}

impl<T> std::ops::Add for BoundedValue<T>
where
    T: Copy + PartialOrd + std::ops::Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(
            self.value + rhs.value,
            self.min.or(rhs.min),
            self.max.or(rhs.max),
        )
    }
}

impl<T> std::ops::AddAssign for BoundedValue<T>
where
    T: Copy + PartialOrd + std::ops::Add<Output = T>,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T> std::ops::Mul<f64> for BoundedValue<T>
where
    T: Copy + PartialOrd + From<f64> + std::ops::Mul<Output = T>,
{
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        Self::new(self.value * T::from(rhs), self.min, self.max)
    }
}

impl<T> BaseModifiableValue for BoundedValue<T>
where
    T: BaseModifiableValue + Copy + PartialOrd,
{
    fn multiply_value(&self, value: f64) -> Self {
        Self {
            value: self.value.multiply_value(value),
            min: self.min,
            max: self.max,
        }
    }

    fn add_value(&self, value: f64) -> Self {
        Self {
            value: self.value.add_value(value),
            min: self.min,
            max: self.max,
        }
    }

    fn is_negative(&self) -> bool {
        self.value.is_negative()
    }

    fn round(&self) -> Self {
        Self {
            value: self.value.round(),
            min: self.min,
            max: self.max,
        }
    }
}
