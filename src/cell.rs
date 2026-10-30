use bitarray::BitArray;

/// A [`Cell`] contains the possible tiles (or rather their indices) at a specific [`pad::position::Position`] on the [`crate::board::Board`].
pub trait Cell {
    /// Create a new [`Cell`] from the amount of tiles the wfc will use the the index of the bucket queue bucket the cell is currently in.
    fn new(
        num_tiles: usize,
        bucket_index: usize,
    ) -> Self;

    /// Return the entropy of this [`Cell`], which is the amount of possible tiles.
    fn entropy(&self) -> u8;

    /// Return the current [`PossibleIndices`] of the [`Cell`]. This basically tells
    /// which tiles are still possible.
    fn get_possible_indices(&self) -> PossibleIndices<'_>;

    /// Collapse the [`Cell`] to the given tile index.
    fn collapse(
        &mut self,
        index: u8,
    );

    /// Update the [`Cell`] by setting its possible indices to the given ones.
    fn set_indices(
        &mut self,
        indices: impl IntoIterator<Item = u8>,
    );

    /// Return the last possible index in this [`Cell`].
    ///
    /// Returns [`Option::None`] if this cell does not have an entropy of 1.
    fn get_collapsed_index(&self) -> Option<u8>;

    /// Returs true if this [`Cell`] is collapsed, or else false.
    fn is_collapsed(&self) -> bool;

    /// Returns the index in the bucket of the bucket queue the cell is currently in.
    fn get_bucket_index(&self) -> usize;

    /// Updates the bucket index in this cell.
    fn set_bucket_index(
        &mut self,
        bucket_index: usize,
    );
}

/// Holds the possible tile indices which are possible by a specific position, known by the board.
/// The const generic C represents the maximum amount of tiles possible and therefore the capacity of
/// the cell.
/// This is the fastest [`Cell`] implementation, but also the most limited, as the amount of tiles must be
/// known at compile time
#[derive(Clone, Copy)]
pub struct ConstCell<const C: usize> {
    /// the current entropy of the cell, or the amount of still possible tiles
    pub entropy: u8,
    /// the indices of all tiles which are currently possible in this cell; only the entries until the self.entropy index are used
    tile_indices: [u8; C],
    bucket_index: usize,
}

impl<const C: usize> Cell for ConstCell<C> {
    fn new(
        num_tiles: usize,
        bucket_index: usize,
    ) -> Self {
        let mut tile_indices = [0; C];

        for (i, index) in tile_indices.iter_mut().enumerate() {
            *index = i as u8
        }

        ConstCell {
            entropy: num_tiles as u8,
            tile_indices,
            bucket_index,
        }
    }

    fn entropy(&self) -> u8 {
        self.entropy
    }

    fn get_possible_indices(&self) -> PossibleIndices<'_> {
        PossibleIndices::from_array(&self.tile_indices, self.entropy)
    }

    fn collapse(
        &mut self,
        index: u8,
    ) {
        self.tile_indices[0] = index;
        self.entropy = 1;
    }

    fn set_indices(
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

    fn get_collapsed_index(&self) -> Option<u8> {
        if !self.is_collapsed() {
            return None;
        }

        self.tile_indices.first().copied()
    }

    fn is_collapsed(&self) -> bool {
        self.entropy == 1
    }

    fn get_bucket_index(&self) -> usize {
        self.bucket_index
    }

    fn set_bucket_index(
        &mut self,
        bucket_index: usize,
    ) {
        self.bucket_index = bucket_index
    }
}

/// A [`Cell`] which uses a [`BitArray`] internally to store its possible indices. As [`u128`]
/// is used as the base, this [`Cell`] supports up to 128 different tiles.
#[derive(Clone, Copy)]
pub struct BitCell {
    pub entropy: u8,
    tile_indices: BitArray<u128>,
    bucket_index: usize,
}

impl Cell for BitCell {
    fn new(
        num_tiles: usize,
        bucket_index: usize,
    ) -> Self {
        let mut tile_indices = BitArray::new(0);

        for i in 0..num_tiles {
            tile_indices.set(i as u8, true);
        }

        BitCell {
            entropy: num_tiles as u8,
            tile_indices,
            bucket_index,
        }
    }

    fn entropy(&self) -> u8 {
        self.entropy
    }

    fn get_possible_indices(&self) -> PossibleIndices<'_> {
        PossibleIndices::from_bitarray(self.tile_indices, self.entropy)
    }

    fn collapse(
        &mut self,
        index: u8,
    ) {
        self.tile_indices = BitArray::new(0);
        self.tile_indices.set(index, true);
        self.entropy = 1;
    }

    fn set_indices(
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

    fn get_collapsed_index(&self) -> Option<u8> {
        if !self.is_collapsed() {
            return None;
        }

        self.tile_indices.ones().next()
    }

    fn is_collapsed(&self) -> bool {
        self.entropy == 1
    }

    fn get_bucket_index(&self) -> usize {
        self.bucket_index
    }

    fn set_bucket_index(
        &mut self,
        bucket_index: usize,
    ) {
        self.bucket_index = bucket_index
    }
}

/// A [`Cell`] which supports an arbitrary amount of tiles. This is the most flexible implementation
/// of [`Cell`]
pub struct DynCell {
    indices: Vec<u8>,
    bucket_index: usize,
}

impl Cell for DynCell {
    fn new(
        num_tiles: usize,
        bucket_index: usize,
    ) -> Self {
        let mut indices = Vec::with_capacity(num_tiles);

        for i in 0..num_tiles {
            indices.push(i as u8);
        }

        DynCell {
            indices,
            bucket_index,
        }
    }

    fn entropy(&self) -> u8 {
        self.indices.len() as u8
    }

    fn get_possible_indices(&self) -> PossibleIndices<'_> {
        PossibleIndices::from_array(&self.indices, self.entropy())
    }

    fn collapse(
        &mut self,
        index: u8,
    ) {
        self.indices.clear();
        self.indices.push(index);
    }

    fn set_indices(
        &mut self,
        indices: impl IntoIterator<Item = u8>,
    ) {
        self.indices.clear();
        indices
            .into_iter()
            .for_each(|index| self.indices.push(index));
    }

    fn get_collapsed_index(&self) -> Option<u8> {
        if !self.is_collapsed() {
            return None;
        }

        self.indices.first().copied()
    }

    fn is_collapsed(&self) -> bool {
        self.entropy() == 1
    }

    fn get_bucket_index(&self) -> usize {
        self.bucket_index
    }

    fn set_bucket_index(
        &mut self,
        bucket_index: usize,
    ) {
        self.bucket_index = bucket_index
    }
}

/// The currently possible indices of a [`Cell`].
#[derive(Clone, Copy)]
pub enum PossibleIndices<'a> {
    Array { indices: &'a [u8], entropy: u8 },
    BitArray { array: BitArray<u128>, entropy: u8 },
}

impl<'a> Default for PossibleIndices<'a> {
    // This Default implementation only exists to provide placeholders in an empty array,
    // so the content of the possible indices doesn't really matter
    fn default() -> Self {
        PossibleIndices::from_bitarray(BitArray::new(0), 0)
    }
}

impl<'a> PossibleIndices<'a> {
    fn from_array(
        indices: &'a [u8],
        entropy: u8,
    ) -> Self {
        PossibleIndices::Array { indices, entropy }
    }

    fn from_bitarray(
        array: BitArray<u128>,
        entropy: u8,
    ) -> Self {
        PossibleIndices::BitArray { array, entropy }
    }

    pub fn iter(&self) -> impl Iterator<Item = u8> {
        PossibleIndicesIter::new(*self)
    }

    pub fn get(
        &self,
        index: usize,
    ) -> u8 {
        match self {
            PossibleIndices::Array { indices, .. } => indices[index],
            PossibleIndices::BitArray { array, .. } => array
                .ones()
                .nth(index)
                .expect("The index should be in bounds of the bitarray"),
        }
    }

    pub fn entropy(&self) -> u8 {
        match self {
            PossibleIndices::Array { entropy, .. } => *entropy,
            PossibleIndices::BitArray { entropy, .. } => *entropy,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entropy() == 0
    }
}

pub struct PossibleIndicesIter<'a> {
    counter: u8,
    possible_indices: PossibleIndices<'a>,
}

impl<'a> PossibleIndicesIter<'a> {
    fn new(possible_indices: PossibleIndices<'a>) -> Self {
        PossibleIndicesIter {
            counter: 0,
            possible_indices,
        }
    }
}

impl<'a> Iterator for PossibleIndicesIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter == self.possible_indices.entropy() {
            return None;
        }

        let elem = self.possible_indices.get(self.counter as usize);
        self.counter += 1;
        Some(elem)
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::{BitCell, Cell, ConstCell, DynCell, PossibleIndices};
    use bitarray::BitArray;

    #[test]
    fn collapse_works() {
        collapse_works_(ConstCell::<3>::new(3, 0));
        collapse_works_(BitCell::new(3, 0));
        collapse_works_(DynCell::new(3, 0));
    }

    fn collapse_works_(mut cell: impl Cell) {
        assert_eq!(cell_indices(&cell), vec![0, 1, 2]);

        cell.collapse(1);

        assert_eq!(cell_indices(&cell), vec![1]);
    }

    #[test]
    fn set_indices_works() {
        set_indices_works_(ConstCell::<3>::new(3, 0));
        set_indices_works_(BitCell::new(3, 0));
        set_indices_works_(DynCell::new(3, 0));
    }

    fn set_indices_works_(mut cell: impl Cell) {
        assert_eq!(cell_indices(&cell), vec![0, 1, 2]);

        cell.set_indices([1, 2]);

        assert_eq!(cell_indices(&cell), vec![1, 2]);

        cell.set_indices([]);

        // A cell with empty indices must be possible
        assert_eq!(cell_indices(&cell), vec![]);
    }

    #[test]
    fn get_collapse_index_works() {
        get_collapse_index_works_(ConstCell::<3>::new(3, 0));
        get_collapse_index_works_(BitCell::new(3, 0));
        get_collapse_index_works_(DynCell::new(3, 0));
    }

    fn get_collapse_index_works_(mut cell: impl Cell) {
        assert_eq!(cell_indices(&cell), vec![0, 1, 2]);

        cell.collapse(0);

        assert_eq!(cell.get_collapsed_index(), Some(0));

        cell.set_indices([1, 2]);

        assert_eq!(cell.get_collapsed_index(), None);

        cell.set_indices([]);

        assert_eq!(cell.get_collapsed_index(), None);
    }

    #[test]
    fn is_collapsed_works() {
        is_collapsed_works_(ConstCell::<3>::new(3, 0));
        is_collapsed_works_(BitCell::new(3, 0));
        is_collapsed_works_(DynCell::new(3, 0));
    }

    fn is_collapsed_works_(mut cell: impl Cell) {
        assert_eq!(cell_indices(&cell), vec![0, 1, 2]);

        assert!(!cell.is_collapsed());

        cell.set_indices([1, 2]);

        assert!(!cell.is_collapsed());

        cell.collapse(0);

        assert!(cell.is_collapsed());

        cell.set_indices([]);

        // A cell with zero entropy is not collapsed, but an error state
        assert!(!cell.is_collapsed());
    }

    #[test]
    fn bucket_index_works() {
        bucket_index_works_(ConstCell::<3>::new(3, 0));
        bucket_index_works_(BitCell::new(3, 0));
        bucket_index_works_(DynCell::new(3, 0));
    }

    fn bucket_index_works_(mut cell: impl Cell) {
        assert_eq!(0, cell.get_bucket_index());

        cell.set_bucket_index(1);

        assert_eq!(1, cell.get_bucket_index());
    }

    #[test]
    fn possible_indices_iter_works() {
        let array = [2, 4, 6];
        let array_indices = PossibleIndices::from_array(&array, 3);
        let iter = array_indices.iter();
        assert_eq!(iter.collect::<Vec<_>>(), vec![2, 4, 6]);

        let mut bitarray = BitArray::<u128>::new(0);
        bitarray.set(2, true);
        bitarray.set(4, true);
        bitarray.set(6, true);
        let bitarray_indices = PossibleIndices::from_bitarray(bitarray, 3);
        let iter = bitarray_indices.iter();
        assert_eq!(iter.collect::<Vec<_>>(), vec![2, 4, 6])
    }

    fn cell_indices(cell: &impl Cell) -> Vec<u8> {
        cell.get_possible_indices().iter().collect()
    }
}
