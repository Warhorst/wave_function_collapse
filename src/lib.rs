use crate::{
    board::Board,
    cell::{Cell, PossibleIndices},
    constraints::{Constraint, Constraints},
    prob_mod::{ProbabilityModifier, ProbabilityModifiers},
    random::Random,
};
use pad::position::Position;
use std::{
    error::Error,
    fmt::{Display, Formatter},
    hash::Hash,
    marker::PhantomData,
};

mod board;
pub mod cell;
pub mod constraints;
pub mod prob_mod;
mod random;

/// Performs the wave function collapse.
pub struct Wfc<T: Clone, C: Cell> {
    board: Board<C>,
    tiles: Vec<T>,
    weights: Vec<f32>,
    tile_constraints: Constraints<T>,
    probability_modifiers: ProbabilityModifiers<T>,
    random: Random,
}

impl<T, C: Cell> Wfc<T, C>
where
    T: Clone,
{
    /// Start creating a new [`Wfc`].
    ///
    /// * `width`: The width (in [`Position`]s) of the board
    /// * `height`: The height (in [`Position`]s) of the board
    /// * `tiles`: The possible tiles in the Wfc. This list should be free of duplicates, as a duplicate just increases its chance of being picked.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        width: usize,
        height: usize,
        tiles: Vec<T>,
    ) -> WfcBuilder<T, C> {
        let num_tiles = tiles.len();
        WfcBuilder {
            width,
            height,
            weights: vec![1.0; num_tiles],
            tiles,
            constraints: Constraints::default(),
            probability_modifiers: ProbabilityModifiers::new(num_tiles),
            random: Random::new(),
            _cell_type: PhantomData,
        }
    }

    /// Collapse the [`Wfc`] until no more tiles are not collapsed.
    pub fn collapse(mut self) -> Result<Vec<(Position, T)>, WfcError> {
        while !self.board.collapsed() {
            let (pos, cell) = self.board.get_min_entropy_position();

            // TODO performance got worse
            let collapsed_positions = self
                .board
                .get_collapsed_indices()
                .map(|(pos, i)| (&self.tiles[i], pos));
            let modified_weights = self.probability_modifiers.modify_probability(
                &self.weights,
                collapsed_positions,
                pos,
                self.board.width,
                self.board.height,
            );

            let possible_indices = cell.get_possible_indices();
            let weights = possible_indices.iter().map(|i| modified_weights[i as usize]);
            let index = Self::choose_next_index(&mut self.random, possible_indices, weights);
            self.board.collapse_position(pos, index);
            self.board
                .propagate(pos, &mut self.tile_constraints, &self.tiles)?;
        }

        Ok(self
            .board
            .get_collapsed_indices()
            .map(|(pos, index)| (pos, self.tiles[index].clone()))
            .collect())
    }

    fn choose_next_index(
        random: &mut Random,
        possible_indices: PossibleIndices,
        tile_weights: impl IntoIterator<Item = f32>,
    ) -> u8 {
        random.choose_weighted(tile_weights, possible_indices)
    }
}

impl<T, C: Cell> Wfc<T, C>
where
    T: Clone + PartialEq,
{
    /// Collapse the board partially with the provided tiles.
    ///
    /// This will at first collapse all positions and afterward propagate all the changes
    /// to their neighbour positions. This allows for results that would otherwise be impossible
    /// due to the constraints. It might render the WFC impossible to solve.
    pub fn collapse_tiles(
        &mut self,
        tiles: impl IntoIterator<Item = (Position, T)>,
    ) -> Result<(), WfcError> {
        let tiles = tiles.into_iter().collect::<Vec<_>>();
        let get_tile_index = |tile: &T| {
            self.tiles
                .iter()
                .enumerate()
                .find(|(_, t)| *t == tile)
                .unwrap()
                .0 as u8
        };

        for (pos, t) in &tiles {
            self.board.collapse_position(*pos, get_tile_index(t));
        }

        for (pos, _) in tiles {
            self.board
                .propagate(pos, &mut self.tile_constraints, &self.tiles)?
        }

        Ok(())
    }
}

/// The builder for a [`Wfc`].
pub struct WfcBuilder<T: Clone, C: Cell> {
    width: usize,
    height: usize,
    tiles: Vec<T>,
    constraints: Constraints<T>,
    probability_modifiers: ProbabilityModifiers<T>,
    random: Random,
    weights: Vec<f32>,
    _cell_type: PhantomData<C>,
}

impl<T, C: Cell> WfcBuilder<T, C>
where
    T: Clone,
{
    /// Set a custom seed for the [`Wfc`]
    pub fn with_seed(
        mut self,
        seed: impl Hash,
    ) -> Self {
        self.random = Random::from_seed(seed);
        self
    }

    /// Overwrite the weights used for the tiles in the [`Wfc`]. By default, every
    /// tile hast the same probability.
    ///
    /// The index of the weight in the iterator will be the weight for the tile at the same index.
    /// Providing more or less weights than there are tiles will produce a [`WfcError::InvalidWeightAmount`]
    pub fn with_weights(
        mut self,
        tile_weights: impl IntoIterator<Item = f32>,
    ) -> Self {
        self.weights.clear();

        for weight in tile_weights.into_iter() {
            self.weights.push(weight);
        }

        self
    }

    /// Add a [`Constraint`] to the [`Wfc`].
    pub fn with_constraint(
        mut self,
        constraint: impl Constraint<T> + 'static,
    ) -> Self {
        self.constraints.add_constraint(constraint);
        self
    }

    /// Add a [`ProbabilityModifier`] to the [`Wfc`].
    pub fn with_probability_modifier(
        mut self,
        modifier: impl ProbabilityModifier<T> + 'static,
    ) -> Self {
        self.probability_modifiers.add_modifier(modifier);
        self
    }

    /// Validate the input and create a [`Wfc`].
    pub fn build(self) -> Result<Wfc<T, C>, WfcError> {
        if self.weights.len() != self.tiles.len() {
            return Err(WfcError::InvalidWeightAmount {
                expected: self.tiles.len(),
                was: self.weights.len(),
            });
        }

        let board = Board::<C>::new(self.width, self.height, self.tiles.len());

        Ok(Wfc {
            board,
            tiles: self.tiles,
            weights: self.weights,
            tile_constraints: self.constraints,
            probability_modifiers: self.probability_modifiers,
            random: self.random,
        })
    }
}

/// An error for the wave function collapse, which occurs due to configuration errors
/// or runtime errors.
#[derive(Debug)]
pub enum WfcError {
    /// A different amount of weights was set in the WFC than expected.
    InvalidWeightAmount { expected: usize, was: usize },
    /// A cell has zero entropy after a propagation, which means no tile
    /// can be picked for it
    CellHasZeroEntropy(Position),
}

impl Error for WfcError {}

impl Display for WfcError {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            WfcError::InvalidWeightAmount { expected, was } => write!(
                f,
                "{expected} weights were expected, but {was} were provided!"
            ),
            WfcError::CellHasZeroEntropy(pos) => write!(
                f,
                "The position {pos:?} has zero entropy and cannot be collapsed!"
            ),
        }
    }
}
