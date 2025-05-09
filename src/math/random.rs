use rand::{
    distr::uniform::{SampleRange, SampleUniform},
    Rng,
};
/// Generate a random number in the range.
pub fn range<T, R>(range: R) -> T
where
    T: SampleUniform,
    R: SampleRange<T>,
{
    let mut rng = rand::rng();
    rng.random_range(range)
}
