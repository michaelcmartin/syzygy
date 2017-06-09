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

use std::collections::HashSet;
use toml;

use gui::Rect;
use save::{Access, Direction, Location, PrimaryColor};
use save::column::Columns;
use save::device::{Device, DeviceGrid};
use save::ice::{Object, ObjectGrid, Symbol, Transform};
use save::plane::{PlaneGrid, PlaneObj};
use super::PuzzleState;
use super::super::util::{ACCESS_KEY, pop_array, pop_table, to_i32};

// ========================================================================= //

const STAGE_KEY: &str = "stage";
const YTTRIS_KEY: &str = "yttris";
const ARGONY_KEY: &str = "argony";
const ELINSA_KEY: &str = "elinsa";
const UGRENT_KEY: &str = "ugrent";
const RELYNG_LIGHTS_KEY: &str = "relyng_lights";
const RELYNG_NEXT_KEY: &str = "relyng_next";
// const MEZURE_KEY: &str = "mezure";

#[cfg_attr(rustfmt, rustfmt_skip)]
const YTTRIS_COLUMNS_SPEC: &[(&str, i32, i32, &[(usize, i32)])] = &[
    ("UNDO", -1, 3, &[(0, 1), (4,  2), (3,  3)]),
    ("OPEN", -1, 2, &[(1, 1), (3, -2), (4,  3)]),
    ("FILE", -1, 1, &[(2, 1), (0,  2), (5,  3)]),
    ("BOLT", -1, 0, &[(3, 1), (5, -2), (1, -3)]),
    ("PICK", -1, 1, &[(4, 1),          (0, -3)]),
    ("KEYS", -1, 3, &[(5, 1), (2, -2), (4,  3)]),
];

const RELYNG_NUM_COLS: i32 = 5;
const RELYNG_NUM_ROWS: i32 = 4;
const RELYNG_INIT_NEXT: char = '+';

// ========================================================================= //

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyzygyStage {
    Yttris,
    Argony,
    Elinsa,
    Ugrent,
    Relyng,
    Mezure,
}

impl SyzygyStage {
    pub fn first() -> SyzygyStage { SyzygyStage::Yttris }

    pub fn from_toml(value: Option<&toml::Value>) -> SyzygyStage {
        if let Some(string) = value.and_then(toml::Value::as_str) {
            match string {
                "yttris" => return SyzygyStage::Yttris,
                "argony" => return SyzygyStage::Argony,
                "elinsa" => return SyzygyStage::Elinsa,
                "ugrent" => return SyzygyStage::Ugrent,
                "relyng" => return SyzygyStage::Relyng,
                "mezure" => return SyzygyStage::Mezure,
                _ => {}
            }
        }
        SyzygyStage::first()
    }

    pub fn to_toml(self) -> toml::Value {
        let string = match self {
            SyzygyStage::Yttris => "yttris",
            SyzygyStage::Argony => "argony",
            SyzygyStage::Elinsa => "elinsa",
            SyzygyStage::Ugrent => "ugrent",
            SyzygyStage::Relyng => "relyng",
            SyzygyStage::Mezure => "mezure",
        };
        toml::Value::String(string.to_string())
    }
}

// ========================================================================= //

pub struct SyzygyState {
    access: Access,
    stage: SyzygyStage,
    yttris: Columns,
    argony: ObjectGrid,
    elinsa: PlaneGrid,
    ugrent: DeviceGrid,
    relyng_lights: HashSet<i32>,
    relyng_next: char,
}

impl SyzygyState {
    fn argony_initial_grid() -> ObjectGrid {
        let q_goal = Symbol::CyanQ(Transform::identity());
        let u_goal = Symbol::CyanU(Transform::identity());
        let a_goal = Symbol::CyanA(Transform::identity());
        let d_goal = Symbol::CyanQ(Transform::identity().flipped_vert());
        let mut grid = ObjectGrid::new(11, 5);
        grid.add_object(5, 0, Object::Wall);
        grid.add_object(1, 1, Object::Wall);
        grid.add_object(5, 1, Object::Wall);
        grid.add_object(1, 2, Object::Rotator);
        grid.add_object(5, 2, Object::Reflector(false));
        grid.add_object(10, 2, Object::Wall);
        grid.add_object(1, 3, Object::Wall);
        grid.add_object(5, 3, Object::Wall);
        grid.add_object(10, 3, Object::PushPop(Direction::South));
        grid.add_object(2, 4, Object::Goal(q_goal));
        grid.add_object(4, 4, Object::Goal(u_goal));
        grid.add_object(5, 4, Object::Wall);
        grid.add_object(6, 4, Object::Goal(a_goal));
        grid.add_object(8, 4, Object::Goal(d_goal));

        let q_trans = Transform::identity().rotated_cw().rotated_cw();
        let u_trans = Transform::identity().flipped_vert();
        let a_trans = Transform::identity().rotated_cw().rotated_cw();
        let d_trans = Transform::identity().flipped_vert();
        grid.add_ice_block(6, 0, Symbol::CyanQ(q_trans));
        grid.add_ice_block(7, 0, Symbol::CyanA(a_trans));
        grid.add_ice_block(8, 0, Symbol::CyanU(u_trans));
        grid.add_ice_block(9, 0, Symbol::CyanQ(d_trans));
        grid
    }

    fn elinsa_initial_grid() -> PlaneGrid {
        let mut grid = PlaneGrid::new(Rect::new(0, 0, 12, 6));
        grid.place_object(0, 0, PlaneObj::Wall);
        grid.place_object(9, 1, PlaneObj::BlueNode);
        grid.place_object(10, 1, PlaneObj::Wall);
        grid.place_object(2, 2, PlaneObj::PurpleNode);
        grid.place_object(9, 2, PlaneObj::Cross);
        grid.place_object(5, 3, PlaneObj::RedNode);
        grid.place_object(9, 3, PlaneObj::Cross);
        grid.place_object(11, 3, PlaneObj::BlueNode);
        grid.place_object(3, 4, PlaneObj::Wall);
        grid.place_object(1, 5, PlaneObj::RedNode);
        grid
    }

    fn ugrent_initial_grid() -> DeviceGrid {
        let mut grid = DeviceGrid::new(7, 5);
        grid.set(0, 0, Device::Emitter(PrimaryColor::Red), Direction::South);
        grid.set(3, 0, Device::Detector(PrimaryColor::Red), Direction::East);
        grid.set(6, 2, Device::Detector(PrimaryColor::Green), Direction::West);
        grid.set(3, 3, Device::Wall, Direction::East);
        grid.set(0, 4, Device::Emitter(PrimaryColor::Green), Direction::North);
        grid.set(3, 4, Device::Detector(PrimaryColor::Blue), Direction::East);

        grid.set(2, 0, Device::Mirror, Direction::East);
        grid.set(2, 1, Device::Mirror, Direction::South);
        grid.set(2, 2, Device::Mirror, Direction::East);
        grid.set(2, 3, Device::Mirror, Direction::South);
        grid.set(2, 4, Device::Mirror, Direction::East);
        grid.set(3, 1, Device::Mixer, Direction::East);
        grid.set(3, 2, Device::Mixer, Direction::East);
        grid.set(4, 1, Device::Splitter, Direction::East);
        grid.set(4, 2, Device::Splitter, Direction::East);
        grid.set(4, 3, Device::Splitter, Direction::East);
        grid.set(5, 0, Device::Mirror, Direction::South);
        grid.set(5, 1, Device::Mirror, Direction::East);
        grid.set(5, 2, Device::Mirror, Direction::South);
        grid.set(5, 3, Device::Mirror, Direction::East);
        grid.set(5, 4, Device::Mirror, Direction::South);
        grid
    }

    pub fn from_toml(mut table: toml::value::Table) -> SyzygyState {
        let access = Access::from_toml(table.get(ACCESS_KEY));
        let stage = SyzygyStage::from_toml(table.get(STAGE_KEY));
        let yttris = Columns::from_toml(YTTRIS_COLUMNS_SPEC,
                                        pop_array(&mut table, YTTRIS_KEY));
        let argony =
            ObjectGrid::from_toml(pop_table(&mut table, ARGONY_KEY),
                                  &SyzygyState::argony_initial_grid());

        let mut elinsa = SyzygyState::elinsa_initial_grid();
        elinsa.set_pipes_from_toml(pop_array(&mut table, ELINSA_KEY));
        let ugrent =
            DeviceGrid::from_toml(pop_array(&mut table, UGRENT_KEY),
                                  &SyzygyState::ugrent_initial_grid());
        let relyng_next = match table.get(RELYNG_NEXT_KEY)
                                     .and_then(toml::Value::as_str)
                                     .unwrap_or("") {
            "+" => '+',
            "N" => 'N',
            "X" => 'X',
            "Z" => 'Z',
            _ => RELYNG_INIT_NEXT,
        };
        let relyng_lights =
            pop_array(&mut table, RELYNG_LIGHTS_KEY)
                .into_iter()
                .map(to_i32)
                .filter(|&idx| {
                    0 <= idx && idx < RELYNG_NUM_COLS * RELYNG_NUM_ROWS
                })
                .collect();
        SyzygyState {
            access: access,
            stage: stage,
            yttris: yttris,
            argony: argony,
            elinsa: elinsa,
            ugrent: ugrent,
            relyng_next: relyng_next,
            relyng_lights: relyng_lights,
        }
    }

    // TODO: Solve stages one at a time.
    pub fn solve(&mut self) { self.access = Access::Solved; }

    pub fn stage(&self) -> SyzygyStage { self.stage }

    pub fn advance_stage_if_done(&mut self) -> bool {
        match self.stage {
            SyzygyStage::Elinsa => {
                if self.elinsa.all_nodes_are_connected() {
                    self.stage = SyzygyStage::Ugrent;
                    return true;
                }
            }
            _ => {} // TODO
        }
        false
    }

    pub fn yttris_columns(&self) -> &Columns { &self.yttris }

    pub fn yttris_columns_mut(&mut self) -> &mut Columns { &mut self.yttris }

    fn reset_yttris(&mut self) { self.yttris.reset() }

    pub fn argony_grid(&self) -> &ObjectGrid { &self.argony }

    pub fn argony_grid_mut(&mut self) -> &mut ObjectGrid { &mut self.argony }

    fn reset_argony(&mut self) {
        self.argony = SyzygyState::argony_initial_grid();
    }

    pub fn elinsa_grid(&self) -> &PlaneGrid { &self.elinsa }

    pub fn elinsa_grid_mut(&mut self) -> &mut PlaneGrid { &mut self.elinsa }

    fn reset_elinsa(&mut self) { self.elinsa.remove_all_pipes() }

    pub fn ugrent_grid(&self) -> &DeviceGrid { &self.ugrent }

    pub fn ugrent_grid_mut(&mut self) -> &mut DeviceGrid { &mut self.ugrent }

    fn reset_ugrent(&mut self) {
        self.ugrent = SyzygyState::ugrent_initial_grid();
    }

    pub fn relyng_is_lit(&self, (col, row): (i32, i32)) -> bool {
        debug_assert!(col >= 0 && col < RELYNG_NUM_COLS);
        debug_assert!(row >= 0 && row < RELYNG_NUM_ROWS);
        !self.relyng_lights.contains(&(row * RELYNG_NUM_COLS + col))
    }

    pub fn relyng_is_done(&self) -> bool {
        self.relyng_lights.len() ==
        (RELYNG_NUM_COLS * RELYNG_NUM_ROWS) as usize
    }

    pub fn relyng_next_shape(&self) -> char { self.relyng_next }

    pub fn relyng_toggle(&mut self, (col, row): (i32, i32)) {
        self.relyng_toggle_shape(col, row);
        self.relyng_next = match self.relyng_next {
            '+' => 'N',
            'N' => 'X',
            'X' => 'Z',
            'Z' => '+',
            _ => unreachable!(),
        };
    }

    pub fn relyng_untoggle(&mut self, (col, row): (i32, i32)) {
        self.relyng_next = match self.relyng_next {
            '+' => 'Z',
            'N' => '+',
            'X' => 'N',
            'Z' => 'X',
            _ => unreachable!(),
        };
        self.relyng_toggle_shape(col, row);
    }

    fn relyng_toggle_shape(&mut self, col: i32, row: i32) {
        match self.relyng_next {
            '+' => {
                self.relyng_toggle_light(col, row);
                self.relyng_toggle_light(col + 1, row);
                self.relyng_toggle_light(col, row + 1);
                self.relyng_toggle_light(col - 1, row);
                self.relyng_toggle_light(col, row - 1);
            }
            'N' => {
                self.relyng_toggle_light(col, row);
                self.relyng_toggle_light(col - 1, row);
                self.relyng_toggle_light(col - 1, row + 1);
                self.relyng_toggle_light(col + 1, row);
                self.relyng_toggle_light(col + 1, row - 1);
            }
            'X' => {
                self.relyng_toggle_light(col, row);
                self.relyng_toggle_light(col - 1, row - 1);
                self.relyng_toggle_light(col + 1, row - 1);
                self.relyng_toggle_light(col - 1, row + 1);
                self.relyng_toggle_light(col + 1, row + 1);
            }
            'Z' => {
                self.relyng_toggle_light(col, row);
                self.relyng_toggle_light(col, row - 1);
                self.relyng_toggle_light(col - 1, row - 1);
                self.relyng_toggle_light(col, row + 1);
                self.relyng_toggle_light(col + 1, row + 1);
            }
            _ => unreachable!(),
        }
    }

    fn relyng_toggle_light(&mut self, col: i32, row: i32) {
        if (col >= 0 && col < RELYNG_NUM_COLS) &&
           (row >= 0 && row < RELYNG_NUM_ROWS) {
            let index = row * RELYNG_NUM_COLS + col;
            if !self.relyng_lights.remove(&index) {
                self.relyng_lights.insert(index);
            }
        }
    }

    fn reset_relyng(&mut self) {
        self.relyng_lights.clear();
        self.relyng_next = RELYNG_INIT_NEXT;
    }
}

impl PuzzleState for SyzygyState {
    fn location(&self) -> Location { Location::SystemSyzygy }

    fn access(&self) -> Access { self.access }

    fn access_mut(&mut self) -> &mut Access { &mut self.access }

    fn can_reset(&self) -> bool {
        match self.stage {
            SyzygyStage::Yttris => self.yttris.can_reset(),
            SyzygyStage::Argony => self.argony.is_modified(),
            SyzygyStage::Elinsa => !self.elinsa.pipes().is_empty(),
            SyzygyStage::Ugrent => self.ugrent.is_modified(),
            SyzygyStage::Relyng => !self.relyng_lights.is_empty(),
            _ => false, // TODO
        }
    }

    fn reset(&mut self) {
        match self.stage {
            SyzygyStage::Yttris => self.reset_yttris(),
            SyzygyStage::Argony => self.reset_argony(),
            SyzygyStage::Elinsa => self.reset_elinsa(),
            SyzygyStage::Ugrent => self.reset_ugrent(),
            SyzygyStage::Relyng => self.reset_relyng(),
            _ => {} // TODO
        }
    }

    fn replay(&mut self) {
        self.stage = SyzygyStage::first();
        self.reset_yttris();
        self.reset_argony();
        self.reset_elinsa();
        self.reset_ugrent();
        self.reset_relyng();
        // TODO others
        self.access = Access::BeginReplay;
    }

    fn to_toml(&self) -> toml::Value {
        let mut table = toml::value::Table::new();
        table.insert(ACCESS_KEY.to_string(), self.access.to_toml());
        if !self.is_solved() {
            table.insert(STAGE_KEY.to_string(), self.stage.to_toml());
            match self.stage {
                SyzygyStage::Yttris => {
                    table.insert(YTTRIS_KEY.to_string(),
                                 self.yttris.to_toml());
                }
                SyzygyStage::Argony => {
                    if self.argony.is_modified() {
                        table.insert(ARGONY_KEY.to_string(),
                                     self.argony.to_toml());
                    }
                }
                SyzygyStage::Elinsa => {
                    table.insert(ELINSA_KEY.to_string(),
                                 self.elinsa.pipes_to_toml());
                }
                SyzygyStage::Ugrent => {
                    if self.ugrent.is_modified() {
                        table.insert(UGRENT_KEY.to_string(),
                                     self.ugrent.to_toml());
                    }
                }
                SyzygyStage::Relyng => {
                    let lights =
                        self.relyng_lights
                            .iter()
                            .map(|&idx| toml::Value::Integer(idx as i64))
                            .collect();
                    table.insert(RELYNG_LIGHTS_KEY.to_string(),
                                 toml::Value::Array(lights));
                    let mut next = String::new();
                    next.push(self.relyng_next);
                    table.insert(RELYNG_NEXT_KEY.to_string(),
                                 toml::Value::String(next));
                }
                _ => {} // TODO
            }
        }
        toml::Value::Table(table)
    }
}

// ========================================================================= //

#[cfg(test)]
mod tests {
    use super::SyzygyStage;

    const ALL_STAGES: &[SyzygyStage] = &[SyzygyStage::Yttris,
                                         SyzygyStage::Argony,
                                         SyzygyStage::Elinsa,
                                         SyzygyStage::Ugrent,
                                         SyzygyStage::Relyng,
                                         SyzygyStage::Mezure];

    #[test]
    fn stage_toml_round_trip() {
        for &original in ALL_STAGES {
            let result = SyzygyStage::from_toml(Some(&original.to_toml()));
            assert_eq!(result, original);
        }
    }
}

// ========================================================================= //
