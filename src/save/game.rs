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

use std::default::Default;
use toml;

use super::access::Access;
use super::location::Location;
use super::puzzles::{AtticState, DisconState, DotsState, GroundState,
                     MissedState, PrologState, PuzzleState, WreckedState};
use super::util::{pop_table, to_table};

// ========================================================================= //

const LOCATION_KEY: &'static str = "location";

// ========================================================================= //

#[derive(Default)]
pub struct Game {
    pub location: Location,
    pub prolog: PrologState,
    pub a_light_in_the_attic: AtticState,
    pub connect_the_dots: DotsState,
    pub disconnected: DisconState,
    pub missed_connections: MissedState,
    pub shifting_ground: GroundState,
    pub wrecked_angle: WreckedState,
}

impl Game {
    pub fn new() -> Game {
        let mut game: Game = Default::default();
        game.location = Location::Prolog;
        game
    }

    pub fn from_toml(value: toml::Value) -> Game {
        let mut table = to_table(value);
        let prolog = pop_table(&mut table, Location::Prolog.key());
        let attic = pop_table(&mut table, Location::ALightInTheAttic.key());
        let dots = pop_table(&mut table, Location::ConnectTheDots.key());
        let discon = pop_table(&mut table, Location::Disconnected.key());
        let missed = pop_table(&mut table, Location::MissedConnections.key());
        let ground = pop_table(&mut table, Location::ShiftingGround.key());
        let wrecked = pop_table(&mut table, Location::WreckedAngle.key());
        Game {
            location: Location::from_toml(table.get(LOCATION_KEY)),
            prolog: PrologState::from_toml(prolog),
            a_light_in_the_attic: AtticState::from_toml(attic),
            connect_the_dots: DotsState::from_toml(dots),
            disconnected: DisconState::from_toml(discon),
            missed_connections: MissedState::from_toml(missed),
            shifting_ground: GroundState::from_toml(ground),
            wrecked_angle: WreckedState::from_toml(wrecked),
        }
    }

    pub fn to_toml(&self) -> toml::Value {
        let mut table = toml::Table::new();
        table.insert(LOCATION_KEY.to_string(), self.location.to_toml());
        table.insert(Location::Prolog.key().to_string(),
                     self.prolog.to_toml());
        table.insert(Location::ALightInTheAttic.key().to_string(),
                     self.a_light_in_the_attic.to_toml());
        table.insert(Location::ConnectTheDots.key().to_string(),
                     self.connect_the_dots.to_toml());
        table.insert(Location::Disconnected.key().to_string(),
                     self.disconnected.to_toml());
        table.insert(Location::MissedConnections.key().to_string(),
                     self.missed_connections.to_toml());
        table.insert(Location::ShiftingGround.key().to_string(),
                     self.shifting_ground.to_toml());
        table.insert(Location::WreckedAngle.key().to_string(),
                     self.wrecked_angle.to_toml());
        toml::Value::Table(table)
    }

    pub fn is_unlocked(&self, location: Location) -> bool {
        match location {
            Location::Map => true,
            Location::Prolog => true,
            Location::ALightInTheAttic => self.is_solved(Location::Prolog),
            Location::ConnectTheDots => self.is_solved(Location::Disconnected),
            Location::Disconnected => self.is_solved(Location::Prolog),
            Location::MissedConnections => {
                self.is_solved(Location::ConnectTheDots)
            }
            Location::ShiftingGround => self.is_solved(Location::WreckedAngle),
            Location::WreckedAngle => self.is_solved(Location::Prolog),
        }
    }

    pub fn is_solved(&self, location: Location) -> bool {
        self.access(location) >= Access::Solved
    }

    pub fn access(&self, location: Location) -> Access {
        match location {
            Location::Map => Access::Solved,
            Location::Prolog => self.prolog.access(),
            Location::ALightInTheAttic => self.a_light_in_the_attic.access(),
            Location::ConnectTheDots => self.connect_the_dots.access(),
            Location::Disconnected => self.disconnected.access(),
            Location::MissedConnections => self.missed_connections.access(),
            Location::ShiftingGround => self.shifting_ground.access(),
            Location::WreckedAngle => self.wrecked_angle.access(),
        }
    }
}

// ========================================================================= //
