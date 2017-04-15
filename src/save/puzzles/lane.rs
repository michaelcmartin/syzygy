// +--------------------------------------------------------------------------+
// | Copyright 2016 Matthew D. Steele <mdsteele@alum.mit.edu>                 |
// |                                                                          |
// | This file is part of System Syzygy.                                      |
// |                                                                          |
// | System Syzygy is free software: you can redistribute it and/or modify it |
// | under the terms of the GNU General Public License as published by the    |
// | Free Software Foundation, either version 3 of the License, or (at your   |
// | option) any later version.                                               |
// |                                                                          |
// | System Syzygy is distributed in the hope that it will be useful, but     |
// | WITHOUT ANY WARRANTY; without even the implied warranty of               |
// | MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU        |
// | General Public License for details.                                      |
// |                                                                          |
// | You should have received a copy of the GNU General Public License along  |
// | with System Syzygy.  If not, see <http://www.gnu.org/licenses/>.         |
// +--------------------------------------------------------------------------+

use std::cmp::min;
use toml;

use save::{Access, Location};
use save::memory::{Grid, Shape};
use save::util::{ACCESS_KEY, pop_array, to_u32};
use super::PuzzleState;

// ========================================================================= //

const GRID_KEY: &'static str = "grid";
const STAGE_KEY: &'static str = "stage";

const NUM_COLS: usize = 6;
const NUM_ROWS: usize = 4;
const NUM_SYMBOLS: i32 = 6;

enum Stage {
    Place(Shape),
    Remove(i8),
}

#[cfg_attr(rustfmt, rustfmt_skip)]
const STAGES: &'static [Stage] = &[
    Stage::Place(Shape([0, 1, 0, 0, 1, 1, 0, 1, 0])),
    Stage::Place(Shape([0, 0, 0, 2, 2, 2, 0, 0, 2])),
    Stage::Place(Shape([0, 0, 0, 3, 3, 0, 3, 3, 0])),
    Stage::Place(Shape([0, 0, 0, 5, 5, 5, 5, 0, 0])),
    Stage::Remove(1),
    Stage::Place(Shape([0, 6, 6, 0, 6, 0, 0, 6, 0])),
    Stage::Remove(3),
    Stage::Place(Shape([0, 4, 0, 4, 4, 0, 4, 0, 0])),
    Stage::Remove(2),
    Stage::Remove(5),
    Stage::Remove(6),
    Stage::Place(Shape([0, 3, 0, 3, 3, 3, 0, 0, 0])),
    Stage::Place(Shape([0, 0, 0, 0, 2, 2, 2, 2, 0])),
    Stage::Place(Shape([0, 1, 1, 0, 1, 0, 0, 1, 0])),
    Stage::Place(Shape([5, 5, 0, 0, 5, 0, 0, 5, 0])),
    Stage::Remove(4),
    Stage::Remove(2),
    Stage::Place(Shape([0, 0, 0, 6, 6, 6, 6, 0, 0])),
    Stage::Remove(3),
    Stage::Remove(5),
    Stage::Remove(1),
    Stage::Remove(6),
];

// ========================================================================= //

pub struct LaneState {
    access: Access,
    grid: Grid,
    stage: usize,
}

impl LaneState {
    pub fn from_toml(mut table: toml::value::Table) -> LaneState {
        let access = Access::from_toml(table.get(ACCESS_KEY));
        let stage = if access.is_solved() {
            STAGES.len()
        } else {
            min(table.remove(STAGE_KEY).map(to_u32).unwrap_or(0) as usize,
                STAGES.len() - 1)
        };
        let grid = Grid::from_toml(NUM_COLS,
                                   NUM_ROWS,
                                   pop_array(&mut table, GRID_KEY));
        LaneState {
            access: access,
            grid: grid,
            stage: stage,
        }
    }

    pub fn solve(&mut self) {
        self.access = Access::Solved;
        self.grid.clear();
        self.stage = STAGES.len();
    }

    pub fn total_num_stages(&self) -> usize { STAGES.len() }

    pub fn current_stage(&self) -> usize { self.stage }

    pub fn grid(&self) -> &Grid { &self.grid }

    pub fn grid_mut(&mut self) -> &mut Grid { &mut self.grid }

    pub fn next_shape(&self) -> Option<Shape> {
        if self.stage < STAGES.len() {
            match STAGES[self.stage] {
                Stage::Place(ref shape) => Some(shape.clone()),
                Stage::Remove(_) => None,
            }
        } else {
            None
        }
    }

    pub fn next_remove(&self) -> Option<i8> {
        if self.stage < STAGES.len() {
            match STAGES[self.stage] {
                Stage::Place(_) => None,
                Stage::Remove(symbol) => Some(symbol),
            }
        } else {
            None
        }
    }

    pub fn try_place_shape(&mut self, col: i32, row: i32) -> Option<i8> {
        if let Some(shape) = self.next_shape() {
            if self.grid.try_place_shape(&shape, col, row) {
                self.advance();
                return shape.symbol();
            }
        }
        None
    }

    pub fn can_remove_symbol(&self, symbol: i8) -> bool {
        assert!(symbol > 0 && symbol as i32 <= NUM_SYMBOLS);
        self.next_remove() == Some(symbol)
    }

    pub fn decay_symbol_all(&mut self, symbol: i8) {
        self.grid.decay_symbol(symbol, NUM_COLS * NUM_ROWS);
    }

    pub fn remove_symbol(&mut self, symbol: i8) {
        assert!(symbol > 0 && symbol as i32 <= NUM_SYMBOLS);
        if self.can_remove_symbol(symbol) {
            self.grid.remove_symbol(symbol);
            self.advance();
        } else {
            self.reset();
        }
    }

    fn advance(&mut self) {
        debug_assert!(self.stage < STAGES.len());
        self.stage += 1;
        if self.stage == STAGES.len() {
            self.access = Access::Solved;
        }
    }
}

impl PuzzleState for LaneState {
    fn location(&self) -> Location { Location::MemoryLane }

    fn access(&self) -> Access { self.access }

    fn access_mut(&mut self) -> &mut Access { &mut self.access }

    fn can_reset(&self) -> bool { self.stage > 0 }

    fn reset(&mut self) {
        self.grid.clear();
        self.stage = 0;
    }

    fn to_toml(&self) -> toml::Value {
        let mut table = toml::value::Table::new();
        table.insert(ACCESS_KEY.to_string(), self.access.to_toml());
        if !self.access.is_solved() {
            table.insert(STAGE_KEY.to_string(),
                         toml::Value::Integer(self.stage as i64));
            table.insert(GRID_KEY.to_string(), self.grid.to_toml());
        }
        toml::Value::Table(table)
    }
}

// ========================================================================= //

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{NUM_SYMBOLS, STAGES, Stage};

    #[test]
    fn stages_are_well_formed() {
        let mut symbols_on_board = HashSet::new();
        for (index, stage) in STAGES.iter().enumerate() {
            match stage {
                &Stage::Place(ref shape) => {
                    let mut num_symbols = 0;
                    for (_, symbol) in shape.tiles() {
                        assert!(symbol as i32 <= NUM_SYMBOLS);
                        num_symbols += 1;
                        symbols_on_board.insert(symbol);
                    }
                    assert_eq!(num_symbols, 4);
                }
                &Stage::Remove(symbol) => {
                    assert!(symbol > 0 && symbol as i32 <= NUM_SYMBOLS);
                    assert!(symbols_on_board.contains(&symbol),
                            "Stage {} frees {}, but it's not on the board.",
                            index,
                            symbol);
                    symbols_on_board.remove(&symbol);
                }
            }
        }
        assert!(symbols_on_board.is_empty(),
                "At the end of the puzzle, {:?} are still on the board.",
                symbols_on_board);
    }
}

// ========================================================================= //