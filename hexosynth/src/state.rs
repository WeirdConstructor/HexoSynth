// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

pub const ATNID_SAMPLE_LOAD_ID : u32 = 190001;
pub const ATNID_HELP_BUTTON    : u32 = 190002;
pub const ATNID_SAVE_BUTTON    : u32 = 190003;
pub const ATNID_CLR_SELECT     : u32 = 190004;

use crate::dyn_widgets::DynamicWidgets;
use hexodsp::{NodeId, CellDir, Cell, NodeInfo, Matrix, SAtom};
use hexotk::AtomId;
pub use crate::menu::MenuState;
pub use hexodsp::dsp::UICategory;

use hexotk::widgets::{
    TextSourceRef
};

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum RandSpecifier {
    One,
    Six,
}

#[derive(Debug, Clone)]
pub enum ItemType {
    Back,
    Delete,
    ClearPorts,
    Help(NodeId),
    Category(UICategory),
    NodeId(NodeId),
    Direction(CellDir),
    OutputIdx(usize),
    InputIdx(usize),
    SubMenu { menu_state: Box<MenuState>, title: String },
    RandomNode(RandSpecifier),
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub typ:    ItemType,
    pub label:  String,
    pub help:   String,
}

#[derive(Debug, Clone)]
pub struct State {
    pub show_help:           bool,

    pub focus_cell:          Cell,
    pub focus_node_info:     NodeInfo,
    pub focus_uniq_node_idx: u32,

    pub sample_load_id:      AtomId,
    pub sample_dir_from:     Option<AtomId>,

    pub current_tracker_idx: usize,

    pub menu_title:          Rc<RefCell<String>>,
    pub menu_help_text:      Rc<TextSourceRef>,
    pub help_text_src:       Rc<TextSourceRef>,
    pub menu_items:          Vec<MenuItem>,
    pub menu_pos:            (f64, f64),
    pub next_menu_pos:       Option<(f64, f64)>,

    pub menu_history:        Vec<(MenuState, String)>,
    pub menu_state:          MenuState,

    pub widgets:             DynamicWidgets,

    pub node_colors:         HashMap<NodeId, u8>,
}

impl State {
    pub fn new() -> Self {
        Self {
            show_help:       false,
            menu_help_text:  Rc::new(TextSourceRef::new(32)),
            help_text_src:
                Rc::new(TextSourceRef::new(
                    crate::ui::UI_MAIN_HELP_TEXT_WIDTH)),
            menu_items:             vec![],
            menu_pos:               (0.0, 0.0),
            next_menu_pos:          None,
            menu_history:           vec![],
            menu_state:             MenuState::None,
            menu_title:             Rc::new(RefCell::new("?".to_string())),
            focus_cell:             Cell::empty(NodeId::Nop),
            focus_node_info:        NodeInfo::from_node_id(NodeId::Nop),
            focus_uniq_node_idx:    9999999,
            sample_load_id:         AtomId::from(99999),
            sample_dir_from:        None,
            current_tracker_idx:    0,
            widgets:                DynamicWidgets::new(),
            node_colors:            HashMap::new(),
        }
    }

    pub fn sync_to_matrix(&self, m: &mut Matrix) {
        let mut entries = vec![];

        for (k, v) in self.node_colors.iter() {
            entries.push(format!("{},{},{}",
                k.name(),
                k.instance(),
                v));
        }

        m.set_prop("node_colors", SAtom::str(&entries.join(";")));
    }

    pub fn sync_from_matrix(&mut self, m: &mut Matrix) {
        println!("SYNC FROM");
        if let Some(SAtom::Str(s)) = m.get_prop("node_colors") {
            println!("SYNC FROM {}", s);

            for entry in s.split(";") {
                let entry : Vec<&str> = entry.split(",").collect();

                let node_id = NodeId::from_str(entry[0]);
                let inst = entry[1].parse::<usize>().unwrap_or(0);
                let node_id = node_id.to_instance(inst);
                let color = entry[2].parse::<u8>().unwrap_or(0);

                self.node_colors.insert(node_id, color);
            }
        }
    }

    pub fn color_for_node(&self, node_id: NodeId) -> u8 {
        if let Some(clr) = self.node_colors.get(&node_id) {
            *clr
        } else {
            node_id.ui_category().default_color_idx()
        }
    }

    pub fn is_cell_focussed(&self, x: usize, y: usize) -> bool {
        let cell = self.focus_cell;

        if cell.node_id() == NodeId::Nop {
            return false;
        }

        let (cx, cy) = cell.pos();
        cx == x && cy == y
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn show_help(&mut self) {
        self.show_help = true;
    }
}
