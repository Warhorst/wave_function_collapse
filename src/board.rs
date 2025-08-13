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
    ) -> Self {
        let cells = (0..(width * height))
            .into_iter()
            .map(|_| Cell::<C>::new(num_tiles))
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
        all_tiles: &[T]
    ) -> Result<(), WfcError> {
        // todo this gets quite complex. Refactor or add comments to explain the blocks
        self.propagation_queue.push_back(collapsed_position);

        while !self.propagation_queue.is_empty() {
            let collapsed_position = self.propagation_queue.pop_front().unwrap();

            for pos in collapsed_position.cardinal_neighbours() {
                // cover the edges of the board
                if !self.pos_in_bounds(pos) {
                    continue;
                }

                // ignore collapsed cells
                if self.get_cell(pos).is_collapsed() {
                    continue;
                }

                let neighbours = pos.cardinal_neighbours()
                    .into_iter()
                    .filter(|p| self.pos_in_bounds(*p))
                    .map(|p| (self.get_cell(p).get_possible_indices(), p));

                let cell = self.get_cell(pos);
                let cell_indices = cell.get_possible_indices();
                let possible_indices = tile_constraints.get_possible_indices::<C>(
                    (&cell_indices, pos),
                    neighbours,
                    all_tiles,
                );
                let new_indices = possible_indices.get();

                if new_indices.is_empty() {
                    return Err(WfcError::CellHasZeroEntropy(pos))
                }

                if cell_indices != new_indices {
                    self.propagation_queue.push_back(pos);
                }

                self.get_cell_mut(pos).set_indices(new_indices.into_iter().copied());

                let cell = self.get_cell(pos);

                if cell.is_collapsed() {
                    // if the best known position to collapse next is now collapsed, clear
                    // it so a new position can be determined
                    if let Some((p, _)) = self.min_non_collapsed && p == pos {
                        self.min_non_collapsed = None
                    }

                    self.not_collapsed_positions -= 1;
                    //self.propagate(pos, tile_constraints, all_tiles)
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
