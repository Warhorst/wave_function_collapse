use pad::position::Position;

// TODO doc in whole module

pub trait TileFilter<T> {
    fn filter_tiles(
        &self,
        position: Position,
        context: TileFilterContext<T>,
        possible_tiles: &mut FilteredTiles,
    );
}

impl<T, F> TileFilter<T> for F
where
    F: Fn(Position, TileFilterContext<T>, &mut FilteredTiles),
{
    fn filter_tiles(
        &self,
        position: Position,
        context: TileFilterContext<T>,
        possible_tiles: &mut FilteredTiles,
    ) {
        self(position, context, possible_tiles)
    }
}

pub struct TileFilterContext<'a, T> {
    pub width: usize,
    pub height: usize,
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
                TileFilterContext {
                    width,
                    height,
                    tiles,
                },
                &mut self.filtered_tiles,
            )
        });

        &self.filtered_tiles
    }
}

pub struct FilteredTiles {
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

    pub fn get_possible_tile_indices(&self) -> impl Iterator<Item = u8> {
        self.tiles
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(i, b)| if b { Some(i as u8) } else { None })
    }

    pub fn filter(
        &mut self,
        i: usize,
    ) {
        if let Some(b) = self.tiles.get_mut(i) {
            *b = false
        }
    }
}
