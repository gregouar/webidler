use rand::{
    distr::uniform::{SampleRange, SampleUniform},
    Rng,
};

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

pub fn random_weighted_pick<I>(items: Vec<&I>) -> Option<&I>
where
    I: RandomWeighted,
{
    random_range(0..items.iter().map(|w| w.random_weight()).sum()).and_then(|p| {
        items
            .iter()
            .scan(0, |cumul_prob, &w| {
                *cumul_prob += w.random_weight();
                Some((*cumul_prob, w))
            })
            .find(|(max_prob, w)| p >= *max_prob - w.random_weight() && p < *max_prob)
            .map(|(_, w)| w)
    })
}
