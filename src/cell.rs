use std::collections;

use bitarray::BitArray;

use crate::MAX_NUM_TILES;

pub type Cell<const C: usize> = Cell_BA<C>;

#[derive(Clone, Copy)]
pub struct Cell_BA<const C: usize> {
    pub entropy: u8,
    max_entropy: u8,
    tile_indices: BitArray<u128>,
    collapsed_index: Option<u8>,
}

impl<const C: usize> Cell_BA<C> {
    pub fn new(num_tiles: usize) -> Self {
        let mut tile_indices = BitArray::new(0);

        for i in 0..num_tiles {
            tile_indices.set(i as u8, true);
        }

        Cell_BA {
            entropy: num_tiles as u8,
            max_entropy: num_tiles as u8,
            tile_indices,
            collapsed_index: None,
        }
    }

    pub fn get_possible_indices(&self) -> PossibleIndices {
        match self.collapsed_index {
            Some(i) => PossibleIndices::new([i]),
            None => PossibleIndices::new(self.tile_indices.ones().take(self.max_entropy as usize)),
        }
    }

    pub fn collapse(
        &mut self,
        index: u8,
    ) {
        self.collapsed_index = Some(index);
        self.entropy = 1;
    }

    pub fn set_indices(
        &mut self,
        indices: impl IntoIterator<Item = u8>,
    ) {
        let mut entropy = 0;
        let mut tile_indices = BitArray::new(0);

        for index in indices {
            if index == u8::MAX {
                // usize::MAX is used as a placeholder. The first occurrence
                // of it tells that there are no more indices to consider
                break;
            }

            tile_indices.set(index, true);
            entropy += 1;
        }

        self.entropy = entropy;
        self.tile_indices = tile_indices
    }

    pub fn get_collapsed_index(&self) -> u8 {
        self.collapsed_index.unwrap()
    }

    pub fn is_collapsed(&self) -> bool {
        self.entropy == 1
    }
}

// TODO maybe provide a dynamic variant which works with vectors internally (Or a fixed array and the amount of tiles, an I just use slices)
/// Holds the possible tile indices which are possible by a specific position, known by the board.
/// The const generic C represents the maximum amount of tiles possible and therefore the capacity of
/// the cell.
#[derive(Clone, Copy)]
pub struct Cell_CG<const C: usize> {
    /// the current entropy of the cell, or the amount of still possible tiles
    pub entropy: u8,
    /// the indices of all tiles which are currently possible in this cell; only the entries until the self.entropy index are used
    tile_indices: [u8; C],
}

impl<const C: usize> Cell_CG<C> {
    pub fn new(num_tiles: usize) -> Self {
        let mut tile_indices = [0; C];

        for (i, index) in tile_indices.iter_mut().enumerate() {
            *index = i as u8
        }

        Cell_CG {
            entropy: num_tiles as u8,
            tile_indices,
        }
    }

    // TODO this should not return a slice. Maybe return an iter and create arrays where needed at the call side
    pub fn get_possible_indices(&self) -> PossibleIndices {
        PossibleIndices::new(self.tile_indices[0..self.entropy as usize].iter().copied())
    }

    pub fn collapse(
        &mut self,
        index: u8,
    ) {
        self.tile_indices[0] = index;
        self.entropy = 1;
    }

    pub fn set_indices(
        &mut self,
        indices: impl IntoIterator<Item = u8>,
    ) {
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PossibleIndices {
    possible_indices: [u8; MAX_NUM_TILES],
    entropy: u8,
}

impl Default for PossibleIndices {
    fn default() -> Self {
        PossibleIndices {
            possible_indices: [u8::MAX; MAX_NUM_TILES],
            entropy: 0,
        }
    }
}

impl PossibleIndices {
    fn new(iter: impl IntoIterator<Item = u8>) -> Self {
        let mut entropy = 0u8;
        let mut possible_indices = [u8::MAX; MAX_NUM_TILES];

        for i in iter {
            possible_indices[entropy as usize] = i;
            entropy += 1;
        }

        PossibleIndices {
            possible_indices,
            entropy,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = u8> {
        self.possible_indices
            .into_iter()
            .take_while(|i| *i != u8::MAX)
    }

    pub fn get(
        &self,
        index: usize,
    ) -> u8 {
        // TODO bounds check and stuff
        self.possible_indices[index]
    }

    pub fn push(
        &mut self,
        val: u8,
    ) {
        self.possible_indices[self.entropy as usize] = val;
        self.entropy += 1
    }

    pub fn set(
        &mut self,
        index: usize,
        val: u8,
    ) {
        self.possible_indices[index] = val
    }

    pub fn is_empty(&self) -> bool {
        self.entropy == 0
    }
}
