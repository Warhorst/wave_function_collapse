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
    tile_weights: [f32; C]
}

impl<const C: usize> Cell<C> {
    pub fn new(num_tiles: usize, weights: &[f32]) -> Self {
        let mut tile_indices = [0; C];
        let mut tile_weights = [0.0; C];

        for i in 0..num_tiles {
            tile_indices[i] = i as u8;
            tile_weights[i] = weights[i]
        }

        Cell {
            entropy: num_tiles as u8,
            tile_indices,
            tile_weights
        }
    }

    pub fn get_possible_indices(&self) -> &[u8] {
        &self.tile_indices[0..self.entropy as usize]
    }

    pub fn get_tile_weights(&self) -> &[f32] {
        &self.tile_weights[0..self.entropy as usize]
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

    pub fn set_weights(&mut self, weights: impl IntoIterator<Item=f32>) {
        for (i, weight) in weights.into_iter().enumerate() {
            if weight == f32::MAX {
                // f32::MAX is used as a placeholder. The first occurrence
                // of it tells that there are no more indices to consider
                break;
            }

            self.tile_weights[i] = weight
        }
    }

    pub fn get_collapsed_index(&self) -> u8 {
        self.tile_indices[0]
    }

    pub fn is_collapsed(&self) -> bool {
        self.entropy == 1
    }
}
