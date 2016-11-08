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

use std::collections::VecDeque;
use std::default::Default;
use toml;

use save::{Access, Direction};
use super::super::util::{ACCESS_KEY, pop_array};

// ========================================================================= //

const NUM_COLS: i32 = 9;
const NUM_ROWS: i32 = 7;

const GRID_KEY: &'static str = "grid";

const INITIAL_GRID: &'static [i8] = &[2, 1, 2, 1, 2, -1, 2, 0, 1, -1, -1, 1,
                                      2, 0, -1, -1, 2, 0, 0, 1, 2, 1, -1, 0,
                                      1, 1, 2, 2, 0, -1, -1, 2, -1, -1, -1,
                                      1, 2, 1, 0, 2, -1, 1, 2, 2, 0, 0, -1,
                                      -1, -1, 1, 0, -1, 0, 1, 1, 0, 2, 2, 0,
                                      1, -1, 2, 2];

const SOLVED_GRID: &'static [i8] = &[0, 0, 0, 1, 1, -1, 2, 2, 2, -1, -1, 0,
                                     1, 1, -1, -1, 2, 2, 2, 2, 2, 0, -1, 0,
                                     1, 1, 1, 2, 2, -1, -1, 0, -1, -1, -1, 1,
                                     2, 2, 2, 0, -1, 0, 1, 1, 1, 1, -1, -1,
                                     -1, 2, 2, -1, 0, 0, 1, 1, 1, 2, 2, 2,
                                     -1, 0, 0];

// ========================================================================= //

pub struct WreckedState {
    access: Access,
    grid: Vec<i8>,
    is_initial: bool,
}

impl WreckedState {
    pub fn from_toml(mut table: toml::Table) -> WreckedState {
        let mut grid: Vec<i8> = pop_array(&mut table, GRID_KEY)
                                    .iter()
                                    .filter_map(|value| value.as_integer())
                                    .filter(|&tile| -1 <= tile && tile < 3)
                                    .map(|tile| tile as i8)
                                    .collect();
        let mut init_sorted = INITIAL_GRID.to_vec();
        init_sorted.sort();
        let mut grid_sorted = grid.clone();
        grid_sorted.sort();
        if grid_sorted != init_sorted {
            grid = INITIAL_GRID.to_vec()
        } else {
            let init_neg: Vec<bool> = INITIAL_GRID.iter()
                                                  .map(|&tile| tile < 0)
                                                  .collect();
            let grid_neg: Vec<bool> = grid.iter()
                                          .map(|&tile| tile < 0)
                                          .collect();
            if grid_neg != init_neg {
                grid = INITIAL_GRID.to_vec();
            }
        }
        let is_initial = &grid as &[i8] == INITIAL_GRID;
        WreckedState {
            access: Access::from_toml(table.get(ACCESS_KEY)),
            grid: grid,
            is_initial: is_initial,
        }
    }

    pub fn to_toml(&self) -> toml::Value {
        let mut table = toml::Table::new();
        table.insert(ACCESS_KEY.to_string(), self.access.to_toml());
        if !self.is_initial {
            let grid = self.grid
                           .iter()
                           .map(|&idx| toml::Value::Integer(idx as i64))
                           .collect();
            table.insert(GRID_KEY.to_string(), toml::Value::Array(grid));
        }
        toml::Value::Table(table)
    }

    pub fn access(&self) -> Access { self.access }

    pub fn is_visited(&self) -> bool { self.access.is_visited() }

    pub fn visit(&mut self) { self.access.visit(); }

    pub fn is_solved(&self) -> bool { self.access == Access::Solved }

    pub fn replay(&mut self) {
        self.access = Access::Replay;
        self.grid = INITIAL_GRID.to_vec();
        self.is_initial = true;
    }

    pub fn is_in_initial_configuration(&self) -> bool { self.is_initial }

    pub fn reset(&mut self) {
        self.grid = INITIAL_GRID.to_vec();
        self.is_initial = true;
    }

    pub fn solve(&mut self) {
        self.access = Access::Solved;
        self.grid = SOLVED_GRID.to_vec();
        self.is_initial = false;
    }

    pub fn tile_at(&self, col: i32, row: i32) -> Option<usize> {
        if col < 0 || col >= NUM_COLS || row < 0 || row >= NUM_ROWS {
            None
        } else {
            let index = (row * NUM_COLS + col) as usize;
            let value = self.grid[index];
            if value < 0 {
                None
            } else {
                Some(value as usize)
            }
        }
    }

    pub fn shift_tiles(&mut self, dir: Direction, rank: i32) {
        match dir {
            Direction::East | Direction::West => {
                if rank >= 0 && rank < NUM_ROWS {
                    let mut tiles = VecDeque::new();
                    for col in 0..NUM_COLS {
                        let index = (rank * NUM_COLS + col) as usize;
                        let value = self.grid[index];
                        if value >= 0 {
                            tiles.push_back(value);
                        }
                    }
                    if dir == Direction::East {
                        let tile = tiles.pop_back().unwrap();
                        tiles.push_front(tile);
                    } else {
                        let tile = tiles.pop_front().unwrap();
                        tiles.push_back(tile);
                    }
                    for col in 0..NUM_COLS {
                        let index = (rank * NUM_COLS + col) as usize;
                        if self.grid[index] >= 0 {
                            self.grid[index] = tiles.pop_front().unwrap();
                        }
                    }
                }
            }
            Direction::South | Direction::North => {
                if rank >= 0 && rank < NUM_COLS {
                    let mut tiles = VecDeque::new();
                    for row in 0..NUM_ROWS {
                        let index = (row * NUM_COLS + rank) as usize;
                        let value = self.grid[index];
                        if value >= 0 {
                            tiles.push_back(value);
                        }
                    }
                    if dir == Direction::South {
                        let tile = tiles.pop_back().unwrap();
                        tiles.push_front(tile);
                    } else {
                        let tile = tiles.pop_front().unwrap();
                        tiles.push_back(tile);
                    }
                    for row in 0..NUM_ROWS {
                        let index = (row * NUM_COLS + rank) as usize;
                        if self.grid[index] >= 0 {
                            self.grid[index] = tiles.pop_front().unwrap();
                        }
                    }
                }
            }
        }
        self.is_initial = &self.grid as &[i8] == INITIAL_GRID;
        if &self.grid as &[i8] == SOLVED_GRID {
            self.access = Access::Solved;
        }
    }
}

impl Default for WreckedState {
    fn default() -> WreckedState {
        WreckedState {
            access: Default::default(),
            grid: INITIAL_GRID.to_vec(),
            is_initial: true,
        }
    }
}

// ========================================================================= //

#[cfg(test)]
mod tests {
    use save::Direction;
    use super::WreckedState;

    #[test]
    fn shift_east() {
        let mut state: WreckedState = Default::default();
        state.shift_tiles(Direction::East, 0);
        assert_eq!(state.tile_at(0, 0), Some(1));
        assert_eq!(state.tile_at(5, 0), None);
        assert_eq!(state.tile_at(6, 0), Some(2));
    }

    #[test]
    fn shift_west() {
        let mut state: WreckedState = Default::default();
        state.shift_tiles(Direction::West, 1);
        assert_eq!(state.tile_at(1, 1), None);
        assert_eq!(state.tile_at(4, 1), Some(2));
        assert_eq!(state.tile_at(8, 1), Some(1));
    }

    #[test]
    fn shift_south() {
        let mut state: WreckedState = Default::default();
        state.shift_tiles(Direction::South, 0);
        assert_eq!(state.tile_at(0, 0), Some(1));
        assert_eq!(state.tile_at(0, 1), None);
        assert_eq!(state.tile_at(0, 2), Some(2));
    }

    #[test]
    fn shift_north() {
        let mut state: WreckedState = Default::default();
        state.shift_tiles(Direction::North, 8);
        assert_eq!(state.tile_at(8, 1), Some(2));
        assert_eq!(state.tile_at(8, 4), Some(1));
        assert_eq!(state.tile_at(8, 6), Some(1));
    }
}

// ========================================================================= //