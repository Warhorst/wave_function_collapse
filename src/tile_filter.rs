use pad::position::Position;

/// Mofifies the possible tiles for a cell before the collapse starts.
pub trait TileFilter<T> {
    /// Modifiy the possible tiles for the [`crate::cell::Cell`] at the given [`Position`].
    ///
    /// * `position`: The [`Position`] of the [`crate::cell::Cell`] on the [`crate::board::Board`] which tiles should be filtered.
    /// * `filtered_tiles`: The [`FilteredTiles`] where tiles can be filtered out.
    /// * `context`: Provided addtional data that might be required for the filter.
    fn filter_tiles(
        &self,
        position: Position,
        filtered_tiles: &mut FilteredTiles,
        context: TileFilterContext<T>,
    );
}

impl<T, F> TileFilter<T> for F
where
    F: Fn(Position, &mut FilteredTiles, TileFilterContext<T>),
{
    fn filter_tiles(
        &self,
        position: Position,
        possible_tiles: &mut FilteredTiles,
        context: TileFilterContext<T>,
    ) {
        self(position, possible_tiles, context)
    }
}

/// Additional data for a [`TileFilter`]
pub struct TileFilterContext<'a, T> {
    /// The width of the [`crate::board::Board`] (in tiles).
    pub width: usize,
    /// The height of the [`crate::board::Board`] (in tiles).
    pub height: usize,
    /// All existing tiles in the [`crate::Wfc`].
    pub tiles: &'a [T],
}

pub(super) struct TileFilters<T> {
    tile_filters: Vec<Box<dyn TileFilter<T>>>,
    filtered_tiles: FilteredTiles,
}

impl<T> TileFilters<T> {
    pub fn new(num_tiles: usize) -> Self {
        TileFilters {
            tile_filters: vec![],
            filtered_tiles: FilteredTiles::new(num_tiles),
        }
    }

    pub fn add_tile_filter(
        &mut self,
        tile_filter: impl TileFilter<T> + 'static,
    ) {
        self.tile_filters.push(Box::new(tile_filter));
    }

    pub fn filter_tiles(
        &mut self,
        position: Position,
        width: usize,
        height: usize,
        tiles: &[T],
    ) -> &FilteredTiles {
        self.filtered_tiles.reset();
        self.tile_filters.iter().for_each(|f| {
            f.filter_tiles(
                position,
                &mut self.filtered_tiles,
                TileFilterContext {
                    width,
                    height,
                    tiles,
                },
            )
        });

        &self.filtered_tiles
    }

    pub fn is_empty(&self) -> bool {
        self.tile_filters.is_empty()
    }
}

/// The filtered tiles for one specific [`Position`] on the [`crate::board::Board`].
pub struct FilteredTiles {
    /// The filtered tiles. `true` means the tile is still possible while `false` means it is not.
    /// The index of the filter is the same as the index of the tile in the list of tiles the [`crate::Wfc`] holds.
    tiles: Vec<bool>,
}

impl FilteredTiles {
    fn new(num_tiles: usize) -> Self {
        FilteredTiles {
            tiles: (0..num_tiles).into_iter().map(|_| true).collect(),
        }
    }

    fn reset(&mut self) {
        self.tiles.iter_mut().for_each(|b| *b = true);
    }

    pub (crate) fn get_possible_tile_indices(&self) -> impl Iterator<Item = u8> {
        self.tiles
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(i, b)| if b { Some(i as u8) } else { None })
    }

    /// Filter out the tile at index `i` of the list of all tiles.
    pub fn filter(
        &mut self,
        i: usize,
    ) {
        if let Some(b) = self.tiles.get_mut(i) {
            *b = false
        }
    }
}
