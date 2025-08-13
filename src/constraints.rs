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

    pub fn get_possible_indices<'a, const C: usize>(
        &self,
        (possible_tiles, possible_tiles_pos): (&'a [u8], Position),
        neighbours_iter: impl IntoIterator<Item=(&'a [u8], Position)>,
        all_weights: &[f32],
        tiles: &[T]
    ) -> PossibleIndices<C> {
        let mut neighbours: [(&[u8], Position); 4] = [(&[], Position::default()); 4];
        let mut num_neighbours = 0;

        for (i, (possible_tiles, pos)) in neighbours_iter.into_iter().enumerate() {
            neighbours[i] = (possible_tiles, pos);
            num_neighbours += 1;
        }

        let mut indices = [u8::MAX; C];
        let mut weights = [f32::MAX; C];
        let mut entropy = 0;

        'outer: for (i, index) in possible_tiles.iter().enumerate() {
            let mut weight = all_weights[i];

            for c in self.constraints.iter() {
                let valid = c.valid(
                    (*index, possible_tiles_pos),
                    &neighbours[0..num_neighbours],
                    tiles,
                );

                // todo add some comments and update the doc

                match valid {
                    Some(weight_modifier) => weight *= weight_modifier,
                    None => continue 'outer
                }
            }

            indices[entropy as usize] = *index;
            weights[entropy as usize] = weight;
            entropy += 1;
        }

        PossibleIndices {
            indices,
            weights,
            entropy
        }
    }
}

pub (crate) struct PossibleIndices<const C: usize> {
    indices: [u8; C],
    weights: [f32; C],
    entropy: u8
}

impl<const C: usize> PossibleIndices<C> {
    pub fn get_indices(&self) -> &[u8] {
        &self.indices[0..self.entropy as usize]
    }

    pub fn get_weights(&self) -> &[f32] {
        &self.weights[0..self.entropy as usize]
    }
}

// todo Things I want as possible constraints:
//  - The classic color constraint
//  - Simple neighbourhood, just like currently
//  - Directional neighbour restrictions, like the original coast example

// todo the Constraint should not return a bool, but an optional weight modifier.
//  - Every cell should hold the weights for their tiles
//  - If the modifier is not None, the weight for that tile gets set accordingly
//  - The initial weight for every tile is the setting provided to the WFC
//  This allows for a "Bias Constraint", where based on the neighbours, some tile should have
//  way higher or lower probability to get picked

pub trait Constraint<T> {
    /// Check for a specific tile and its given collapsed neighbour if it would be a valid
    /// remaining choice.
    /// Only the surroundings of the tile to check are taken into consideration for the validation. Some kind
    /// of global constraint (which for example could access every cell in the current state) can easily lead
    /// to dead ends. The parameters and capabilities are therefore, by design, sparse.
    ///
    /// # Parameters
    /// * `tile_to_check` - The tile index and its position which I want to know would be valid according to this constraint
    /// * `neighbours` - The possible tiles and their positions of all neighbours of the tile to check. 
    /// * `tiles` - All actual possible tiles. This can be used to map the tile index to the actual tile for more complex logic
    /// todo fix doc
    /// Returns true if the tile to check could be placed on this position, according to this constraint.
    fn valid(
        &self,
        tile_to_check: (u8, Position),
        neighbours: &[(&[u8], Position)],
        tiles: &[T]
    ) -> Option<f32>;
}

/// A [Constraint] which defines what tiles can be neighboured to each other.
pub struct PossibleNeighbours {
    allowed_neighbours: Vec<(u8, u8)>
}

impl PossibleNeighbours {
    pub fn new<T: PartialEq>(
        allowed_neighbours: impl IntoIterator<Item = (T, T)>,
        all_tiles: &[T]
    ) -> Self {
        let get_index = |tile: T| all_tiles.iter().position(|t| *t == tile).expect("The tile should be in the possible tiles") as u8;

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
        (tile, _): (u8, Position),
        neighbours: &[(&[u8], Position)],
        _tiles: &[T]
    ) -> Option<f32> {
        // for every neighbour, one possible tile must match with the current tile
        let valid = neighbours
            .into_iter()
            .all(|(nts, _)| nts
                .iter()
                .any(|nt| self.allowed_neighbours.contains(&(tile, *nt)) || self.allowed_neighbours.contains(&(*nt, tile)))
            );

        if valid {
            Some(1.0)
        } else {
            None
        }
    }
}
