use std::collections::VecDeque;
use pad::p;
use pad::position::Position;
use crate::cell::Cell;
use crate::constraints::TileConstraints;
use crate::WfcError;
// todo Maybe try using a set of not collapsed positions

/// Contains the current state of the WFC with all the cells at their respective positions.
/// The WFC is done if all positions on the board are collapsed.
pub struct Board<const C: usize> {
    /// Width of the board
    width: usize,
    /// Height of the board
    height: usize,
    /// The cells of the board, which tell what tiles are still possible
    cells: Vec<Cell<C>>,
    /// Caches the amount of not already collapsed positions to quickly check
    /// if the whole board is collapsed
    not_collapsed_positions: usize,
    // todo there might be cases where this does not provide the position with the lowest entropy
    //  For example, what if there are multiple cells with the lowest possible entropy, I collapse the
    //  one which is cached here and in the propagation I set a position with a larger entropy as the min,
    //  as this was the last one I saw. Maybe I need to cache more positions in a map or whatever
    /// Caches the best (lowest entropy) next cell to collapse which might was found when collapsing
    /// If no min next cell is known, the whole board has to be searched for the lowest entropy position
    pub min_non_collapsed: Option<(Position, Cell<C>)>,
    /// The preallocated queue which will be used to hold positions to propagate next
    /// in the propagation step.
    propagation_queue: VecDeque<Position>
}

impl<const C: usize> Board<C> {
    pub fn new(
        width: usize,
        height: usize,
        num_tiles: usize,
        weights: [f32; C]
    ) -> Self {
        let cells = (0..(width * height))
            .into_iter()
            .map(|_| Cell::<C>::new(num_tiles, &weights))
            .collect();

        Board {
            width,
            height,
            cells,
            not_collapsed_positions: width * height,
            min_non_collapsed: None,
            propagation_queue: VecDeque::new()
        }
    }

    /// tells if the full board is collapsed
    pub fn collapsed(&self) -> bool {
        self.not_collapsed_positions == 0
    }

    /// Collapse the cell at the given position and set its tile index to the given one
    pub fn collapse_position(&mut self, position: Position, index: u8) {
        self.get_cell_mut(position).collapse(index);
        self.not_collapsed_positions -= 1;
    }

    /// Adapt all the cardinal neighbours of the given collapsed position
    /// This works recursive, so a collapsed neighbour will propagate the collapse to all
    /// its neighbours
    pub (crate) fn propagate<T>(
        &mut self,
        collapsed_position: Position,
        tile_constraints: &TileConstraints<T>,
        weights: &[f32],
        all_tiles: &[T]
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
                let neighbours = pos.cardinal_neighbours()
                    .into_iter()
                    .filter(|p| self.pos_in_bounds(*p))
                    .map(|p| (self.get_cell(p).get_possible_indices(), p));
                let cell = self.get_cell(pos);
                let cell_indices = cell.get_possible_indices();
                let cell_update = tile_constraints.update_cell::<C>(
                    (&cell_indices, pos),
                    neighbours,
                    weights,
                    all_tiles,
                );
                let new_indices = cell_update.new_indices();
                let new_weights = cell_update.new_weights();

                // If the new indices are empty, it means there is
                // no tile which fulfills all constraints. This is an error
                // and is returned to the caller
                if new_indices.is_empty() {
                    return Err(WfcError::CellHasZeroEntropy(pos))
                }

                // If the indices changed (which can only mean: there are
                // now fewer tiles), the cell has now a lower entropy. This
                // must be propagated to its neighbours, so it is added to the
                // propagation queue
                if cell_indices != new_indices {
                    self.propagation_queue.push_back(pos);
                } else {
                    continue
                }

                // update the cell with the values from the cell update
                let cell_mut = self.get_cell_mut(pos);
                cell_mut.set_indices(new_indices.into_iter().copied());
                cell_mut.set_weights(new_weights.into_iter().copied());

                // As the last step in the propagation, update the
                // min_non_collapsed, which is the best known position
                // which can be collapsed next

                let cell = self.get_cell(pos);

                if cell.is_collapsed() {
                    // if the best known position to collapse next is now collapsed, clear
                    // it so a new position can be determined
                    if let Some((p, _)) = self.min_non_collapsed && p == pos {
                        self.min_non_collapsed = None
                    }

                    // todo why does this sometimes overflow?
                    self.not_collapsed_positions -= 1;
                } else {
                    // update the cache with the best next cell
                    self.min_non_collapsed = Some(match self.min_non_collapsed {
                        // the current cell now has a lower entropy than the current min cell, overwrite
                        Some((_, c)) if cell.entropy < c.entropy => (pos, *cell),
                        // the current cell is not better, keep the same vale
                        Some((p, c)) => (p, c),
                        // no min value is set, use the current cell
                        None => (pos, *cell)
                    });
                }
            }
        }

        Ok(())
    }

    fn pos_in_bounds(&self, pos: Position) -> bool {
        pos.x >= 0
            && pos.y >= 0
            && pos.x < self.width as isize
            && pos.y < self.height as isize
    }

    pub fn get_min_entropy_position(&self) -> (Position, Cell<C>) {
        p!(0, 0)
            .iter_to(p!(self.width - 1, self.height - 1))
            .map(|pos| (pos, *self.get_cell(pos)))
            .filter(|(_, cell)| cell.entropy > 1)
            .min_by(|(_, cell_a), (_, cell_b)| cell_a.entropy.cmp(&cell_b.entropy))
            .expect("At least one not collapsed cell should exist")
    }

    pub fn get_cell(&self, pos: Position) -> &Cell<C> {
        self.cells.get(self.width * pos.y as usize + pos.x as usize).expect(format!("A cell at position {:?} should exist", pos).as_str())
    }

    pub fn get_cell_mut(&mut self, pos: Position) -> &mut Cell<C> {
        self.cells.get_mut(self.width * pos.y as usize + pos.x as usize).expect(format!("A cell at position {:?} should exist", pos).as_str())
    }

    pub fn get_collapsed_indices(&self) -> impl Iterator<Item=(Position, usize)> + '_ {
        p!(0, 0)
            .iter_to(p!(self.width - 1, self.height - 1))
            .map(|pos| (pos, self.get_cell(pos).get_collapsed_index() as usize))
    }
}
