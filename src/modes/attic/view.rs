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

use elements::{Hud, HudCmd, HudInput, PuzzleCmd, PuzzleView, Scene,
               ScreenFade, Theater};
use gui::{Action, Canvas, Element, Event, Point, Rect, Resources, Sprite};
use modes::SOLVED_INFO_TEXT;
use save::{AtticState, Game, Location};
use super::scenes::{compile_intro_scene, compile_outro_scene};

// ========================================================================= //

pub struct View {
    theater: Theater,
    intro_scene: Scene,
    outro_scene: Scene,
    screen_fade: ScreenFade<PuzzleCmd>,
    hud: Hud,
    toggles: Vec<ToggleLight>,
    passives: Vec<PassiveLight>,
    undo_stack: Vec<(i32, i32)>,
    redo_stack: Vec<(i32, i32)>,
}

impl View {
    pub fn new(resources: &mut Resources, visible: Rect, state: &AtticState)
               -> View {
        let background = resources.get_background("a_light_in_the_attic");
        let mut theater = Theater::new(background);
        let mut intro_scene = compile_intro_scene(resources);
        let mut outro_scene = compile_outro_scene(resources);
        if state.is_visited() {
            intro_scene.skip(&mut theater);
            if state.is_solved() {
                outro_scene.skip(&mut theater);
            }
        } else {
            intro_scene.begin(&mut theater);
        }
        let mut view = View {
            theater: theater,
            intro_scene: intro_scene,
            outro_scene: outro_scene,
            screen_fade: ScreenFade::new(resources),
            hud: Hud::new(resources, visible, Location::ALightInTheAttic),
            toggles: vec![
                ToggleLight::new(resources, state, (1, 1), 'C'),
                ToggleLight::new(resources, state, (2, 1), 'Z'),
                ToggleLight::new(resources, state, (3, 1), 'H'),
                ToggleLight::new(resources, state, (4, 1), 'A'),
                ToggleLight::new(resources, state, (1, 2), 'U'),
                ToggleLight::new(resources, state, (2, 2), 'V'),
                ToggleLight::new(resources, state, (3, 2), 'X'),
                ToggleLight::new(resources, state, (4, 2), 'S'),
                ToggleLight::new(resources, state, (1, 3), 'J'),
                ToggleLight::new(resources, state, (2, 3), 'T'),
                ToggleLight::new(resources, state, (3, 3), 'I'),
                ToggleLight::new(resources, state, (4, 3), 'K'),
                ToggleLight::new(resources, state, (1, 4), 'Y'),
                ToggleLight::new(resources, state, (2, 4), 'O'),
                ToggleLight::new(resources, state, (3, 4), 'L'),
                ToggleLight::new(resources, state, (4, 4), 'N'),
            ],
            passives: vec![
                PassiveLight::new(resources, state, (1, 0)),
                PassiveLight::new(resources, state, (2, 0)),
                PassiveLight::new(resources, state, (3, 0)),
                PassiveLight::new(resources, state, (4, 0)),
                PassiveLight::new(resources, state, (1, 5)),
                PassiveLight::new(resources, state, (2, 5)),
                PassiveLight::new(resources, state, (3, 5)),
                PassiveLight::new(resources, state, (4, 5)),
                PassiveLight::new(resources, state, (0, 1)),
                PassiveLight::new(resources, state, (0, 2)),
                PassiveLight::new(resources, state, (0, 3)),
                PassiveLight::new(resources, state, (0, 4)),
                PassiveLight::new(resources, state, (5, 1)),
                PassiveLight::new(resources, state, (5, 2)),
                PassiveLight::new(resources, state, (5, 3)),
                PassiveLight::new(resources, state, (5, 4)),
            ],
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };
        view.drain_queue();
        view
    }

    fn current_scene(&self, state: &AtticState) -> &Scene {
        if state.is_solved() {
            &self.outro_scene
        } else {
            &self.intro_scene
        }
    }

    fn hud_input(&self, state: &AtticState) -> HudInput {
        let scene = self.current_scene(state);
        HudInput {
            name: "A Light in the Attic",
            access: state.access(),
            is_paused: scene.is_paused(),
            active: self.screen_fade.is_transparent() && scene.is_finished(),
            can_undo: !self.undo_stack.is_empty(),
            can_redo: !self.redo_stack.is_empty(),
            can_reset: state.any_toggled(),
        }
    }

    fn undo(&mut self, state: &mut AtticState) {
        if let Some(position) = self.undo_stack.pop() {
            self.redo_stack.push(position);
            state.toggle(position);
        }
    }

    fn redo(&mut self, state: &mut AtticState) {
        if let Some(position) = self.redo_stack.pop() {
            self.undo_stack.push(position);
            state.toggle(position);
        }
    }

    fn reset(&mut self, state: &mut AtticState) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        state.reset();
    }

    fn solve(&mut self, state: &mut AtticState) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        state.solve();
    }

    fn drain_queue(&mut self) {
        for (index, enable) in self.theater.drain_queue() {
            self.toggles[index as usize].set_hilight(enable != 0);
        }
    }
}

impl Element<Game, PuzzleCmd> for View {
    fn draw(&self, game: &Game, canvas: &mut Canvas) {
        let state = &game.a_light_in_the_attic;
        self.theater.draw_background(canvas);
        self.theater.draw_foreground(canvas);
        self.passives.draw(state, canvas);
        self.toggles.draw(state, canvas);
        self.theater.draw_speech_bubbles(canvas);
        self.hud.draw(&self.hud_input(state), canvas);
        self.screen_fade.draw(&(), canvas);
    }

    fn handle_event(&mut self, event: &Event, game: &mut Game)
                    -> Action<PuzzleCmd> {
        let state = &mut game.a_light_in_the_attic;
        let mut action = self.screen_fade.handle_event(event, &mut ());
        if !action.should_stop() {
            let subaction = if state.is_solved() {
                self.outro_scene.handle_event(event, &mut self.theater)
            } else {
                self.intro_scene.handle_event(event, &mut self.theater)
            };
            action.merge(subaction.but_no_value());
            self.drain_queue();
        }
        if !action.should_stop() {
            let mut input = self.hud_input(state);
            let subaction = self.hud.handle_event(event, &mut input);
            action.merge(match subaction.value() {
                Some(&HudCmd::Back) => {
                    self.screen_fade.fade_out_and_return(PuzzleCmd::Back);
                    subaction.but_no_value()
                }
                Some(&HudCmd::Info) => subaction.but_return(PuzzleCmd::Info),
                Some(&HudCmd::Undo) => {
                    self.undo(state);
                    subaction.but_no_value()
                }
                Some(&HudCmd::Redo) => {
                    self.redo(state);
                    subaction.but_no_value()
                }
                Some(&HudCmd::Reset) => {
                    self.reset(state);
                    subaction.but_no_value()
                }
                Some(&HudCmd::Replay) => {
                    self.screen_fade.fade_out_and_return(PuzzleCmd::Replay);
                    subaction.but_no_value()
                }
                Some(&HudCmd::Solve) => {
                    self.solve(state);
                    subaction.but_no_value()
                }
                None => subaction.but_no_value(),
            });
        }
        if !action.should_stop() {
            let subaction = self.toggles.handle_event(event, state);
            if let Some(&position) = subaction.value() {
                state.toggle(position);
                if state.is_solved() {
                    if cfg!(debug_assertions) {
                        println!("Puzzle solved, beginning outro.");
                    }
                    self.outro_scene.begin(&mut self.theater);
                    self.undo_stack.clear();
                } else {
                    self.undo_stack.push(position);
                }
                self.redo_stack.clear();
            }
            action.merge(subaction.but_no_value());
        }
        if !action.should_stop() {
            action.merge(self.passives.handle_event(event, state));
        }
        action
    }
}

impl PuzzleView for View {
    fn info_text(&self, game: &Game) -> &'static str {
        if game.a_light_in_the_attic.is_solved() {
            SOLVED_INFO_TEXT
        } else {
            INFO_BOX_TEXT
        }
    }

    fn replay(&mut self, game: &mut Game) {
        game.a_light_in_the_attic.replay();
        for toggle in self.toggles.iter_mut() {
            toggle.set_hilight(false);
        }
        self.theater.reset();
        self.intro_scene.reset();
        self.outro_scene.reset();
        self.intro_scene.begin(&mut self.theater);
        self.screen_fade.fade_in();
    }
}

// ========================================================================= //

const LIGHTS_TOP: i32 = 56;
const LIGHTS_LEFT: i32 = 312;
const TOGGLE_MAX_LIGHT_RADIUS: i32 = 12;

pub struct ToggleLight {
    frame_off: Sprite,
    frame_on: Sprite,
    label: Sprite,
    position: (i32, i32),
    light_radius: i32,
    hilight: bool,
}

impl ToggleLight {
    fn new(resources: &mut Resources, state: &AtticState,
           position: (i32, i32), label: char)
           -> ToggleLight {
        let sprites = resources.get_sprites("toggle_light");
        ToggleLight {
            frame_off: sprites[0].clone(),
            frame_on: sprites[1].clone(),
            label: resources.get_font("block").glyph(label).sprite().clone(),
            position: position,
            light_radius: if state.is_lit(position) {
                TOGGLE_MAX_LIGHT_RADIUS
            } else {
                0
            },
            hilight: false,
        }
    }

    fn rect(&self) -> Rect {
        let (col, row) = self.position;
        Rect::new(LIGHTS_LEFT + 32 * col, LIGHTS_TOP + 32 * row, 32, 32)
    }

    fn set_hilight(&mut self, hilight: bool) { self.hilight = hilight; }
}

impl Element<AtticState, (i32, i32)> for ToggleLight {
    fn draw(&self, state: &AtticState, canvas: &mut Canvas) {
        let mut canvas = canvas.subcanvas(self.rect());
        draw_light(&mut canvas,
                   self.light_radius,
                   TOGGLE_MAX_LIGHT_RADIUS,
                   self.hilight);
        let center = canvas.rect().center();
        canvas.draw_sprite_centered(&self.label, center);
        let frame = if state.is_toggled(self.position) {
            &self.frame_on
        } else {
            &self.frame_off
        };
        canvas.draw_sprite_centered(frame, center);
    }

    fn handle_event(&mut self, event: &Event, state: &mut AtticState)
                    -> Action<(i32, i32)> {
        match event {
            &Event::ClockTick => {
                tick_radius(state.is_lit(self.position),
                            &mut self.light_radius,
                            TOGGLE_MAX_LIGHT_RADIUS)
            }
            &Event::MouseDown(pt) if self.rect().contains(pt) &&
                                     !state.is_solved() => {
                Action::redraw().and_return(self.position)
            }
            _ => Action::ignore(),
        }
    }
}

// ========================================================================= //

const PASSIVE_MAX_LIGHT_RADIUS: i32 = 11;

pub struct PassiveLight {
    frame: Sprite,
    position: (i32, i32),
    light_radius: i32,
}

impl PassiveLight {
    fn new(resources: &mut Resources, state: &AtticState, position: (i32, i32))
           -> PassiveLight {
        let sprites = resources.get_sprites("toggle_light");
        let (col, row) = position;
        let sprite_index = if col == 5 {
            2
        } else if row == 0 {
            3
        } else if col == 0 {
            4
        } else {
            5
        };
        PassiveLight {
            frame: sprites[sprite_index].clone(),
            position: position,
            light_radius: if state.is_lit(position) {
                PASSIVE_MAX_LIGHT_RADIUS
            } else {
                0
            },
        }
    }

    fn rect(&self) -> Rect {
        let (col, row) = self.position;
        Rect::new(LIGHTS_LEFT + 32 * col, LIGHTS_TOP + 32 * row, 32, 32)
    }
}

impl Element<AtticState, PuzzleCmd> for PassiveLight {
    fn draw(&self, _: &AtticState, canvas: &mut Canvas) {
        let mut canvas = canvas.subcanvas(self.rect());
        draw_light(&mut canvas,
                   self.light_radius,
                   PASSIVE_MAX_LIGHT_RADIUS,
                   false);
        let center = canvas.rect().center();
        canvas.draw_sprite_centered(&self.frame, center);
    }

    fn handle_event(&mut self, event: &Event, state: &mut AtticState)
                    -> Action<PuzzleCmd> {
        match event {
            &Event::ClockTick => {
                tick_radius(state.is_lit(self.position),
                            &mut self.light_radius,
                            PASSIVE_MAX_LIGHT_RADIUS)
            }
            _ => Action::ignore(),
        }
    }
}

// ========================================================================= //

fn light_rect(center: Point, radius: i32) -> Rect {
    Rect::new(center.x() - radius,
              center.y() - radius,
              2 * radius as u32,
              2 * radius as u32)
}

fn draw_light(canvas: &mut Canvas, radius: i32, max: i32, hilight: bool) {
    let center = canvas.rect().center();
    if hilight {
        canvas.fill_rect((255, 64, 255), light_rect(center, max));
    } else {
        if radius < max {
            canvas.fill_rect((0, 0, 32), light_rect(center, max));
        }
        if radius > 0 {
            canvas.fill_rect((255, 255, 192), light_rect(center, radius));
        }
    }
}

fn tick_radius<A>(lit: bool, radius: &mut i32, max: i32) -> Action<A> {
    if lit {
        if *radius < max {
            *radius = cmp::min(max, *radius + 3);
            return Action::redraw();
        }
    } else {
        if *radius > 0 {
            *radius = cmp::max(0, *radius - 3);
            return Action::redraw();
        }
    }
    Action::ignore()
}

// ========================================================================= //

const INFO_BOX_TEXT: &'static str = "\
Your goal is to turn all thirty-two lights on.

$M{Tapp}{Click}ing on one of the lights labelled with a letter
will toggle some of the nearby lights.

The letter labels give a hint as to which other lights
will be toggled by $M{tapp}{click}ing on that light.";

// ========================================================================= //
