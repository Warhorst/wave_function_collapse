// todo maybe provide a dynamic variant which works with vectors internally
/// Holds the possible tile indices which are possible by a specific position, known by the board.
/// The const generic C represents the maximum amount of tiles possible and therefore the capacity of
/// the cell.
#[derive(Clone, Copy)]
pub struct Cell<const C: usize> {
    /// the current entropy of the cell, or the amount of still possible tiles
    pub entropy: usize,
    /// the indices of all tiles which are currently possible in this cell; only the entries until the self.entropy index are used
    tile_indices: [usize; C],
}

impl<const C: usize> Cell<C> {
    pub fn new(num_tiles: usize) -> Self {
        let mut tile_indices = [0; C];

        for i in 0..num_tiles {
            tile_indices[i] = i;
        }

        Cell {
            entropy: num_tiles,
            tile_indices,
        }
    }

    pub fn get_possible_indices(&self) -> &[usize] {
        &self.tile_indices[0..self.entropy]
    }

    pub fn collapse(&mut self, index: usize) {
        self.tile_indices[0] = index;
        self.entropy = 1;
    }

    pub fn set_indices(&mut self, indices: impl IntoIterator<Item=usize>) {
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

    pub fn get_collapsed_index(&self) -> usize {
        self.tile_indices[0]
    }

    pub fn is_collapsed(&self) -> bool {
        self.entropy == 1
    }
}
