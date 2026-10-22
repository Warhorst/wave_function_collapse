// TODO instead of setting my weights using an iterator where I need to know the index in the
// tiles, I could use an initial probability modifier to set the weights initially, and
// then some to update per cell

use pad::position::Position;

/// Modifies the probability of tiles. The modifiers are applied when a tile to
/// collapse should be picked.
pub trait ProbabilityModifier<T> {
    /// Modifiy the tile probabilities for the given position.
    ///
    /// * `position`: The [`Position`] which tile probabilities will be modified
    /// * `position`: The weights to modify. These might be the base weigths or, when working with multiple modifiers, the modified weights of a previous modifier.
    /// * `context`: The [`ProbModContext`] which provides additional data that might be relevant for the modication.
    fn modify_probability(
        &self,
        position: Position,
        weights: &mut Vec<f32>,
        context: ProbModContext<T>,
    );
}

/// Context for a [`ProbabilityModifier`] which contains relevant, read-only data
/// for the modification.
pub struct ProbModContext<'a, T> {
    /// The width of the board
    pub width: usize,
    /// The height of the board
    pub height: usize,
    /// All existing tiles in the WFC. The index of the tile is the same as the index to the weight of the tile.
    pub tiles: &'a [T],
    /// All currently collapsed positions in the WFC
    pub collapsed_positions: &'a [(Position, T)],
}

impl<T, F> ProbabilityModifier<T> for F
where
    F: Fn(Position, &mut Vec<f32>, ProbModContext<T>),
{
    fn modify_probability(
        &self,
        position: Position,
        weights: &mut Vec<f32>,
        context: ProbModContext<T>,
    ) {
        self(position, weights, context)
    }
}

pub(super) struct ProbabilityModifiers<T> {
    modifiers: Vec<Box<dyn ProbabilityModifier<T>>>,
    /// Preallocated vec of weights for the tiles
    weights: Vec<f32>,
}

impl<T> ProbabilityModifiers<T> {
    pub(super) fn new(num_tiles: usize) -> Self {
        ProbabilityModifiers {
            modifiers: vec![],
            weights: vec![0.0; num_tiles],
        }
    }

    pub(super) fn modify_probability<'a>(
        &mut self,
        position: Position,
        weights: impl IntoIterator<Item = &'a f32>,
        width: usize,
        height: usize,
        tiles: &[T],
        collapsed_positions: &[(Position, T)],
    ) -> &[f32]
    where
        T: 'a,
    {
        weights
            .into_iter()
            .copied()
            .enumerate()
            .for_each(|(i, w)| self.weights[i] = w);

        self.modifiers.iter().for_each(|m| {
            m.modify_probability(
                position,
                &mut self.weights,
                ProbModContext {
                    width,
                    height,
                    tiles,
                    collapsed_positions,
                },
            )
        });

        &self.weights
    }

    pub(super) fn add_modifier(
        &mut self,
        modifier: impl ProbabilityModifier<T> + 'static,
    ) {
        self.modifiers.push(Box::new(modifier));
    }
}
