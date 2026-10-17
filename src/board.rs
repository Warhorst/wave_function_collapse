use crate::constraints::TileConstraints;
use crate::{WfcError, cell::Cell};
use pad::p;
use pad::position::Position;
use std::collections::VecDeque;
// todo Maybe try using a set of not collapsed positions

/// Contains the current state of the WFC with all the cells at their respective positions.
/// The WFC is done if all positions on the board are collapsed.
pub struct Board<C: Cell> {
    /// Width of the [Board]
    width: usize,
    /// Height of the [Board]
    height: usize,
    /// The [Cell]s of the [Board], which tell what tiles are still possible
    cells: Vec<C>,
    pub(crate) weights: Vec<f32>,
    bucket_queue: BucketQueue,
    /// The preallocated queue which will be used to hold positions to propagate next
    /// in the propagation step.
    propagation_queue: VecDeque<Position>,
}

impl<C: Cell> Board<C> {
    pub fn new(
        width: usize,
        height: usize,
        num_tiles: usize,
        weights: &[f32],
    ) -> Self {
        let cells = (0..(width * height))
            .map(|i| C::new(num_tiles, i))
            .collect();
        let bucket_queue =
            BucketQueue::initialize(p!(0, 0).iter_to(p!(width - 1, height - 1)), num_tiles);

        let weights = weights.to_vec();

        Board {
            width,
            height,
            cells,
            weights,
            bucket_queue,
            propagation_queue: VecDeque::new(),
        }
    }

    /// tells if the full board is collapsed
    pub fn collapsed(&self) -> bool {
        self.bucket_queue.is_empty()
    }

    /// Collapse the cell at the given position and set its tile index to the given one
    pub fn collapse_position(
        &mut self,
        position: Position,
        index: u8,
    ) {
        let cell = self.get_cell_mut(position);
        let entropy = cell.entropy();
        cell.collapse(index);

        let cell = self.get_cell(position);
        if let Some((pos, index)) = self
            .bucket_queue
            .remove_cell(entropy as usize, cell.get_bucket_index())
        {
            let cell = self.get_cell_mut(pos);
            cell.set_bucket_index(index);
        }
    }

    /// Adapt all the cardinal neighbours of the given collapsed position
    /// This works recursive, so a collapsed neighbour will propagate the collapse to all
    /// its neighbours
    pub(crate) fn propagate<T>(
        &mut self,
        collapsed_position: Position,
        tile_constraints: &mut TileConstraints<T>,
        all_tiles: &[T],
    ) -> Result<(), WfcError> {
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
                    tile_constraints.update_cell((cell_indices, pos), neighbours, all_tiles);

                // If the new indices are empty, it means there is
                // no tile which fulfills all constraints. This is an error
                // and is returned to the caller
                if new_indices.is_empty() {
                    return Err(WfcError::CellHasZeroEntropy(pos));
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

                let cell = self.get_cell(pos);

                // Update the cell position in the bucket list
                if cell.is_collapsed() {
                    if let Some((pos, index)) = self
                        .bucket_queue
                        .remove_cell(current_entropy as usize, cell.get_bucket_index())
                    {
                        let cell = self.get_cell_mut(pos);
                        cell.set_bucket_index(index);
                    }
                } else {
                    let (new_bucket_index, moved_cell_opt) = self.bucket_queue.move_cell(
                        current_entropy as usize,
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
        }

        Ok(())
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

    pub fn get_collapsed_indices(&self) -> impl Iterator<Item = (Position, usize)> + '_ {
        p!(0, 0)
            .iter_to(p!(self.width - 1, self.height - 1))
            .map(|pos| (pos, self.get_cell(pos).get_collapsed_index() as usize))
    }
}

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
            return
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
