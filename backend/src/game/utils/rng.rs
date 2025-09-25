use rand::{
    distr::uniform::{SampleRange, SampleUniform},
    Rng,
};
use shared::data::chance::{Chance, QuantityChance, ValueChance};

pub fn flip_coin() -> bool {
    let mut rng = rand::rng();
    rng.random_bool(0.5)
}

pub fn random_range<T, R>(range: R) -> Option<T>
where
    T: SampleUniform,
    R: SampleRange<T>,
{
    if range.is_empty() {
        return None;
    }

    let mut rng = rand::rng();
    Some(rng.random_range(range))
}

pub trait RandomWeighted {
    fn random_weight(&self) -> u64;
}

pub fn random_weighted_pick<I>(items: &[I]) -> Option<&I>
where
    I: RandomWeighted,
{
    random_range(0..items.iter().map(|item| item.random_weight()).sum()).and_then(|p| {
        items
            .iter()
            .scan(0, |cumul_prob, item| {
                *cumul_prob += item.random_weight();
                Some((*cumul_prob, item))
            })
            .find(|(max_prob, item)| p >= *max_prob - item.random_weight() && p < *max_prob)
            .map(|(_, item)| item)
    })
}

pub trait Rollable<T> {
    fn roll(&self) -> T;
    fn clamp(&mut self);
}

impl Rollable<bool> for Chance {
    fn roll(&self) -> bool {
        if random_range(0.0..=100.0).unwrap_or(100.0) <= self.value {
            return true;
        }

        if random_range(0.0..=100.0).unwrap_or(100.0) <= self.lucky_chance {
            if random_range(0.0..=100.0).unwrap_or(100.0) <= self.value {
                return true;
            }
        }

        false
    }

    fn clamp(&mut self) {
        self.value.clamp(0.0, 100.0);
        self.lucky_chance.clamp(0.0, 100.0);
    }
}

impl Rollable<f64> for ValueChance {
    fn roll(&self) -> f64 {
        let first_result = random_range(self.min..=self.max).unwrap_or(self.max);

        if random_range(0.0..=100.0).unwrap_or(100.0) <= self.lucky_chance {
            return first_result.max(random_range(self.min..=self.max).unwrap_or(self.max));
        } else {
            return first_result;
        }
    }

    fn clamp(&mut self) {
        self.min = self.min.min(self.max);
        self.lucky_chance.clamp(0.0, 100.0);
    }
}

impl Rollable<u16> for QuantityChance {
    fn roll(&self) -> u16 {
        let first_result = random_range(self.min..=self.max).unwrap_or(self.max);

        if random_range(0.0..=100.0).unwrap_or(100.0) <= self.lucky_chance {
            return first_result.max(random_range(self.min..=self.max).unwrap_or(self.max));
        } else {
            return first_result;
        }
    }

    fn clamp(&mut self) {
        self.min = self.min.min(self.max);
        self.lucky_chance.clamp(0.0, 100.0);
    }
}
