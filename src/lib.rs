pub mod board;
pub mod constraints;
mod random;
mod cell;

use crate::board::Board;
use crate::random::Random;
use pad::position::Position;
use std::hash::Hash;
use crate::constraints::{Constraint, TileConstraints};
// todo
//  - Edge Colors, so no neighbour constraints have to be set manually
//  - rotating tiles (this might be possible using a trait)
//  - weights
//  - MAYBE allowing more complex constraints (this might require fast access to all the collapsed/not collapsed tiles)
//  - MAYBE Providing an iterator, so I can watch the wfc work

// todo propagation queue idea: The Board holds a queue of positions which need propagation.
//  In every iteration, if something is in the queue, it will propagate it first
//  ... and then how do I use this to create an iterator? The propagation might not yield a newly collapsed position. Maybe
//  I should have a collapsed queue instead

pub struct WaveFunctionCollapse<const C: usize, T: Clone> {
    board: Board<C>,
    tiles: Vec<T>,
    tile_constraints: TileConstraints<C, T>,
    random: Random,
    weights: Option<[f32; C]>,
}

impl<const C: usize, T: Clone> WaveFunctionCollapse<C, T> {
    pub fn new(
        width: usize,
        height: usize,
        tiles: Vec<T>,
    ) -> Self {
        let board = Board::<C>::new(width, height, tiles.len());

        WaveFunctionCollapse {
            board,
            tiles,
            tile_constraints: TileConstraints::default(),
            random: Random::new(),
            weights: None,
        }
    }

    /// Set a custom seed for the WFC
    pub fn with_seed(mut self, seed: impl Hash) -> Self {
        self.random = Random::from_seed(seed);
        self
    }

    pub fn with_weights(mut self, tile_weights: impl IntoIterator<Item=f32>) -> Self {
        let mut weights = [0.0; C];

        for (i, weight) in tile_weights.into_iter().enumerate() {
            weights[i] = weight;
        }

        self.weights = Some(weights);
        self
    }

    pub fn with_constraint(mut self, constraint: impl Constraint<C, T> + 'static) -> Self {
        self.tile_constraints.add_constraint(constraint);
        self
    }

    /// Attempting to perform a wfc with more tiles than C results in a crash
    pub fn collapse(mut self) -> Vec<(Position, T)> {
        while !self.board.collapsed() {
            let (pos, cell) = match self.board.min_non_collapsed {
                // Some best next cell is already known. Use it and clear the cache
                Some(_) => self.board.min_non_collapsed.take().unwrap(),
                // No best next cell is known, search the whole board for the one with the lowest entropy
                None => self.board.get_min_entropy_position()
            };

            let possible_indices = cell.get_possible_indices();
            let index = self.choose_next_index(possible_indices);
            self.board.collapse_position(pos, index);
            self.board.propagate(pos, &self.tile_constraints, &self.tiles);
        }

        self.board
            .get_collapsed_indices()
            .map(|(pos, index)| (pos, self.tiles[index].clone()))
            .collect()
    }

    fn choose_next_index(&mut self, possible_indices: &[u8]) -> u8 {
        match self.weights {
            Some(weights) => {
                let mut possible_weights = [0.0; C];

                for (i, index) in possible_indices.iter().enumerate() {
                    possible_weights[i] = weights[*index as usize];
                }

                let possible_weights = &possible_weights[0..possible_indices.len()];

                self.random.choose_weighted(possible_weights, possible_indices)
            }
            None => self.random.choose(possible_indices)
        }
    }
}
