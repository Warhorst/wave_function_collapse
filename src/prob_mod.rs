// TODO instead of setting my weights using an iterator where I need to know the index in the
// tiles, I could use an initial probability modifier to set the weights initially, and
// then some to update per cell

use pad::position::Position;

/// Modifies the probability of tiles. The modifiers are applied when a tile to
/// collapse should be picked.
pub trait ProbabilityModifier<T> {
    fn modify_probability(
        &self,
        weights: &mut Vec<f32>,
        position: Position,
        width: usize,
        height: usize,
        collapsed_positions: &[(&T, Position)],
    );
}

impl<T, F> ProbabilityModifier<T> for F
where
    F: Fn(&mut Vec<f32>, Position, usize, usize, &[(&T, Position)]),
{
    fn modify_probability(
        &self,
        weights: &mut Vec<f32>,
        position: Position,
        width: usize,
        height: usize,
        collapsed_positions: &[(&T, Position)],
    ) {
        self(weights, position, width, height, collapsed_positions)
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
        weights: impl IntoIterator<Item = &'a f32>,
        collapsed_positions: impl IntoIterator<Item = (&'a T, Position)>,
        position: Position,
        width: usize,
        height: usize,
    ) -> &[f32]
    where
        T: 'a,
    {
        // TODO maybe make the weights a special struct which holds its original weights and its currently modified weighst
        // Set the weights to the current weights
        weights
            .into_iter()
            .copied()
            .enumerate()
            .for_each(|(i, w)| self.weights[i] = w);

        // TODO the WFC itself should hold a vec of all collapsed positions and provide these as a reference rather than an iter
        let collapsed_positions = collapsed_positions.into_iter().collect::<Vec<_>>();

        self.modifiers.iter().for_each(|m| {
            m.modify_probability(
                &mut self.weights,
                position,
                width,
                height,
                &collapsed_positions,
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
