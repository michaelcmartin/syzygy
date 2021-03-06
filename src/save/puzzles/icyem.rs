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

use toml;

use super::PuzzleState;
use crate::save::column::Columns;
use crate::save::util::{pop_array, to_table, Tomlable, ACCESS_KEY};
use crate::save::{Access, Location};

// ========================================================================= //

const COLUMNS_KEY: &str = "columns";

#[cfg_attr(rustfmt, rustfmt_skip)]
const COLUMNS_SPEC: &[(&str, i32, i32, &[(usize, i32)])] = &[
    // First row:
    ("ELF",  -1, 1, &[(0, 1), (7, 1), (1, 1), (8, 1), (6, -1), (13, -1)]),
    ("HIDE", -2, 1, &[(0, -1), (7, -1), (1, 1), (8, 1), (2, -1), (9, -1)]),
    ("SCAR", -2, 0, &[(2, 1), (9, 1), (4, 1), (11, 1)]),
    ("RUNS", -2, 2, &[(0, -1), (7, -1), (3, 1), (10, 1), (6, -1), (13, -1)]),
    ("FLED", -2, 2, &[(2, 1), (9, 1), (4, 1), (11, 1)]),
    ("BURN", -2, 3, &[(4, -1), (11, -1), (5, 1), (12, 1), (6, -1), (13, -1)]),
    ("SOL",  -1, 1, &[(0, -1), (7, -1), (5, 1), (12, 1), (6, 1), (13, 1)]),
    // Second row:
    ("IDS",  -1, 1, &[(0, 1), (7, 1), (1, 1), (8, 1), (6, -1), (13, -1)]),
    ("LIES", -1, 1, &[(0, -1), (7, -1), (1, 1), (8, 1), (2, -1), (9, -1)]),
    ("SCAM", -1, 0, &[(2, 1), (9, 1), (4, 1), (11, 1)]),
    ("BLUR", -1, 2, &[(0, -1), (7, -1), (3, 1), (10, 1), (6, -1), (13, -1)]),
    ("FAKE", -1, 2, &[(2, 1), (9, 1), (4, 1), (11, 1)]),
    ("CODE", -1, 3, &[(4, -1), (11, -1), (5, 1), (12, 1), (6, -1), (13, -1)]),
    ("SPY",  -1, 1, &[(0, -1), (7, -1), (5, 1), (12, 1), (6, 1), (13, 1)]),
];

// ========================================================================= //

pub struct IcyEmState {
    access: Access,
    columns: Columns,
}

impl IcyEmState {
    pub fn solve(&mut self) {
        self.access = Access::Solved;
        self.columns.solve();
    }

    pub fn columns(&self) -> &Columns {
        &self.columns
    }

    pub fn columns_mut(&mut self) -> &mut Columns {
        &mut self.columns
    }

    pub fn rotate_column(&mut self, col: usize, by: i32) {
        self.columns.rotate_column(col, by);
        if self.columns.is_solved() {
            self.access = Access::Solved;
        }
    }
}

impl PuzzleState for IcyEmState {
    fn location() -> Location {
        Location::ColumnAsIcyEm
    }

    fn access(&self) -> Access {
        self.access
    }

    fn access_mut(&mut self) -> &mut Access {
        &mut self.access
    }

    fn can_reset(&self) -> bool {
        self.columns.can_reset()
    }

    fn reset(&mut self) {
        self.columns.reset();
    }
}

impl Tomlable for IcyEmState {
    fn to_toml(&self) -> toml::Value {
        let mut table = toml::value::Table::new();
        table.insert(ACCESS_KEY.to_string(), self.access.to_toml());
        if !self.is_solved() && self.columns.can_reset() {
            table.insert(COLUMNS_KEY.to_string(), self.columns.to_toml());
        }
        toml::Value::Table(table)
    }

    fn from_toml(value: toml::Value) -> IcyEmState {
        let mut table = to_table(value);
        let access = Access::pop_from_table(&mut table, ACCESS_KEY);
        let mut columns = Columns::from_toml(
            COLUMNS_SPEC,
            pop_array(&mut table, COLUMNS_KEY),
        );
        if access.is_solved() {
            columns.solve();
        }
        IcyEmState { access, columns }
    }
}

// ========================================================================= //

#[cfg(test)]
mod tests {
    use toml;

    use super::{IcyEmState, COLUMNS_SPEC};
    use crate::save::util::{Tomlable, ACCESS_KEY};
    use crate::save::Access;

    #[test]
    fn toml_round_trip() {
        let mut state = IcyEmState::from_toml(toml::Value::Boolean(false));
        state.access = Access::Replaying;
        state.rotate_column(3, 1);
        state.rotate_column(1, 2);
        state.rotate_column(4, 3);
        assert!(!state.columns.is_solved());
        assert!(state.columns.can_reset());
        let old_positions: Vec<i32> = (0..state.columns().num_columns())
            .map(|col| state.columns().column_position(col))
            .collect();

        let state = IcyEmState::from_toml(state.to_toml());
        assert_eq!(state.access, Access::Replaying);
        assert!(!state.columns.is_solved());
        assert!(state.columns.can_reset());
        let new_positions: Vec<i32> = (0..state.columns().num_columns())
            .map(|col| state.columns().column_position(col))
            .collect();
        assert_eq!(new_positions, old_positions);
    }

    #[test]
    fn from_empty_toml() {
        let state = IcyEmState::from_toml(toml::Value::Boolean(false));
        assert_eq!(state.access, Access::Unvisited);
        assert!(!state.columns.is_solved());
        assert!(!state.columns.can_reset());
    }

    #[test]
    fn from_solved_toml() {
        let mut table = toml::value::Table::new();
        table.insert(ACCESS_KEY.to_string(), Access::Solved.to_toml());

        let state = IcyEmState::from_toml(toml::Value::Table(table));
        assert_eq!(state.access, Access::Solved);
        assert!(state.columns.is_solved());
        let actual_positions: Vec<i32> = (0..state.columns().num_columns())
            .map(|col| state.columns().column_position(col))
            .collect();
        let solved_positions: Vec<i32> = (0..state.columns().num_columns())
            .map(|col| COLUMNS_SPEC[col].2)
            .collect();
        assert_eq!(actual_positions, solved_positions);
    }
}

// ========================================================================= //
