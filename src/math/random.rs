use rand::{
    distributions::uniform::{SampleRange, SampleUniform},
    Rng,
};
/// Generate a random number in the range.
pub fn range<T, R>(range: R) -> T
where
    T: SampleUniform,
    R: SampleRange<T>,
{
    let mut rng = rand::thread_rng();
    rng.gen_range(range)
}
