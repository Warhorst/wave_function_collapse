use std::hash::{DefaultHasher, Hash, Hasher};
use pad::*;
use rand::distributions::WeightedIndex;
use rand::prelude::{Distribution, StdRng};
use rand::{Rng, SeedableRng};

// todo
//  - Edge Colors, so no neighbour constraints have to be set manually
//  - rotating tiles (this might be possible using a trait)
//  - weights
//  - MAYBE allowing more complex constraints (this might require fast access to all the collapsed/not collapsed tiles)
//  - MAYBE Providing an iterator, so I can watch the wfc work

pub struct WaveFunctionCollapse<const C: usize, T: Clone> {
    board: Board<C>,
    tiles: Vec<T>,
    tile_constraints: TileConstraints<T>,
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

    pub fn with_constraint(mut self, constraint: impl Constraint<T> + 'static) -> Self {
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

    fn choose_next_index(&mut self, possible_indices: &[usize]) -> usize {
        match self.weights {
            Some(weights) => {
                let mut possible_weights = [0.0; C];

                for (i, index) in possible_indices.iter().enumerate() {
                    possible_weights[i] = weights[*index];
                }

                let possible_weights = &possible_weights[0..possible_indices.len()];

                self.random.choose_weighted(possible_weights, possible_indices)
            }
            None => self.random.choose(possible_indices)
        }
    }
}

struct TileConstraints<T> {
    constraints: Vec<Box<dyn Constraint<T>>>
}

impl<T> Default for TileConstraints<T> {
    fn default() -> Self {
        TileConstraints {
            constraints: vec![]
        }
    }
}

impl<T> TileConstraints<T> {
    fn add_constraint(&mut self, constraint: impl Constraint<T> + 'static) {
        self.constraints.push(Box::new(constraint));
    }

    fn get_possible_indices<const C: usize>(
        &self,
        (possible_tiles, possible_tiles_pos): (&[usize], Position),
        collapsed_neighbour: (usize, Position),
        tiles: &[T],
    ) -> [usize; C] {
        let mut indices = [usize::MAX; C];

        let iter = possible_tiles
            .iter()
            .filter(|index| self.constraints
                .iter()
                .all(|c| c.valid(
                    (**index, possible_tiles_pos),
                    collapsed_neighbour,
                    tiles
                ))
            );

        for (i, index) in iter.enumerate() {
            indices[i] = *index
        }

        indices
    }
}

// todo Things I want as possible constraints:
//  - The classic color constraint
//  - The borders (or some other position) must be a specific tile
//  - Simple neighbourhood, just like currently
//  - Directional neighbour restrictions, like the original coast example
//  I could store all the collapsed positions of the board in a vector and give a reference to them
//  into the constraints, so I could create even more complex constraints + This would make a WFC iterator possible

pub trait Constraint<T> {
    /// Check for a specific tile and its given collapsed neighbour if it would be a valid
    /// remaining choice.
    /// * `tile_to_check` - The tile index and its position which I want to know would be valid according to this constraint
    /// * `collapsed_neighbour` - The neighbour tile index and its position which just collapsed
    /// * `tiles` - All actual possible tiles. This can be used to map the tile index to the actual tile for more complex logic
    fn valid(
        &self,
        tile_to_check: (usize, Position),
        collapsed_neighbour: (usize, Position),
        tiles: &[T],
    ) -> bool;
}

/// A [Constraint] which defines what tiles can be neighboured to each other.
pub struct PossibleNeighbours {
    allowed_neighbours: Vec<(usize, usize)>
}

impl PossibleNeighbours {
    pub fn new<T: PartialEq>(
        allowed_neighbours: impl IntoIterator<Item = (T, T)>,
        all_tiles: &[T]
    ) -> Self {
        let get_index = |tile: T| all_tiles.iter().position(|t| *t == tile).expect("The tile should be in the possible tiles");

        PossibleNeighbours {
            allowed_neighbours: allowed_neighbours
                .into_iter()
                .map(|(t0, t1)| (get_index(t0), get_index(t1)))
                .collect()
        }
    }
}

impl<T> Constraint<T> for PossibleNeighbours {
    fn valid(
        &self,
        (tile, _): (usize, Position),
        (neighbour, _): (usize, Position),
        _tiles: &[T]
    ) -> bool {
        self.allowed_neighbours
            .iter()
            .any(|(i0, i1)| tile == *i0 && neighbour == *i1 || tile == *i1 && neighbour == *i0)
    }
}

/// Contains the current state of the WFC with all the cells at their respective positions.
/// The WFC is done if all positions on the board are collapsed.
pub struct Board<const C: usize> {
    /// Width of the board
    width: usize,
    /// Height of the board
    height: usize,
    /// The cells of the board, which tell what tiles are still possible
    cells: Vec<Cell<C>>,
    /// Caches the amount of not already collapsed positions to quickly check
    /// if the whole board is collapsed
    not_collapsed_positions: usize,
    /// Caches the best (lowest entropy) next cell to collapse which might was found when collapsing
    /// If no min next cell is known, the whole board has to be searched for the lowest entropy position
    min_non_collapsed: Option<(Position, Cell<C>)>
}

impl<const C: usize> Board<C> {
    fn new(
        width: usize,
        height: usize,
        num_tiles: usize,
    ) -> Self {
        let cells = (0..(width * height))
            .into_iter()
            .map(|_| Cell::<C>::new(num_tiles))
            .collect();

        Board {
            width,
            height,
            cells,
            not_collapsed_positions: width * height,
            min_non_collapsed: None
        }
    }

    /// tells if the full board is collapsed
    fn collapsed(&self) -> bool {
        self.not_collapsed_positions == 0
    }

    /// Collapse the cell at the given position and set its tile index to the given one
    fn collapse_position(&mut self, position: Position, index: usize) {
        self.get_cell_mut(position).collapse(index);
        self.not_collapsed_positions -= 1;
    }

    /// Adapt all the cardinal neighbours of the given collapsed position
    /// This works recursive, so a collapsed neighbour will propagate the collapse to all
    /// its neighbours
    fn propagate<T>(
        &mut self,
        collapsed_position: Position,
        tile_constraints: &TileConstraints<T>,
        all_tiles: &[T]
    ) {
        let collapsed_tile = self.get_cell(collapsed_position).get_collapsed_index();

        for pos in collapsed_position.cardinal_neighbours() {
            // cover the edges of the board
            if !self.pos_in_bounds(pos) {
                continue;
            }

            // ignore collapsed cells
            if self.get_cell(pos).is_collapsed() {
                continue;
            }

            let cell = self.get_cell(pos);
            let cell_indices = cell.get_possible_indices();
            let new_indices = tile_constraints.get_possible_indices::<C>(
                (&cell_indices, pos),
                (collapsed_tile, collapsed_position),
                all_tiles
            );

            if new_indices[0] == usize::MAX {
                // todo better error handling
                panic!("No new indices could be determined")
            }

            self.get_cell_mut(pos).set_indices(new_indices);

            let cell = self.get_cell(pos);

            if cell.is_collapsed() {
                self.not_collapsed_positions -= 1;
                self.propagate(pos, tile_constraints, all_tiles)
            } else {
                // update the cache with the best next cell
                self.min_non_collapsed = Some(match self.min_non_collapsed {
                    // the current cell now has a lower entropy than the current min cell, overwrite
                    Some((_, c)) if cell.entropy < c.entropy => (pos, *cell),
                    // the current cell is not better, keep the same vale
                    Some((p, c)) => (p, c),
                    // no min value is set, use the current cell
                    None => (pos, *cell)
                });
            }
        }
    }

    fn pos_in_bounds(&self, pos: Position) -> bool {
        pos.x >= 0
            && pos.y >= 0
            && pos.x < self.width as isize
            && pos.y < self.height as isize
    }

    fn get_min_entropy_position(&self) -> (Position, Cell<C>) {
        p!(0, 0)
            .iter_to(p!(self.width - 1, self.height - 1))
            .map(|pos| (pos, *self.get_cell(pos)))
            .filter(|(_, cell)| cell.entropy > 1)
            .min_by(|(_, cell_a), (_, cell_b)| cell_a.entropy.cmp(&cell_b.entropy))
            .expect("At least one not collapsed cell should exist")
    }

    fn get_cell(&self, pos: Position) -> &Cell<C> {
        self.cells.get(self.width * pos.y as usize + pos.x as usize).expect(format!("A cell at position {:?} should exist", pos).as_str())
    }

    fn get_cell_mut(&mut self, pos: Position) -> &mut Cell<C> {
        self.cells.get_mut(self.width * pos.y as usize + pos.x as usize).expect(format!("A cell at position {:?} should exist", pos).as_str())
    }

    fn get_collapsed_indices(&self) -> impl Iterator<Item=(Position, usize)> + '_ {
        p!(0, 0)
            .iter_to(p!(self.width - 1, self.height - 1))
            .map(|pos| (pos, self.get_cell(pos).get_collapsed_index()))
    }
}

// todo maybe provide a dynamic variant which works with vectors internally
/// Holds the possible tile indices which are possible by a specific position, known by the board.
/// The const generic C represents the maximum amount of tiles possible and therefore the capacity of
/// the cell.
#[derive(Clone, Copy)]
pub struct Cell<const C: usize> {
    /// the current entropy of the cell, or the amount of still possible tiles
    entropy: usize,
    /// the indices of all tiles which are currently possible in this cell; only the entries until the self.entropy index are used
    tile_indices: [usize; C],
}

impl<const C: usize> Cell<C> {
    fn new(num_tiles: usize) -> Self {
        let mut tile_indices = [0; C];

        for i in 0..num_tiles {
            tile_indices[i] = i;
        }

        Cell {
            entropy: num_tiles,
            tile_indices,
        }
    }

    fn get_possible_indices(&self) -> &[usize] {
        &self.tile_indices[0..self.entropy]
    }

    fn collapse(&mut self, index: usize) {
        self.tile_indices[0] = index;
        self.entropy = 1;
    }

    fn set_indices(&mut self, indices: impl IntoIterator<Item=usize>) {
        let mut entropy = 0;

        for (i, index) in indices.into_iter().enumerate() {
            if index == usize::MAX {
                // usize::MAX is used as a placeholder. The first occurrence
                // of it tells that there are no more indices to consider
                break;
            }

            self.tile_indices[i] = index;
            entropy += 1;
        }

        self.entropy = entropy
    }

    fn get_collapsed_index(&self) -> usize {
        self.tile_indices[0]
    }

    fn is_collapsed(&self) -> bool {
        self.entropy == 1
    }
}

/// Provides random numbers to the WFC.
struct Random {
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
