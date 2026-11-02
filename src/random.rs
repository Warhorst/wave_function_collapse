use crate::cell::PossibleIndices;
use rand::{
    RngCore, SeedableRng,
    distributions::WeightedIndex,
    prelude::{Distribution, StdRng},
};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::{fmt::Debug, ops::Deref};

/// Something that can be used to seed a [`Random`].
pub trait Seed: Debug {
    /// Create a [`Hash`] value from this [`Seed`] which can be used
    /// to seed a [`Random`].
    fn seed_hash(&self) -> u64;
}

impl<T: Hash + ?Sized + Debug> Seed for T {
    fn seed_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

/// Struct which allows the [`crate::Wfc`] to make random choices.
pub struct Random {
    /// The original seed used to create this Random
    seed: Box<dyn Seed + 'static>,
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

    /// Create a [`Random`] from the given [`Seed`].
    pub fn from_seed(seed: impl Seed + 'static) -> Self {
        let seed_hash = seed.seed_hash();

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
        let dist = WeightedIndex::new(weights).expect("The provided weights must be valid");
        choices.get(dist.sample(&mut self.rng))
    }

    /// Return the original seed used to create this [`Random`].
    pub fn seed(&self) -> &dyn Seed {
        self.seed.deref()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cell::PossibleIndices,
        random::{Random, Seed},
    };

    #[test]
    fn seed_creation_stable() {
        // Check if the hash of some known seeds did not change. If these tests fail,
        // something in the hash creation was changed.

        assert_eq!(11710589568619304606, "Seed".seed_hash());
        assert_eq!(11710589568619304606, "Seed".to_string().seed_hash());
        assert_eq!(8880661182590738257, 42_u64.seed_hash());
        assert_eq!(8880661182590738257, 42_usize.seed_hash());
        assert_eq!(4952851536318644461, true.seed_hash());
        assert_eq!(7541581120933061747, false.seed_hash());
    }

    #[test]
    fn random_stable() {
        // Check if the same indices are still chosen as previously. If these tests fail,
        // something in the random generator was changed.
    
        assert_eq!(184, chose_index("Seed"));
        assert_eq!(184, chose_index("Seed".to_string()));
        assert_eq!(23, chose_index(42_u64));
        assert_eq!(23, chose_index(42_usize));
        assert_eq!(218, chose_index(true));
        assert_eq!(1, chose_index(false));
    }

    fn chose_index(seed: impl Seed + 'static) -> u8 {
        let indices = (0..250)
            .into_iter()
            .enumerate()
            .map(|(i, _)| i as u8)
            .collect::<Vec<_>>();
        let choices = PossibleIndices::from_array(&indices, 250);
        let weights = vec![1.0; 250];

        let mut random = Random::from_seed(seed);
        random.choose_weighted(weights, choices)
    }
}
