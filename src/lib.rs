mod board;
mod cell;
pub mod constraints;
mod random;

use crate::board::Board;
use crate::constraints::{Constraint, TileConstraints};
use crate::random::Random;
use pad::position::Position;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

// todo
//  - Edge Colors, so no neighbour constraints have to be set manually
//  - rotating tiles (this might be possible using a trait)
//  - MAYBE Providing an iterator, so I can watch the wfc work

/// The builder for a [Wfc].
pub struct WfcBuilder<const C: usize, T: Clone> {
    width: usize,
    height: usize,
    tiles: Vec<T>,
    tile_constraints: TileConstraints<T>,
    random: Random,
    weights: [f32; C],
}

impl<const C: usize, T> WfcBuilder<C, T>
where
    T: Clone,
{
    pub fn new(
        width: usize,
        height: usize,
        tiles: Vec<T>,
    ) -> Self {
        WfcBuilder {
            width,
            height,
            tiles,
            tile_constraints: TileConstraints::default(),
            random: Random::new(),
            weights: [1.0; C],
        }
    }

    /// Set a custom seed for the WFC
    pub fn with_seed(
        mut self,
        seed: impl Hash,
    ) -> Self {
        self.random = Random::from_seed(seed);
        self
    }

    pub fn with_weights(
        mut self,
        tile_weights: impl IntoIterator<Item = f32>,
    ) -> Self {
        let mut weights = [0.0; C];

        for (i, weight) in tile_weights.into_iter().enumerate() {
            weights[i] = weight;
        }

        self.weights = weights;
        self
    }

    pub fn with_constraint(
        mut self,
        constraint: impl Constraint<T> + 'static,
    ) -> Self {
        self.tile_constraints.add_constraint(constraint);
        self
    }

    /// Validate the input and create a [Wfc].
    pub fn build(self) -> Result<Wfc<C, T>, WfcError> {
        if self.tiles.len() > C {
            return Err(WfcError::TooManyTiles {
                max: C,
                was: self.tiles.len(),
            });
        }

        let board = Board::<C>::new(self.width, self.height, self.tiles.len(), self.weights);

        Ok(Wfc {
            board,
            tiles: self.tiles,
            tile_constraints: self.tile_constraints,
            random: self.random,
            weights: self.weights,
        })
    }
}

/// The struct which performs the wave function collapse.
pub struct Wfc<const C: usize, T: Clone> {
    board: Board<C>,
    tiles: Vec<T>,
    tile_constraints: TileConstraints<T>,
    random: Random,
    weights: [f32; C],
}

impl<const C: usize, T> Wfc<C, T>
where
    T: Clone,
{
    /// Collapse the WFC until no more tiles are not collapsed.
    pub fn collapse(mut self) -> Result<Vec<(Position, T)>, WfcError> {
        while !self.board.collapsed() {
            let (pos, cell) = self.board.get_min_entropy_position();

            let possible_indices = cell.get_possible_indices();
            let weights = cell.get_tile_weights();
            let index = self.choose_next_index(possible_indices, weights);
            self.board.collapse_position(pos, index);
            self.board
                .propagate(pos, &self.tile_constraints, &self.weights, &self.tiles)?;
        }

        Ok(self
            .board
            .get_collapsed_indices()
            .map(|(pos, index)| (pos, self.tiles[index].clone()))
            .collect())
    }

    fn choose_next_index(
        &mut self,
        possible_indices: &[u8],
        tile_weights: &[f32],
    ) -> u8 {
        self.random.choose_weighted(tile_weights, possible_indices)
    }
}

impl<const C: usize, T> Wfc<C, T>
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
                .propagate(pos, &self.tile_constraints, &self.weights, &self.tiles)?
        }

        Ok(())
    }
}

/// An error for the wave function collapse, which occurs due to configuration errors
/// or runtime errors.
#[derive(Debug)]
pub enum WfcError {
    /// More tile types are provided than supported by the WFC
    TooManyTiles { max: usize, was: usize },
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
            WfcError::TooManyTiles { max, was } => write!(
                f,
                "{was} tiles where provided, but only {max} are supported!"
            ),
            WfcError::CellHasZeroEntropy(pos) => write!(
                f,
                "The position {pos:?} has zero entropy and cannot be collapsed!"
            ),
        }
    }
}
