use crate::cell::Cell;
use crate::constraints::Constraints;
use crate::tile_filter::FilteredTiles;
use pad::p;
use pad::position::Position;
use std::collections::VecDeque;
use thiserror::Error;

/// Contains the current state of the WFC with all the cells at their respective positions.
/// The WFC is done if all positions on the board are collapsed.
pub struct Board<C: Cell> {
    /// Width of the [`Board`]
    pub(super) width: usize,
    /// Height of the [`Board`]
    pub(super) height: usize,
    /// The [`Cell`]s of the [`Board`], which tell what tiles are still possible
    cells: Vec<C>,
    /// Bucket queue which is used to determine the next cell with minimal entropy.
    bucket_queue: BucketQueue,
    /// The preallocated queue which will be used to hold positions to propagate next
    /// in the propagation step.
    propagation_queue: VecDeque<Position>,
    /// Preallocated vec to store the collapsed positions and their tile indices when collapsing.
    collapsed_positions: Vec<(Position, u8)>,
}

impl<C: Cell> Board<C> {
    pub fn new(
        width: usize,
        height: usize,
        num_tiles: usize,
    ) -> Self {
        let cells = (0..(width * height))
            .map(|i| C::new(num_tiles, i))
            .collect();
        let bucket_queue =
            BucketQueue::initialize(p!(0, 0).iter_to(p!(width - 1, height - 1)), num_tiles);

        Board {
            width,
            height,
            cells,
            bucket_queue,
            propagation_queue: VecDeque::new(),
            collapsed_positions: Vec::with_capacity((width * height) / 10),
        }
    }

    /// Modify the possible indices of the [`Cell`] at the given [`Position`] based on the given
    /// [`FilteredTiles`]. Should only be used before starting the collapse, as any further cell update
    /// should be handled by the propagation.
    pub fn filter_tiles(
        &mut self,
        position: Position,
        filtered_tiles: &FilteredTiles,
    ) -> Result<(), BoardError> {
        let cell = self.get_cell_mut(position);
        let entropy = cell.entropy();
        cell.set_indices(filtered_tiles.get_possible_tile_indices());

        if cell.entropy() == 0 {
            return Err(BoardError::ZeroEntropyAfterFilter(position));
        }

        self.update_bucket_queue(position, entropy);

        Ok(())
    }

    /// Tells if the full board is collapsed
    pub fn collapsed(&self) -> bool {
        self.bucket_queue.is_empty()
    }

    /// Collapse the cell at the given position and set its tile index to the given one
    pub(crate) fn collapse_position<T>(
        &mut self,
        position: Position,
        index: u8,
        constraints: &mut Constraints<T>,
        all_tiles: &[T],
    ) -> Result<&[(Position, u8)], BoardError> {
        self.collapsed_positions.clear();
        self.collapsed_positions.push((position, index));

        let cell = self.get_cell_mut(position);
        let entropy = cell.entropy();
        cell.collapse(index);

        self.update_bucket_queue(position, entropy);

        self.propagate(position, constraints, all_tiles)
    }

    /// Adapt all the cardinal neighbours of the given collapsed position
    /// This works recursively, so a collapsed neighbour will propagate the collapse to all
    /// its neighbours
    fn propagate<T>(
        &mut self,
        collapsed_position: Position,
        constraints: &mut Constraints<T>,
        all_tiles: &[T],
    ) -> Result<&[(Position, u8)], BoardError> {
        // init the propagation queue with the just collapsed position
        self.propagation_queue.push_back(collapsed_position);

        // process the queue until nothing needs propagation anymore
        while !self.propagation_queue.is_empty() {
            let collapsed_position = self.propagation_queue.pop_front().unwrap();

            // go over all the neighbours to check if they can be updated
            for pos in collapsed_position.cardinal_neighbours() {
                // if on the edge, the neighbour might be out of bounds, so
                // skip in this case
                if !self.pos_in_bounds(pos) {
                    continue;
                }

                // ignore already collapsed neighbours
                if self.get_cell(pos).is_collapsed() {
                    continue;
                }

                // Collect the relevant data from the neighbour cell to
                // create a cell update for it, which is its next state
                let neighbours = pos
                    .cardinal_neighbours()
                    .into_iter()
                    .filter(|p| self.pos_in_bounds(*p))
                    .map(|p| (self.get_cell(p).get_possible_indices(), p));
                let cell = self.get_cell(pos);
                let current_entropy = cell.entropy();
                let cell_indices = cell.get_possible_indices();
                let new_indices =
                    constraints.update_cell((cell_indices, pos), neighbours, all_tiles);

                // If the new indices are empty, it means there is
                // no tile which fulfills all constraints. This is an error
                // and is returned to the caller
                if new_indices.is_empty() {
                    return Err(BoardError::ZeroEntropyAfterPropagate(pos));
                }

                // If the indices changed (which can only mean: there are
                // now fewer tiles), the cell has now a lower entropy. This
                // must be propagated to its neighbours, so it is added to the
                // propagation queue
                if cell_indices.entropy() as usize != new_indices.len() {
                    self.propagation_queue.push_back(pos);
                } else {
                    continue;
                }

                // update the cell with the values from the cell update
                let cell_mut = self.get_cell_mut(pos);
                cell_mut.set_indices(new_indices.iter().copied());

                // Add the collapsed cell to the vec of collapsed cells
                if cell_mut.is_collapsed() {
                    let collapsed_index = cell_mut
                        .get_collapsed_index()
                        .expect("The cell is collapsed, so the collapsed index must be present");
                    self.collapsed_positions.push((pos, collapsed_index));
                }

                self.update_bucket_queue(pos, current_entropy);
            }
        }

        Ok(&self.collapsed_positions)
    }

    fn update_bucket_queue(
        &mut self,
        pos: Position,
        previous_entropy: u8,
    ) {
        let cell = self.get_cell(pos);

        if cell.is_collapsed() {
            if let Some((pos, index)) = self
                .bucket_queue
                .remove_cell(previous_entropy as usize, cell.get_bucket_index())
            {
                let cell = self.get_cell_mut(pos);
                cell.set_bucket_index(index);
            }
        } else {
            let (new_bucket_index, moved_cell_opt) = self.bucket_queue.move_cell(
                previous_entropy as usize,
                cell.get_bucket_index(),
                cell.entropy() as usize,
                pos,
            );

            let cell = self.get_cell_mut(pos);
            cell.set_bucket_index(new_bucket_index);

            if let Some((pos, index)) = moved_cell_opt {
                let cell = self.get_cell_mut(pos);
                cell.set_bucket_index(index);
            }
        }
    }

    fn pos_in_bounds(
        &self,
        pos: Position,
    ) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.width as isize && pos.y < self.height as isize
    }

    pub fn get_min_entropy_position(&self) -> (Position, &C) {
        let min_pos = *self.bucket_queue.next_min();
        let cell = self.get_cell(min_pos);
        (min_pos, cell)
    }

    pub fn get_cell(
        &self,
        pos: Position,
    ) -> &C {
        self.cells
            .get(self.width * pos.y as usize + pos.x as usize)
            .unwrap_or_else(|| panic!("A cell at position {:?} should exist", pos))
    }

    pub fn get_cell_mut(
        &mut self,
        pos: Position,
    ) -> &mut C {
        self.cells
            .get_mut(self.width * pos.y as usize + pos.x as usize)
            .unwrap_or_else(|| panic!("A cell at position {:?} should exist", pos))
    }
}

/// [`Bucket Queue`](https://en.wikipedia.org/wiki/Bucket_queue) implementation
/// where every bucket is a possible entropy (from 1 to max) and the items in the buckets
/// are positions of [`Cell`]s on the [`Board`].
struct BucketQueue {
    min_entropy: usize,
    buckets: Vec<Vec<Position>>,
    num_cells: usize,
}

impl BucketQueue {
    fn initialize(
        positions: impl IntoIterator<Item = Position>,
        max_entroypy: usize,
    ) -> Self {
        let mut buckets = vec![];
        let positions: Vec<Position> = positions.into_iter().collect();
        let num_cells = positions.len();

        for _ in 0..max_entroypy - 1 {
            // It is expected that less space is required for smaller entropies
            buckets.push(Vec::with_capacity(positions.len() / 10));
        }

        buckets.push(positions);

        BucketQueue {
            min_entropy: max_entroypy,
            buckets,
            num_cells,
        }
    }

    fn is_empty(&self) -> bool {
        self.num_cells == 0
    }

    fn next_min(&self) -> &Position {
        self.buckets[self.min_entropy - 1]
            .first()
            .expect("The bucket cannot be empty")
    }

    /// Move a cell at old_entropy and bucket_index to the new entropy.
    ///
    /// This returns the new bucket index of the cell. This also might return
    /// the position and new bucket index of another cell that was also moved in the process
    fn move_cell(
        &mut self,
        old_entropy: usize,
        bucket_index: usize,
        new_entropy: usize,
        position: Position,
    ) -> (usize, Option<(Position, usize)>) {
        let bucket = &mut self.buckets[old_entropy - 1];

        let last_index = bucket.len() - 1;
        // If the cell was the last entry in the bucket, nothing was moved due to the swap remove
        let moved_index = (bucket_index != last_index).then_some(bucket_index);

        bucket.swap_remove(bucket_index);

        let moved_opt = moved_index.map(|i| (self.buckets[old_entropy - 1][i], i));
        self.buckets[new_entropy - 1].push(position);

        self.update_min_entropy();

        // new bucket index, position and new bucket index of possibly moved cell
        (self.buckets[new_entropy - 1].len() - 1, moved_opt)
    }

    /// Remove the cell at the given entropy and bucket_index.
    ///
    /// This returns the new bucket index of the cell. This also might return
    /// the position and new bucket index of another cell that was also moved in the process
    fn remove_cell(
        &mut self,
        entropy: usize,
        bucket_index: usize,
    ) -> Option<(Position, usize)> {
        let bucket = &mut self.buckets[entropy - 1];

        let last_index = bucket.len() - 1;
        // If the cell was the last entry in the bucket, nothing was moved due to the swap remove
        let moved_index = (bucket_index != last_index).then_some(bucket_index);

        bucket.swap_remove(bucket_index);
        self.num_cells -= 1;
        self.update_min_entropy();

        moved_index.map(|i| (self.buckets[entropy - 1][i], i))
    }

    fn update_min_entropy(&mut self) {
        if self.is_empty() {
            return;
        }

        self.min_entropy = self
            .buckets
            .iter()
            .enumerate()
            .find_map(|(i, bucket)| {
                if !bucket.is_empty() {
                    Some(i + 1)
                } else {
                    None
                }
            })
            .expect("The bucket queue must not be empty")
    }
}

#[derive(Debug, Error)]
pub enum BoardError {
    /// The entropy of a [`Cell`] was zero after applying [`crate::tile_filter::TileFilter`]s.
    #[error("Position {0:?} had an entropy of 0 after applying the tile filters.")]
    ZeroEntropyAfterFilter(Position),
    /// A cell has zero entropy after a propagation, which means no tile
    /// can be picked for it
    #[error("The position {0:?} has zero entropy and cannot be collapsed!")]
    ZeroEntropyAfterPropagate(Position),
}
