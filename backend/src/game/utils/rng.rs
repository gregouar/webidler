use rand::{
    Rng, SeedableRng,
    distr::uniform::{SampleRange, SampleUniform},
};
use rand_chacha::ChaCha8Rng;

use shared::data::{
    chance::{BoundedChance, Chance, ChanceRange},
    modifier::{BaseModifiableValue, ModifiableValue},
    rng::{MARBLE_COUNT, MarbleBag},
    values::Luck,
};

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

pub trait Rollable<T>
where
    Self: Sized,
{
    fn roll_with_seed(self, seed: &mut RngSeed) -> T;
    fn roll(self) -> T {
        self.roll_with_seed(&mut roll_seed())
    }
}

/// Rolls a value using percentages supplied by a marble bag.
///
/// Luck checks and their additional values use the regular RNG and never
/// consume marbles.
pub trait MarbleRollable<T>
where
    Self: Sized,
{
    fn roll_with_marble_rolls(self, seed: &mut RngSeed, marble_bag: &mut MarbleBag) -> T;

    fn roll_with_marble_bag(self, marble_bag: &mut MarbleBag) -> T {
        let mut seed = roll_seed();
        self.roll_with_marble_rolls(&mut seed, marble_bag)
    }
}

impl Rollable<bool> for &Chance {
    fn roll_with_seed(self, seed: &mut RngSeed) -> bool {
        let first_result =
            random_range_with_seed(0.0..=100.0, seed).unwrap_or(100.0) <= self.value.get();
        let second_result =
            random_range_with_seed(0.0..=100.0, seed).unwrap_or(100.0) <= self.value.get();

        match roll_luck(*self.lucky_chance, seed) {
            LuckResult::Unlucky => first_result.min(second_result),
            LuckResult::Normal => first_result,
            LuckResult::Lucky => first_result.max(second_result),
        }
    }
}

impl MarbleRollable<bool> for &Chance {
    fn roll_with_marble_rolls(self, seed: &mut RngSeed, marble_bag: &mut MarbleBag) -> bool {
        let first_result = marble_bag.roll_with_seed(seed) <= self.value.get();

        match roll_luck(*self.lucky_chance, seed) {
            LuckResult::Unlucky => first_result.min(
                random_range_with_seed(0.0..=100.0, seed).unwrap_or(100.0) <= self.value.get(),
            ),
            LuckResult::Normal => first_result,
            LuckResult::Lucky => first_result.max(
                random_range_with_seed(0.0..=100.0, seed).unwrap_or(100.0) <= self.value.get(),
            ),
        }
    }
}

impl Rollable<bool> for &BoundedChance {
    fn roll_with_seed(self, seed: &mut RngSeed) -> bool {
        let first_result =
            random_range_with_seed(0.0..=100.0, seed).unwrap_or(100.0) <= self.value.get();
        let second_result =
            random_range_with_seed(0.0..=100.0, seed).unwrap_or(100.0) <= self.value.get();

        match roll_luck(*self.lucky_chance, seed) {
            LuckResult::Unlucky => first_result.min(second_result),
            LuckResult::Normal => first_result,
            LuckResult::Lucky => first_result.max(second_result),
        }
    }
}

impl MarbleRollable<bool> for &BoundedChance {
    fn roll_with_marble_rolls(self, seed: &mut RngSeed, marble_bag: &mut MarbleBag) -> bool {
        let first_result = marble_bag.roll_with_seed(seed) <= self.value.get();

        match roll_luck(*self.lucky_chance, seed) {
            LuckResult::Unlucky => first_result.min(
                random_range_with_seed(0.0..=100.0, seed).unwrap_or(100.0) <= self.value.get(),
            ),
            LuckResult::Normal => first_result,
            LuckResult::Lucky => first_result.max(
                random_range_with_seed(0.0..=100.0, seed).unwrap_or(100.0) <= self.value.get(),
            ),
        }
    }
}

impl<T> Rollable<T> for &ChanceRange<T>
where
    T: rand::distr::uniform::SampleUniform + PartialOrd + Copy,
{
    fn roll_with_seed(self, seed: &mut RngSeed) -> T {
        let min = if let Some(ordering) = self.min.partial_cmp(&self.max)
            && ordering == std::cmp::Ordering::Greater
        {
            self.max
        } else {
            self.min
        };

        let first_result = random_range_with_seed(min..=self.max, seed).unwrap_or(self.max);
        let second_result = random_range_with_seed(min..=self.max, seed).unwrap_or(self.max);

        match roll_luck(*self.lucky_chance, seed) {
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
}

impl<T> Rollable<ModifiableValue<T>> for ChanceRange<ModifiableValue<T>>
where
    T: Into<f64> + From<f64> + Copy + BaseModifiableValue,
{
    fn roll_with_seed(self, seed: &mut RngSeed) -> ModifiableValue<T> {
        let normalized_value = ChanceRange::<f64> {
            min: 0.0,
            max: 1.0,
            lucky_chance: self.lucky_chance,
        }
        .roll_with_seed(seed);

        reconstruct_modifiable_roll(&self, normalized_value)
    }
}

impl<T> MarbleRollable<ModifiableValue<T>> for ChanceRange<ModifiableValue<T>>
where
    T: Into<f64> + From<f64> + Copy + BaseModifiableValue,
{
    fn roll_with_marble_rolls(
        self,
        seed: &mut RngSeed,
        marble_bag: &mut MarbleBag,
    ) -> ModifiableValue<T> {
        let normalized_value = marble_bag.roll_with_seed(seed) as f64 * 0.01;

        let normalized_value = match roll_luck(*self.lucky_chance, seed) {
            LuckResult::Unlucky => {
                normalized_value.min(random_range_with_seed(0.0..=1.0, seed).unwrap_or(1.0))
            }
            LuckResult::Normal => normalized_value,
            LuckResult::Lucky => {
                normalized_value.max(random_range_with_seed(0.0..=1.0, seed).unwrap_or(1.0))
            }
        };

        reconstruct_modifiable_roll(&self, normalized_value)
    }
}

fn reconstruct_modifiable_roll<T>(
    range: &ChanceRange<ModifiableValue<T>>,
    normalized_value: f64,
) -> ModifiableValue<T>
where
    T: Into<f64> + From<f64> + Copy + BaseModifiableValue,
{
    let max: f64 = (*range.max).into();
    let min: f64 = (*range.min).into();

    let normalized_value = if min <= max {
        normalized_value.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let interpolate = |min: f64, max: f64| min + (max - min) * normalized_value;

    let rolled_value = interpolate(min, max);
    let increased = interpolate(range.min.increased(), range.max.increased());
    let decreased = interpolate(range.min.decreased(), range.max.decreased());
    let more = interpolate(range.min.more(), range.max.more());

    let factor = ModifiableValue::new(T::from(1.0), increased, decreased, more, 0.0).factor();

    if factor == 0.0 || !factor.is_finite() || rolled_value < 0.0 {
        return ModifiableValue::new(T::from(rolled_value), 0.0, 0.0, 0.0, 0.0);
    }

    ModifiableValue::new(
        T::from(rolled_value / factor),
        increased,
        decreased,
        more,
        0.0,
    )
}

enum LuckResult {
    Unlucky,
    Normal,
    Lucky,
}

fn roll_luck(lucky_chance: Luck, seed: &mut RngSeed) -> LuckResult {
    let lucky_chance = lucky_chance.get();
    if random_range_with_seed(0.0..=100.0, seed).unwrap_or(100.0) <= lucky_chance.abs() {
        if lucky_chance < 0.0 {
            return LuckResult::Unlucky;
        } else if lucky_chance > 0.0 {
            return LuckResult::Lucky;
        }
    }

    LuckResult::Normal
}

const MARBLE_MASK: u32 = (1 << MARBLE_COUNT) - 1;
const MARBLE_SLICE_WIDTH: f32 = 100.0 / MARBLE_COUNT as f32;

impl Rollable<f32> for &mut MarbleBag {
    fn roll_with_seed(self, seed: &mut RngSeed) -> f32 {
        let rank = seed.random_range(0..self.remaining_marbles);
        let mut free_marbles = !self.picked_marbles & MARBLE_MASK;

        for _ in 0..rank {
            free_marbles &= free_marbles - 1;
        }

        let slice = free_marbles.trailing_zeros();
        self.picked_marbles |= 1 << slice;
        self.remaining_marbles -= 1;

        if self.remaining_marbles == 0 {
            self.picked_marbles = 0;
            self.remaining_marbles = MARBLE_COUNT;
        }

        (slice as f32 + seed.random::<f32>()) * MARBLE_SLICE_WIDTH
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::data::values::Percent;

    #[test]
    fn reconstructed_modifiable_roll_preserves_value_and_interpolates_modifiers() {
        let range: ChanceRange<ModifiableValue<f64>> = ChanceRange {
            min: ModifiableValue::new(10.0, 100.0, 0.0, 0.0, 0.0),
            max: ModifiableValue::new(30.0, 0.0, 0.0, 0.0, 0.0),
            lucky_chance: Default::default(),
        };

        let rolled = reconstruct_modifiable_roll(&range, 0.5);

        assert!((*rolled - 25.0).abs() < 1e-12);
        assert!((rolled.increased() - 50.0).abs() < 1e-12);
        assert_eq!(rolled.converted(), 0.0);
    }

    #[test]
    fn reconstructed_modifiable_roll_bakes_conversion_into_base() {
        let range: ChanceRange<ModifiableValue<f64>> = ChanceRange {
            min: ModifiableValue::new(20.0, 0.0, 0.0, 0.0, 50.0),
            max: ModifiableValue::new(40.0, 0.0, 0.0, 0.0, 50.0),
            lucky_chance: Default::default(),
        };

        let rolled = reconstruct_modifiable_roll(&range, 0.5);

        assert!((*rolled - 15.0).abs() < 1e-12);
        assert!((*rolled.base() - 15.0).abs() < 1e-12);
        assert_eq!(rolled.converted(), 0.0);
    }

    #[test]
    fn marble_bag_picks_every_slice_once_before_resetting() {
        let mut bag = MarbleBag::new();
        let mut seed = RngSeed::seed_from_u64(42);
        let mut picked_slices = 0_u32;

        for _ in 0..MARBLE_COUNT {
            let roll = (&mut bag).roll_with_seed(&mut seed);
            let slice = (roll / MARBLE_SLICE_WIDTH) as u32;

            assert!((0.0..100.0).contains(&roll));
            assert_eq!(picked_slices & (1 << slice), 0);
            picked_slices |= 1 << slice;
        }

        assert_eq!(picked_slices, MARBLE_MASK);
        assert_eq!(bag, MarbleBag::new());
    }

    #[test]
    fn normal_marble_roll_only_draws_once() {
        let chance = Chance::new_sure();
        let mut seed = RngSeed::seed_from_u64(42);
        let mut bag = MarbleBag::new();

        assert!((&chance).roll_with_marble_rolls(&mut seed, &mut bag));
        assert_eq!(bag.remaining_marbles, MARBLE_COUNT - 1);
        assert_eq!(bag.picked_marbles.count_ones(), 1);
    }

    #[test]
    fn lucky_roll_value_does_not_consume_another_marble() {
        let chance = Chance {
            value: Percent::new(50.0).into(),
            lucky_chance: Luck::new(100.0).into(),
        };
        let mut bag = MarbleBag::new();

        let _ = (&chance).roll_with_marble_bag(&mut bag);

        assert_eq!(bag.remaining_marbles, MARBLE_COUNT - 1);
        assert_eq!(bag.picked_marbles.count_ones(), 1);
    }
}
