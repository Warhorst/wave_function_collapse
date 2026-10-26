#![deny(rustdoc::broken_intra_doc_links)]

use crate::{
    board::{Board, BoardError},
    cell::{Cell, PossibleIndices},
    constraints::{Constraint, Constraints},
    prob_mod::{ProbabilityModifier, ProbabilityModifiers},
    random::Random,
    tile_filter::{TileFilter, TileFilters},
};
use pad::{p, position::Position};
use std::{hash::Hash, marker::PhantomData};
use thiserror::Error;

pub mod board;
pub mod cell;
pub mod constraints;
pub mod prob_mod;
mod random;
pub mod tile_filter;

/// Performs the wave function collapse.
pub struct Wfc<T: Clone, C: Cell> {
    board: Board<C>,
    tiles: Vec<T>,
    weights: Vec<f32>,
    constraints: Constraints<T>,
    probability_modifiers: ProbabilityModifiers<T>,
    random: Random,
    collapsed_positions: Vec<(Position, T)>,
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
    pub fn new_builder(
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
            tile_filters: TileFilters::new(num_tiles),
            random: Random::new(),
            _cell_type: PhantomData,
        }
    }

    fn new(
        board: Board<C>,
        tiles: Vec<T>,
        weights: Vec<f32>,
        constraints: Constraints<T>,
        probability_modifiers: ProbabilityModifiers<T>,
        random: Random,
    ) -> Self {
        let width = board.width;
        let height = board.height;

        Wfc {
            board,
            tiles,
            weights,
            constraints,
            probability_modifiers,
            random,
            collapsed_positions: Vec::with_capacity(width * height),
        }
    }

    /// Collapse the [`Wfc`] until no more tiles are not collapsed.
    pub fn collapse(mut self) -> Result<Vec<(Position, T)>, WfcError> {
        while !self.board.collapsed() {
            let (pos, cell) = self.board.get_min_entropy_position();

            let modified_weights = self.probability_modifiers.modify_probability(
                pos,
                &self.weights,
                self.board.width,
                self.board.height,
                &self.tiles,
                &self.collapsed_positions,
            );

            let possible_indices = cell.get_possible_indices();
            let weights = possible_indices
                .iter()
                .map(|i| modified_weights[i as usize]);
            let index = Self::choose_next_index(&mut self.random, possible_indices, weights);
            let collapsed_positions =
                self.board
                    .collapse_position(pos, index, &mut self.constraints, &self.tiles)?;

            collapsed_positions.iter().for_each(|(pos, i)| {
                self.collapsed_positions
                    .push((*pos, self.tiles[*i as usize].clone()))
            });
        }

        Ok(self.collapsed_positions)
    }

    fn choose_next_index(
        random: &mut Random,
        possible_indices: PossibleIndices,
        tile_weights: impl IntoIterator<Item = f32>,
    ) -> u8 {
        random.choose_weighted(tile_weights, possible_indices)
    }
}

/// The builder for a [`Wfc`].
pub struct WfcBuilder<T: Clone, C: Cell> {
    width: usize,
    height: usize,
    tiles: Vec<T>,
    constraints: Constraints<T>,
    probability_modifiers: ProbabilityModifiers<T>,
    tile_filters: TileFilters<T>,
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

    /// Add a [`TileFilter`] to the [`Wfc`].
    pub fn with_tile_filter(
        mut self,
        tile_filter: impl TileFilter<T> + 'static,
    ) -> Self {
        self.tile_filters.add_tile_filter(tile_filter);
        self
    }

    /// Validate the input and create a [`Wfc`].
    pub fn build(mut self) -> Result<Wfc<T, C>, WfcError> {
        if self.weights.len() != self.tiles.len() {
            return Err(WfcError::InvalidWeightAmount {
                expected: self.tiles.len(),
                was: self.weights.len(),
            });
        }

        let mut board = Board::<C>::new(self.width, self.height, self.tiles.len());

        for pos in p!(0, 0).iter_to(p!(self.width - 1, self.height - 1)) {
            let filtered_tiles =
                self.tile_filters
                    .filter_tiles(pos, self.width, self.height, &self.tiles);

            board.filter_tiles(pos, filtered_tiles)?;
        }

        Ok(Wfc::new(
            board,
            self.tiles,
            self.weights,
            self.constraints,
            self.probability_modifiers,
            self.random,
        ))
    }
}

/// An error for the wave function collapse, which occurs due to configuration errors
/// or runtime errors.
#[derive(Debug, Error)]
pub enum WfcError {
    /// A different amount of weights was set in the WFC than expected.
    #[error("{expected} weights were expected, but {was} were provided!")]
    InvalidWeightAmount { expected: usize, was: usize },
    #[error("{0}")]
    BoardError(#[from] BoardError),
}
