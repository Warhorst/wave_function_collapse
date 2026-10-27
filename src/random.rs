use crate::cell::PossibleIndices;
use rand::{
    RngCore, SeedableRng, distributions::WeightedIndex, prelude::{Distribution, StdRng},
};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::fmt::Debug;

/// Struct which allows the [`crate::Wfc`] to make random choices.
pub struct Random {
    /// The original seed used to create this Random
    seed: Box<dyn Debug + 'static>,
    rng: StdRng,
}

impl Random {
    /// Create a [`Random`] with a random seed.
    pub fn new() -> Self {
        let mut rng = StdRng::from_entropy();
        let seed = rng.next_u64();
    
        Random {
            seed: Box::new(seed),
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Create a [`Random`] from the given seed. The [`Hash`] of the seed is
    /// used as the seed for the Random.
    pub fn from_seed(seed: impl Hash + Debug + 'static) -> Self {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let seed_hash = hasher.finish();

        Random {
            seed: Box::new(seed),
            rng: StdRng::seed_from_u64(seed_hash),
        }
    }

    /// Choose an index from the [`PossibleIndices`] based on the provided weights.
    pub fn choose_weighted(
        &mut self,
        weights: impl IntoIterator<Item = f32>,
        choices: PossibleIndices,
    ) -> u8 {
        let dist = WeightedIndex::new(weights).unwrap();
        choices.get(dist.sample(&mut self.rng))
    }

    /// Return the original seed used to create this [`Random`].
    pub fn seed(&self) -> impl Debug {
        &self.seed
    }
}
