use pad::position::Position;

pub (crate) struct TileConstraints<T> {
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
    pub(crate) fn add_constraint(&mut self, constraint: impl Constraint<T> + 'static) {
        self.constraints.push(Box::new(constraint));
    }

    pub fn get_possible_indices<const C: usize>(
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

// todo this interface needs some rework:
//  - I need to provide a reference to the board
//  - for convenience, the tiles should be provided as references to the actual tiles, not the indexes
//  - if I have the board, I dont need the tile slice
//  - if I have the board, I might no longer need the collapsed neighbour

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
