use crate::{cell::Cell, WfcError};
use crate::constraints::TileConstraints;
use pad::p;
use pad::position::Position;
use std::collections::{BTreeSet, VecDeque};
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
    pub (crate) weights: Vec<f32>,
    /// All [Position]s which are not collapsed yet. Used to more efficiently find the
    /// next cell with the lowest entropy.
    non_collapsed_positions: BTreeSet<Position>,
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
            .map(|_| C::new(num_tiles))
            .collect();
        let non_collapsed_positions = p!(0, 0).iter_to(p!(width - 1, height - 1)).collect();

        let weights = weights.to_vec();

        Board {
            width,
            height,
            cells,
            weights,
            non_collapsed_positions,
            propagation_queue: VecDeque::new(),
        }
    }

    /// tells if the full board is collapsed
    pub fn collapsed(&self) -> bool {
        self.non_collapsed_positions.is_empty()
    }

    /// Collapse the cell at the given position and set its tile index to the given one
    pub fn collapse_position(
        &mut self,
        position: Position,
        index: u8,
    ) {
        self.get_cell_mut(position).collapse(index);
        self.non_collapsed_positions.remove(&position);
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
                let cell_indices = cell.get_possible_indices();
                let new_indices = tile_constraints.update_cell(
                    (cell_indices, pos),
                    neighbours,
                    all_tiles,
                );

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

                if cell.is_collapsed() {
                    self.non_collapsed_positions.remove(&pos);
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
        self.non_collapsed_positions
            .iter()
            .map(|pos| (*pos, self.get_cell(*pos)))
            .min_by(|(_, cell_a), (_, cell_b)| cell_a.entropy().cmp(&cell_b.entropy()))
            .expect("At least one non collapsed cell should exist")
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
