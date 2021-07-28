// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

pub const ATNID_SAMPLE_LOAD_ID : u32 = 190001;
pub const ATNID_HELP_BUTTON    : u32 = 190002;

pub use crate::menu::MenuState;
use hexodsp::{NodeId, CellDir};
pub use hexodsp::dsp::UICategory;

use hexotk::widgets::{
    TextSourceRef
};

use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum ItemType {
    Back,
    Category(UICategory),
    NodeId(NodeId),
    Direction(CellDir),
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub typ:    ItemType,
    pub label:  String,
    pub help:   String,
}

#[derive(Debug, Clone)]
pub struct State {
    pub show_help:          bool,

    pub menu_help_text:     Rc<TextSourceRef>,
    pub menu_items:         Vec<MenuItem>,
    pub menu_pos:           (f64, f64),

    pub menu_state:         MenuState,
}

impl State {
    pub fn new() -> Self {
        Self {
            show_help:       false,
            menu_help_text:  Rc::new(TextSourceRef::new(30)),
            menu_items:      vec![],
            menu_pos:        (0.0, 0.0),
            menu_state:      MenuState::None,
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}