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

use std::cmp;
use std::collections::{HashMap, HashSet};
use toml;

use gui::{Point, Rect};
use save::util::{to_array, to_i32};

// ========================================================================= //

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PlaneObj {
    Wall,
    Cross,
    PurpleNode,
    RedNode,
    BlueNode,
}

impl PlaneObj {
    pub fn is_node(self) -> bool {
        match self {
            PlaneObj::Wall | PlaneObj::Cross => false,
            PlaneObj::PurpleNode | PlaneObj::RedNode | PlaneObj::BlueNode => {
                true
            }
        }
    }
}

#[derive(Clone, Copy)]
enum PipePiece {
    EmptyOrNode(bool), // false for empty, true for node
    Start(usize), // pipe index
    Middle(usize, usize), // pipe index, piece index
    End(usize), // pipe index
}

// ========================================================================= //

pub struct PlaneGrid {
    rect: Rect,
    objects: HashMap<Point, PlaneObj>,
    pipes: Vec<Vec<Point>>,
}

impl PlaneGrid {
    pub fn new(rect: Rect) -> PlaneGrid {
        PlaneGrid {
            rect: rect,
            objects: HashMap::new(),
            pipes: Vec::new(),
        }
    }

    pub fn pipes_to_toml(&self) -> toml::Value {
        let mut pipes_toml = toml::value::Array::new();
        for pipe in self.pipes.iter() {
            let mut pipe_toml = toml::value::Array::new();
            for &coords in pipe.iter() {
                let mut coords_toml = toml::value::Array::new();
                coords_toml.push(toml::Value::Integer(coords.x() as i64));
                coords_toml.push(toml::Value::Integer(coords.y() as i64));
                pipe_toml.push(toml::Value::Array(coords_toml));
            }
            pipes_toml.push(toml::Value::Array(pipe_toml));
        }
        toml::Value::Array(pipes_toml)
    }

    pub fn set_pipes_from_toml(&mut self, pipes: toml::value::Array) {
        self.pipes.clear();
        for pipe in pipes.into_iter() {
            let pipe = to_array(pipe);
            if !pipe.is_empty() {
                let mut pipe = pipe.into_iter();
                let mut p1 = point_from_toml(pipe.next().unwrap());
                for p2 in pipe {
                    let p2 = point_from_toml(p2);
                    self.toggle_pipe(p1, p2);
                    p1 = p2;
                }
            }
        }
    }

    pub fn rect(&self) -> Rect { self.rect }

    pub fn width(&self) -> u32 { self.rect.width() }

    pub fn height(&self) -> u32 { self.rect.height() }

    pub fn objects(&self) -> &HashMap<Point, PlaneObj> { &self.objects }

    pub fn place_object(&mut self, col: i32, row: i32, obj: PlaneObj) {
        let pt = Point::new(col, row);
        debug_assert!(self.rect.contains(pt));
        self.pipes.retain(|pipe| pipe.iter().all(|&coords| coords != pt));
        self.objects.insert(pt, obj);
    }

    pub fn remove_object(&mut self, col: i32, row: i32) {
        let pt = Point::new(col, row);
        debug_assert!(self.objects.contains_key(&pt));
        self.objects.remove(&pt);
    }

    pub fn pipes(&self) -> &Vec<Vec<Point>> { &self.pipes }

    pub fn remove_all_pipes(&mut self) { self.pipes.clear(); }

    fn pipe_piece_at(&self, coords: Point, is_vertical: bool) -> PipePiece {
        let obj = self.objects.get(&coords).cloned();
        if obj.map(PlaneObj::is_node).unwrap_or(false) {
            return PipePiece::EmptyOrNode(true);
        }
        let is_cross = obj == Some(PlaneObj::Cross);
        for (pipe_index, pipe) in self.pipes.iter().enumerate() {
            for (piece_index, &piece_coords) in pipe.iter().enumerate() {
                if piece_coords == coords {
                    if is_cross {
                        let piece_index2 =
                            if piece_index == 0 { 1 } else { piece_index - 1 };
                        let piece_coords2 = pipe[piece_index2];
                        let vertical2 = piece_coords2.x() == piece_coords.x();
                        if is_vertical != vertical2 {
                            continue;
                        }
                    }
                    return if piece_index == 0 {
                        PipePiece::Start(pipe_index)
                    } else if piece_index + 1 == pipe.len() {
                        PipePiece::End(pipe_index)
                    } else {
                        PipePiece::Middle(pipe_index, piece_index)
                    };
                }
            }
        }
        PipePiece::EmptyOrNode(false)
    }

    fn is_wall_at(&self, coords: Point) -> bool {
        self.objects.get(&coords) == Some(&PlaneObj::Wall)
    }

    pub fn toggle_pipe(&mut self, coords1: Point, coords2: Point) -> bool {
        if !self.rect.contains(coords1) || !self.rect.contains(coords2) {
            return false;
        }
        let dx = coords2.x() - coords1.x();
        let dy = coords2.y() - coords1.y();
        if !(dx == 0 && dy.abs() == 1 || dy == 0 && dx.abs() == 1) {
            return false;
        }
        if self.is_wall_at(coords1) || self.is_wall_at(coords2) {
            return false;
        }
        let is_vertical = dx == 0;
        match (self.pipe_piece_at(coords1, is_vertical),
               self.pipe_piece_at(coords2, is_vertical)) {
            (PipePiece::EmptyOrNode(node1), PipePiece::EmptyOrNode(node2)) => {
                if node1 && node2 {
                    for p in 0..self.pipes.len() {
                        if self.pipes[p].len() == 2 &&
                           ((self.pipes[p][0] == coords1 &&
                             self.pipes[p][1] == coords2) ||
                            (self.pipes[p][0] == coords2 &&
                             self.pipes[p][1] == coords1)) {
                            self.pipes.swap_remove(p);
                            return true;
                        }
                    }
                }
                self.pipes.push(vec![coords1, coords2]);
            }
            (PipePiece::EmptyOrNode(is_node), PipePiece::Start(p2)) => {
                if is_node && self.pipes[p2][1] == coords1 {
                    debug_assert_eq!(self.pipes[p2].len(), 2);
                    self.pipes.swap_remove(p2);
                } else {
                    self.pipes[p2].insert(0, coords1);
                }
            }
            (PipePiece::EmptyOrNode(is_node), PipePiece::End(p2)) => {
                if is_node && self.pipes[p2].len() == 2 &&
                   self.pipes[p2][0] == coords1 {
                    self.pipes.swap_remove(p2);
                } else {
                    self.pipes[p2].push(coords1);
                }
            }
            (PipePiece::Start(p1), PipePiece::EmptyOrNode(is_node)) => {
                if is_node && self.pipes[p1][1] == coords2 {
                    debug_assert_eq!(self.pipes[p1].len(), 2);
                    self.pipes.swap_remove(p1);
                } else {
                    self.pipes[p1].insert(0, coords2);
                }
            }
            (PipePiece::End(p1), PipePiece::EmptyOrNode(is_node)) => {
                if is_node && self.pipes[p1].len() == 2 &&
                   self.pipes[p1][0] == coords2 {
                    self.pipes.swap_remove(p1);
                } else {
                    self.pipes[p1].push(coords2);
                }
            }
            (PipePiece::Start(p1), PipePiece::Start(p2)) => {
                debug_assert_ne!(p1, p2);
                let (p1, p2) = (cmp::min(p1, p2), cmp::max(p1, p2));
                let mut pipe2 = self.pipes.swap_remove(p2);
                let pipe1 = &mut self.pipes[p1];
                pipe1.reverse();
                pipe1.append(&mut pipe2);
            }
            (PipePiece::End(p1), PipePiece::End(p2)) => {
                debug_assert_ne!(p1, p2);
                let (p1, p2) = (cmp::min(p1, p2), cmp::max(p1, p2));
                let mut pipe2 = self.pipes.swap_remove(p2);
                pipe2.reverse();
                self.pipes[p1].append(&mut pipe2);
            }
            (PipePiece::End(p1), PipePiece::Start(p2)) |
            (PipePiece::Start(p2), PipePiece::End(p1)) => {
                if p1 == p2 {
                    if self.pipes[p1].len() == 2 {
                        self.pipes.swap_remove(p1);
                    } else {
                        return false;
                    }
                } else if p1 < p2 {
                    let mut pipe2 = self.pipes.swap_remove(p2);
                    self.pipes[p1].append(&mut pipe2);
                } else {
                    let mut pipe1 = self.pipes.swap_remove(p1);
                    let mut pipe2 = self.pipes.swap_remove(p2);
                    pipe1.append(&mut pipe2);
                    self.pipes.push(pipe1);
                }
            }
            (PipePiece::Start(p1), PipePiece::Middle(p2, i2)) |
            (PipePiece::Middle(p2, i2), PipePiece::Start(p1)) => {
                if p1 == p2 && i2 == 1 {
                    self.pipes[p1].remove(0);
                } else {
                    return false;
                }
            }
            (PipePiece::Middle(p1, i1), PipePiece::End(p2)) |
            (PipePiece::End(p2), PipePiece::Middle(p1, i1)) => {
                if p1 == p2 && i1 + 2 == self.pipes[p1].len() {
                    self.pipes[p1].pop();
                } else {
                    return false;
                }
            }
            (PipePiece::Middle(p1, i1), PipePiece::Middle(p2, i2)) => {
                if p1 != p2 {
                    return false;
                }
                let (i1, i2) = (cmp::min(i1, i2), cmp::max(i1, i2));
                if i1 + 1 != i2 {
                    return false;
                }
                let pipe = self.pipes[p1].split_off(i2);
                self.pipes.push(pipe);
            }
            (PipePiece::Middle(p1, i1), PipePiece::EmptyOrNode(is_node)) => {
                if is_node && i1 == 1 && self.pipes[p1][0] == coords2 {
                    self.pipes[p1].remove(0);
                } else if is_node && i1 + 2 == self.pipes[p1].len() &&
                          self.pipes[p1][i1 + 1] == coords2 {
                    self.pipes[p1].pop();
                } else {
                    return false;
                }
            }
            (PipePiece::EmptyOrNode(is_node), PipePiece::Middle(p2, i2)) => {
                if is_node && i2 == 1 && self.pipes[p2][0] == coords1 {
                    self.pipes[p2].remove(0);
                } else if is_node && i2 + 2 == self.pipes[p2].len() &&
                          self.pipes[p2][i2 + 1] == coords1 {
                    self.pipes[p2].pop();
                } else {
                    return false;
                }
            }
        }
        true
    }

    pub fn all_nodes_are_connected(&self) -> bool {
        let mut purple_nodes = Vec::new();
        let mut red_nodes = Vec::new();
        let mut blue_nodes = Vec::new();
        for (&pt, &obj) in self.objects.iter() {
            match obj {
                PlaneObj::PurpleNode => purple_nodes.push(pt),
                PlaneObj::RedNode => red_nodes.push(pt),
                PlaneObj::BlueNode => blue_nodes.push(pt),
                _ => {}
            }
        }
        let mut node_pairs = HashSet::new();
        for (index1, node1) in purple_nodes.iter().enumerate() {
            for node2 in purple_nodes[(index1 + 1)..].iter() {
                node_pairs.insert((node1, node2));
            }
        }
        for node1 in red_nodes.iter() {
            for node2 in blue_nodes.iter() {
                node_pairs.insert((node1, node2));
            }
        }
        for pipe in self.pipes.iter() {
            debug_assert!(!pipe.is_empty());
            let start = pipe.first().unwrap();
            let end = pipe.last().unwrap();
            node_pairs.remove(&(start, end));
            node_pairs.remove(&(end, start));
        }
        node_pairs.is_empty()
    }
}

// ========================================================================= //

fn point_from_toml(value: toml::Value) -> Point {
    let mut array = to_array(value);
    if array.len() < 2 {
        return Point::new(0, 0);
    }
    array.truncate(2);
    let y = to_i32(array.pop().unwrap());
    let x = to_i32(array.pop().unwrap());
    Point::new(x, y)
}

// ========================================================================= //