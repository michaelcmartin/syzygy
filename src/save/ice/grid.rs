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

use std::collections::HashMap;
use toml;

use crate::gui::Point;
use crate::save::ice::{Symbol, Transform};
use crate::save::util::{pop_array, to_table, Tomlable};
use crate::save::Direction;

// ========================================================================= //

const BLOCKS_KEY: &str = "blocks";
const PUSH_POPS_KEY: &str = "push_pops";
const COL_KEY: &str = "col";
const ROW_KEY: &str = "row";
const DIRECTION_KEY: &str = "direction";
const SYMBOL_KEY: &str = "symbol";

// ========================================================================= //

#[derive(Clone, Debug)]
pub struct BlockSlide {
    from: Point,
    direction: Direction,
    to: Point,
    pushed: Option<Point>,
    transform: Transform,
}

impl BlockSlide {
    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn to_coords(&self) -> Point {
        self.to
    }

    pub fn distance(&self) -> i32 {
        if self.to.y() == self.from.y() {
            (self.to.x() - self.from.x()).abs()
        } else {
            debug_assert_eq!(self.to.x(), self.from.x());
            (self.to.y() - self.from.y()).abs()
        }
    }

    pub fn pushed(&self) -> Option<Point> {
        self.pushed
    }

    pub fn transform(&self) -> Transform {
        self.transform
    }
}

// ========================================================================= //

#[derive(Clone)]
pub struct ObjectGrid {
    num_cols: i32,
    num_rows: i32,
    objects: HashMap<Point, Object>,
    ice_blocks: HashMap<Point, Symbol>,
    is_modified: bool,
}

impl ObjectGrid {
    pub fn new(num_cols: usize, num_rows: usize) -> ObjectGrid {
        ObjectGrid {
            num_cols: num_cols as i32,
            num_rows: num_rows as i32,
            objects: HashMap::new(),
            ice_blocks: HashMap::new(),
            is_modified: false,
        }
    }

    pub fn from_toml(
        mut table: toml::value::Table,
        default: &ObjectGrid,
    ) -> ObjectGrid {
        let mut blocks = Vec::new();
        for block_toml in pop_array(&mut table, BLOCKS_KEY).into_iter() {
            let mut block_toml = to_table(block_toml);
            let col = i32::pop_from_table(&mut block_toml, COL_KEY);
            let row = i32::pop_from_table(&mut block_toml, ROW_KEY);
            let symbol = Symbol::pop_from_table(&mut block_toml, SYMBOL_KEY);
            if (col < 0 || col >= default.num_cols)
                || (row < 0 || row >= default.num_rows)
            {
                return default.clone();
            }
            blocks.push((col, row, symbol));
        }
        if blocks.len() != default.ice_blocks.len() {
            return default.clone();
        }

        let mut push_pops = Vec::new();
        for pp_toml in pop_array(&mut table, PUSH_POPS_KEY).into_iter() {
            let mut pp_toml = to_table(pp_toml);
            let col = i32::pop_from_table(&mut pp_toml, COL_KEY);
            let row = i32::pop_from_table(&mut pp_toml, ROW_KEY);
            let dir = Direction::pop_from_table(&mut pp_toml, DIRECTION_KEY);
            push_pops.push((col, row, dir));
        }

        let mut grid = default.clone();
        grid.objects.retain(|_, obj| match obj {
            &mut Object::PushPop(_) => false,
            _ => true,
        });

        for (col, row, dir) in push_pops.into_iter() {
            if grid.objects.contains_key(&Point::new(col, row)) {
                return default.clone();
            }
            grid.add_object(col, row, Object::PushPop(dir));
        }

        grid.ice_blocks.clear();
        for (col, row, symbol) in blocks.into_iter() {
            if grid.ice_blocks.contains_key(&Point::new(col, row)) {
                return default.clone();
            }
            grid.add_ice_block(col, row, symbol);
        }
        grid.is_modified = grid.ice_blocks != default.ice_blocks
            || grid.objects != default.objects;
        grid
    }

    pub fn to_toml(&self) -> toml::Value {
        let mut table = toml::value::Table::new();
        let mut blocks = toml::value::Array::new();
        for (&coords, &symbol) in self.ice_blocks.iter() {
            let mut block = toml::value::Table::new();
            block.insert(
                COL_KEY.to_string(),
                toml::Value::Integer(coords.x() as i64),
            );
            block.insert(
                ROW_KEY.to_string(),
                toml::Value::Integer(coords.y() as i64),
            );
            block.insert(SYMBOL_KEY.to_string(), symbol.to_toml());
            blocks.push(toml::Value::Table(block));
        }
        table.insert(BLOCKS_KEY.to_string(), toml::Value::Array(blocks));
        let mut push_pops = toml::value::Array::new();
        for (&coords, object) in self.objects.iter() {
            if let &Object::PushPop(direction) = object {
                let mut push_pop = toml::value::Table::new();
                push_pop.insert(
                    COL_KEY.to_string(),
                    toml::Value::Integer(coords.x() as i64),
                );
                push_pop.insert(
                    ROW_KEY.to_string(),
                    toml::Value::Integer(coords.y() as i64),
                );
                push_pop
                    .insert(DIRECTION_KEY.to_string(), direction.to_toml());
                push_pops.push(toml::Value::Table(push_pop));
            }
        }
        table.insert(PUSH_POPS_KEY.to_string(), toml::Value::Array(push_pops));
        toml::Value::Table(table)
    }

    pub fn size(&self) -> (i32, i32) {
        (self.num_cols, self.num_rows)
    }

    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    pub fn add_object(&mut self, col: i32, row: i32, obj: Object) {
        debug_assert!(col >= 0 && col < self.num_cols);
        debug_assert!(row >= 0 && row < self.num_rows);
        let coords = Point::new(col, row);
        debug_assert!(!self.objects.contains_key(&coords));
        self.objects.insert(coords, obj);
    }

    pub fn objects(&self) -> &HashMap<Point, Object> {
        &self.objects
    }

    pub fn add_ice_block(&mut self, col: i32, row: i32, symbol: Symbol) {
        debug_assert!(col >= 0 && col < self.num_cols);
        debug_assert!(row >= 0 && row < self.num_rows);
        let coords = Point::new(col, row);
        debug_assert!(!self.ice_blocks.contains_key(&coords));
        self.ice_blocks.insert(coords, symbol);
    }

    pub fn ice_blocks(&self) -> &HashMap<Point, Symbol> {
        &self.ice_blocks
    }

    pub fn slide_ice_block(
        &mut self,
        coords: Point,
        slide_dir: Direction,
    ) -> Option<BlockSlide> {
        if let Some(symbol) = self.ice_blocks.remove(&coords) {
            let delta = slide_dir.delta();
            let mut new_coords = coords;
            let mut pushed = None;
            let mut transform = Transform::identity();
            loop {
                let next = new_coords + delta;
                if (next.x() < 0 || next.x() >= self.num_cols)
                    || (next.y() < 0 || next.y() >= self.num_rows)
                    || self.ice_blocks.contains_key(&next)
                {
                    break;
                }
                match self.objects.get(&next).cloned() {
                    Some(Object::Gap) | Some(Object::Wall) => break,
                    Some(Object::PushPop(pp_dir)) => {
                        if pp_dir != slide_dir.opposite() {
                            break;
                        }
                        let mut pp_coords = next + delta;
                        while self.objects.contains_key(&pp_coords) {
                            pp_coords = pp_coords + delta;
                        }
                        if self.ice_blocks.contains_key(&pp_coords) {
                            break;
                        }
                        self.objects.remove(&next);
                        self.objects
                            .insert(pp_coords, Object::PushPop(slide_dir));
                        pushed = Some(pp_coords);
                    }
                    Some(Object::Rotator) => {
                        transform = transform.rotated_cw();
                    }
                    Some(Object::Reflector(false)) => {
                        transform = transform.flipped_horz();
                    }
                    Some(Object::Reflector(true)) => {
                        transform = transform.flipped_vert();
                    }
                    Some(Object::Goal(_)) => {}
                    None => {}
                }
                new_coords = next;
            }
            debug_assert!(!self.ice_blocks.contains_key(&new_coords));
            self.ice_blocks.insert(new_coords, symbol.transformed(transform));
            if new_coords != coords {
                self.is_modified = true;
                let slide = BlockSlide {
                    from: coords,
                    direction: slide_dir,
                    to: new_coords,
                    pushed,
                    transform,
                };
                return Some(slide);
            }
        }
        None
    }

    pub fn undo_slide(&mut self, slide: &BlockSlide) {
        if let Some(symbol) = self.ice_blocks.remove(&slide.to) {
            let symbol = symbol.transformed(slide.transform.inverse());
            self.ice_blocks.insert(slide.from, symbol);
            if let Some(pp_coords) = slide.pushed {
                if let Some(&Object::PushPop(pp_dir)) =
                    self.objects.get(&pp_coords)
                {
                    let delta = pp_dir.opposite().delta();
                    let mut new_pp_coords = pp_coords + delta;
                    while self.objects.contains_key(&new_pp_coords) {
                        new_pp_coords = new_pp_coords + delta;
                    }
                    self.objects.remove(&pp_coords);
                    self.objects.insert(
                        new_pp_coords,
                        Object::PushPop(pp_dir.opposite()),
                    );
                }
            }
        }
    }

    pub fn redo_slide(&mut self, slide: &BlockSlide) {
        self.slide_ice_block(slide.from, slide.direction);
    }

    pub fn all_blocks_on_goals(&self) -> bool {
        for (coords, &block_sym) in self.ice_blocks.iter() {
            match self.objects.get(coords) {
                Some(&Object::Goal(goal_sym)) if goal_sym == block_sym => {}
                _ => return false,
            }
        }
        true
    }

    pub fn solved(mut self) -> ObjectGrid {
        self.ice_blocks.clear();
        for (&coords, object) in self.objects.iter() {
            if let &Object::Goal(symbol) = object {
                self.ice_blocks.insert(coords, symbol);
            }
        }
        self.is_modified = true;
        self
    }
}

// ========================================================================= //

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Object {
    Gap,
    Wall,
    PushPop(Direction),
    Rotator,
    Reflector(bool),
    Goal(Symbol),
}

// ========================================================================= //
