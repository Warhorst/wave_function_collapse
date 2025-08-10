// todo maybe provide a dynamic variant which works with vectors internally
/// Holds the possible tile indices which are possible by a specific position, known by the board.
/// The const generic C represents the maximum amount of tiles possible and therefore the capacity of
/// the cell.
#[derive(Clone, Copy)]
pub struct Cell<const C: usize> {
    /// the current entropy of the cell, or the amount of still possible tiles
    pub entropy: u8,
    /// the indices of all tiles which are currently possible in this cell; only the entries until the self.entropy index are used
    tile_indices: [u8; C],
}

impl<const C: usize> Cell<C> {
    pub fn new(num_tiles: usize) -> Self {
        let mut tile_indices = [0; C];

        for i in 0..num_tiles {
            tile_indices[i] = i as u8;
        }

        Cell {
            entropy: num_tiles as u8,
            tile_indices,
        }
    }

    pub fn get_possible_indices(&self) -> &[u8] {
        &self.tile_indices[0..self.entropy as usize]
    }

    pub fn collapse(&mut self, index: u8) {
        self.tile_indices[0] = index;
        self.entropy = 1;
    }

    pub fn set_indices(&mut self, indices: impl IntoIterator<Item=u8>) {
        let mut entropy = 0;

        for (i, index) in indices.into_iter().enumerate() {
            if index == u8::MAX {
                // usize::MAX is used as a placeholder. The first occurrence
                // of it tells that there are no more indices to consider
                break;
            }

            self.tile_indices[i] = index;
            entropy += 1;
        }

        self.entropy = entropy
    }

    pub fn get_collapsed_index(&self) -> u8 {
        self.tile_indices[0]
    }

    pub fn is_collapsed(&self) -> bool {
        self.entropy == 1
    }
}
