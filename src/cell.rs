use bitarray::BitArray;

// todo maybe provide a dynamic variant which works with vectors internally
/// Holds the possible tile indices which are possible by a specific position, known by the board.
/// The const generic C represents the maximum amount of tiles possible and therefore the capacity of
/// the cell.
#[derive(Clone, Copy)]
pub struct Cell {
    /// the current entropy of the cell, or the amount of still possible tiles
    pub entropy: usize,
    tile_indices: BitArray<u64>
}

// todo replace this with the bit array, I might need to move the index array (and maybe the weight array) to a higher level (aka the wfc itself),
//  so I can set the currently accessed cell still as a slice

impl Cell {
    pub fn new(num_tiles: usize) -> Self {
        Cell {
            entropy: num_tiles,
            tile_indices: BitArray::from_indices(0..num_tiles),
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
