use rand::{
    distr::uniform::{SampleRange, SampleUniform},
    Rng,
};

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

// impl<'a, T> RandomWeighted for T where &'a T: RandomWeighted {}

pub fn random_weighted_pick<I>(items: &Vec<I>) -> Option<&I>
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
