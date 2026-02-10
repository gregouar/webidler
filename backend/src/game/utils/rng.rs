use rand::{
    distr::uniform::{SampleRange, SampleUniform},
    Rng, SeedableRng,
};
use rand_chacha::ChaCha8Rng;

use shared::data::chance::{Chance, ChanceRange};

pub type RngSeed = ChaCha8Rng;

pub fn roll_seed() -> RngSeed {
    RngSeed::seed_from_u64(rand::rng().random())
}

pub fn flip_coin() -> bool {
    let mut rng = rand::rng();
    rng.random_bool(0.5)
}

pub fn random_range<T, R>(range: R) -> Option<T>
where
    T: SampleUniform,
    R: SampleRange<T>,
{
    random_range_with_seed(range, &mut roll_seed())
}

pub fn random_range_with_seed<T, R>(range: R, seed: &mut RngSeed) -> Option<T>
where
    T: SampleUniform,
    R: SampleRange<T>,
{
    if range.is_empty() {
        return None;
    }

    Some(seed.random_range(range))
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
    fn roll_with_seed(&self, seed: &mut RngSeed) -> T;
    fn roll(&self) -> T {
        self.roll_with_seed(&mut roll_seed())
    }
    fn clamp(&mut self);
}

impl Rollable<bool> for Chance {
    fn roll_with_seed(&self, seed: &mut RngSeed) -> bool {
        let first_result = random_range_with_seed(0.0..=100.0, seed).unwrap_or(100.0) <= self.value;
        let second_result =
            random_range_with_seed(0.0..=100.0, seed).unwrap_or(100.0) <= self.value;

        match roll_luck(self.lucky_chance, seed) {
            LuckResult::Unlucky => first_result.min(second_result),
            LuckResult::Normal => first_result,
            LuckResult::Lucky => first_result.max(second_result),
        }
    }

    fn clamp(&mut self) {
        self.value = self.value.clamp(0.0, 100.0);
        self.lucky_chance = self.lucky_chance.clamp(-100.0, 100.0);
    }
}

impl<T> Rollable<T> for ChanceRange<T>
where
    T: rand::distr::uniform::SampleUniform + PartialOrd + Copy,
{
    fn roll_with_seed(&self, seed: &mut RngSeed) -> T {
        let first_result = random_range_with_seed(self.min..=self.max, seed).unwrap_or(self.max);
        let second_result = random_range_with_seed(self.min..=self.max, seed).unwrap_or(self.max);

        match roll_luck(self.lucky_chance, seed) {
            LuckResult::Unlucky => match first_result.partial_cmp(&second_result) {
                Some(std::cmp::Ordering::Greater) => second_result,
                _ => first_result,
            },
            LuckResult::Normal => first_result,
            LuckResult::Lucky => match first_result.partial_cmp(&second_result) {
                Some(std::cmp::Ordering::Less) => second_result,
                _ => first_result,
            },
        }
    }

    fn clamp(&mut self) {
        if let Some(ordering) = self.min.partial_cmp(&self.max)
            && ordering == std::cmp::Ordering::Greater {
                self.min = self.max;
            }
        self.lucky_chance = self.lucky_chance.clamp(-100.0, 100.0);
    }
}

enum LuckResult {
    Unlucky,
    Normal,
    Lucky,
}

fn roll_luck(lucky_chance: f32, seed: &mut RngSeed) -> LuckResult {
    if random_range_with_seed(0.0..=100.0, seed).unwrap_or(100.0) <= lucky_chance.abs() {
        if lucky_chance < 0.0 {
            return LuckResult::Unlucky;
        } else if lucky_chance > 0.0 {
            return LuckResult::Lucky;
        }
    }

    LuckResult::Normal
}
