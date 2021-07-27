// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

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
}
