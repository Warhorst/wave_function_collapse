use crate::cell::PossibleIndices;
use rand::{
    SeedableRng,
    distributions::WeightedIndex,
    prelude::{Distribution, StdRng},
};
use std::hash::{DefaultHasher, Hash, Hasher};

/// Provides random numbers to the WFC.
pub struct Random {
    rng: StdRng,
}

impl Random {
    pub fn new() -> Self {
        Random {
            rng: StdRng::from_entropy(),
        }
    }

    pub fn from_seed(seed: impl Hash) -> Self {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let seed = hasher.finish();

        Random {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn choose_weighted(
        &mut self,
        weights: impl IntoIterator<Item = f32>,
        choices: PossibleIndices,
    ) -> u8 {
        let dist = WeightedIndex::new(weights).unwrap();
        choices.get(dist.sample(&mut self.rng))
    }
}
