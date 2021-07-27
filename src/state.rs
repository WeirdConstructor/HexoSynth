// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::uimsg_queue::{Msg};
use crate::UIParams;

use hexodsp::Matrix;

use keyboard_types::Key;

pub const ATNID_SAMPLE_LOAD_ID : u32 = 190001;
pub const ATNID_HELP_BUTTON    : u32 = 190002;

pub struct State {
    pub show_help: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            show_help: false,
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn apply(
        &mut self, ui_params: &mut UIParams,
        matrix: &mut Matrix, msg: &Msg
    ) {
        match msg {
            Msg::CellDragged { btn, pos_a, pos_b } => {
                println!("DRAG CELL! {:?} {:?}", btn, msg);
            },
            Msg::Key { key } => {
                match key {
                    Key::F1     => self.toggle_help(),
                    Key::Escape => self.toggle_help(),
                    _ => {
                        println!("UNHANDLED KEY: {:?}", key);
                    }
                }
            },
            Msg::UIBtn { id } => {
                match *id {
                    ATNID_HELP_BUTTON => self.toggle_help(),
                    _ => {}
                }
            }
        }
    }
}

impl Default for State { fn default() -> Self { Self::new() } }
