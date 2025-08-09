use std::hash::{DefaultHasher, Hash, Hasher};
use rand::distributions::WeightedIndex;
use rand::prelude::{Distribution, StdRng};
use rand::{Rng, SeedableRng};

/// Provides random numbers to the WFC.
pub struct Random {
    rng: StdRng,
}

impl Random {
    pub fn new() -> Self {
        Random {
            rng: StdRng::from_entropy()
        }
    }

    pub fn from_seed(seed: impl Hash) -> Self {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let seed = hasher.finish();

        Random {
            rng: StdRng::seed_from_u64(seed)
        }
    }

    pub fn choose<T: Copy>(
        &mut self,
        choices: &[T],
    ) -> T {
        let index = self.rng.gen_range(0..choices.len());
        choices[index]
    }

    pub fn choose_weighted<T: Copy>(
        &mut self,
        weights: &[f32],
        choices: &[T],
    ) -> T {
        let dist = WeightedIndex::new(weights).unwrap();
        choices[dist.sample(&mut self.rng)]
    }
}
