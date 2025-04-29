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
