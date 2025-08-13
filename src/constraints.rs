use pad::position::Position;

/// Wrapper around the list of [Constraint]s which are configured in the [crate::Wfc].
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

    /// Determine the next values for a cell based on the set constraints.
    /// * `(cell_tiles, cell_position)` - The currently possible tiles and the position on the board of the cell to check.
    /// * `neighbours_iter` - An iterator over all the neighbours around the cell to check.
    /// * `all_weights` - The weights in the wfc settings which are used as the base to determine the next cell weights.
    /// * `tiles` - A slice of all tiles set in the wfc. Used to access a reference of a tile at a given index.
    pub fn update_cell<'a, const C: usize>(
        &self,
        (cell_tiles, cell_position): (&'a [u8], Position),
        neighbours_iter: impl IntoIterator<Item=(&'a [u8], Position)>,
        all_weights: &[f32],
        tiles: &[T]
    ) -> CellUpdate<C> {
        // collect all the neighbours in an array
        let mut neighbours: [(&[u8], Position); 4] = [(&[], Position::default()); 4];
        let mut num_neighbours = 0;

        for (i, (possible_tiles, pos)) in neighbours_iter.into_iter().enumerate() {
            neighbours[i] = (possible_tiles, pos);
            num_neighbours += 1;
        }

        // initialize the new indices and weights
        let mut new_indices = [u8::MAX; C];
        let mut new_weights = [f32::MAX; C];
        let mut new_entropy = 0;

        // outer loop which iterates over all the currently possible tile indices
        // in the cell (also going wild by using a loop tag)
        'outer: for (i, index) in cell_tiles.iter().enumerate() {
            // the new weight for the currently checked tile index
            let mut weight = all_weights[i];

            // inner loop to check if all constraints are fulfilled
            for c in self.constraints.iter() {
                let valid = c.valid(
                    (*index, cell_position),
                    &neighbours[0..num_neighbours],
                    tiles,
                );

                match valid {
                    // the constraint was fulfilled, so update the current weight with the modifier
                    // from the constraint
                    Some(weight_modifier) => weight *= weight_modifier,
                    // The constraint was not fulfilled, so go to the next iteration (tile index) of the other loop.
                    // This will therefore skip updating the new indices, which means this tile index will not
                    // be used.
                    None => continue 'outer
                }
            }

            // Update the new indices and weight. As the entropy
            // increments, it can also be used as the index here
            new_indices[new_entropy as usize] = *index;
            new_weights[new_entropy as usize] = weight;
            new_entropy += 1;
        }

        CellUpdate {
            new_indices,
            new_weights,
            new_entropy
        }
    }
}

/// Contains the next values a [crate::cell::Cell] will have.
pub (crate) struct CellUpdate<const C: usize> {
    new_indices: [u8; C],
    new_weights: [f32; C],
    new_entropy: u8
}

impl<const C: usize> CellUpdate<C> {
    pub fn new_indices(&self) -> &[u8] {
        &self.new_indices[0..self.new_entropy as usize]
    }

    pub fn new_weights(&self) -> &[f32] {
        &self.new_weights[0..self.new_entropy as usize]
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
