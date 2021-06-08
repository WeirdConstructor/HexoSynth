// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use crate::dsp::{UICategory, NodeInfo, NodeId};
use crate::matrix::{Cell, CellDir};
use std::rc::Rc;
use std::cell::RefCell;

pub trait MenuActionHandler {
    fn update_help_text(&mut self, txt: &str);
    fn assign_cell_port(
        &mut self, cell: Cell, cell_dir: CellDir, idx: Option<usize>);
    fn assign_cell_new_node(
        &mut self, cell: Cell, node_id: NodeId);
}

pub trait MenuControl {
    fn set_hover_index(&mut self, index: usize);

    fn update(&mut self);
    fn select(&mut self, idx: usize);
    fn label(&self, idx: usize) -> Option<&str>;

    fn is_open(&self) -> bool;
    fn open_select_node_category(&mut self, cell: Cell);
    fn open_select_cell_dir(&mut self, cell: Cell, node_info: NodeInfo);
    fn close(&mut self);
}

#[derive(Debug, Clone)]
enum MenuState {
    None,
    NodeCategory {
        cell:   Cell,
    },
    SubCategory {
        cell:   Cell,
        cat:    UICategory,
        offset: usize,
        count:  usize,
    },
    CellDir {
        cell:       Cell,
        node_info:  Rc<NodeInfo>,
        dirs:       Vec<CellDir>,
    },
    AssignPort {
        cell:       Cell,
        cell_dir:   CellDir,
        node_info:  Rc<NodeInfo>,
        offset:     usize,
    },
}

enum PostAction {
    None,
    Close,
    Back,
    NextState(MenuState),
}

impl PostAction {
    fn close() -> Self {
        PostAction::Close
    }

    fn back() -> Self {
        PostAction::Back
    }


    fn next_state(ms: MenuState) -> Self {
        PostAction::NextState(ms)
    }
}

pub struct Menu {
    cur:        MenuState,
    lbl_fun:    MenuLblFun,
    act_fun:    MenuActionFun,

    prev:       Vec<MenuState>,
    handler:    Box<dyn MenuActionHandler>,
    hover_idx:  usize,

    post_action:  Rc<RefCell<PostAction>>,
}

impl Menu {
    pub fn new(handler: Box<dyn MenuActionHandler>) -> Self {
        Self {
            handler,
            cur:        MenuState::None,
            prev:       vec![],
            lbl_fun:    Box::new(|_idx: usize, _help: bool, _ms: &MenuState| { None }),
            act_fun:    Box::new(|_idx: usize, _ms: &MenuState, _hdl: &mut Box<dyn MenuActionHandler>| {}),
            hover_idx:  0,
            post_action: Rc::new(RefCell::new(PostAction::None)),
        }
    }
}

type MenuLblFun    = Box<dyn Fn(usize, bool, &MenuState) -> Option<&'static str>>;
type MenuActionFun = Box<dyn FnMut(usize, &MenuState, &mut Box<dyn MenuActionHandler>)>;

impl Menu {
    fn activate_init_state(&mut self, state: MenuState) {
        self.prev.clear();
        self.cur = state;
        self.load_fun();
    }

    fn activate_prev_state(&mut self) {
        if let Some(state) = self.prev.pop() {
            self.cur = state;
            self.load_fun();
        } else {
            self.close();
        }
    }

    fn check_and_activate_next_state(&mut self, ns: MenuState) {
        let prev = std::mem::replace(&mut self.cur, ns);
        self.prev.push(prev);
        self.load_fun();
    }

    fn load_fun(&mut self) {
        let pa = self.post_action.clone();

        match self.cur {
            MenuState::None  => {
                self.lbl_fun = Box::new(|_idx, _help, _state| { None });
                self.act_fun = Box::new(|_idx, _state, _hdl| ());
            }
            MenuState::NodeCategory { cell } => {
                self.lbl_fun = Box::new(|idx, help, _state| {
                    if help {
                        match idx {
                            0 => Some("\nExit Menu"),
                            1 => Some("Osc\nAudio oscillators."),
                            2 => Some("Signal\nSignal shapers:\n- Filters\n- Waveshapers\n- Delays"),
                            3 => Some("CV\nControl voltage shapers:\n- CV converters\n- Quantizers\n- Sample & Hold\n- Slew"),
                            4 => Some("Mod\nModulation sources:\n- Envelopes\n- LFOs\n- Sequencers\n- Utils"),
                            5 => Some("N->M\nSignal merge and splitters:\n- Mixers\n- Logic\n- Math\n- Multiplexers"),
                            6 => Some("I/O\nExternal connections:\n- Audio I/O\n- MIDI"),
                            _ => None,
                        }
                    } else {
                        match idx {
                            0 => Some("<Exit"),
                            1 => Some("Osc"),
                            2 => Some("Signal"),
                            3 => Some("CV"),
                            4 => Some("Mod"),
                            5 => Some("N->M"),
                            6 => Some("I/O"),
                            _ => None,
                        }
                    }
                });
                self.act_fun = Box::new(move |idx, _state, _hdl| {
                    match idx {
                        0 => { *pa.borrow_mut() = PostAction::close(); },
                        1 | 2 | 3 | 4 | 5 | 6 => {
                            let cat =
                                match idx {
                                    1 => UICategory::Osc,
                                    2 => UICategory::Signal,
                                    3 => UICategory::CV,
                                    4 => UICategory::Mod,
                                    5 => UICategory::NtoM,
                                    6 => UICategory::IOUtil,
                                    _ => UICategory::None,
                                };

                            let mut count = 0;
                            cat.get_node_ids(0, |_| count += 1);

                            *pa.borrow_mut() =
                                PostAction::next_state(
                                    MenuState::SubCategory {
                                        cell,
                                        cat,
                                        count,
                                        offset: 0,
                                    });
                        },
                        _ => (),
                    }
                });
            },
            MenuState::SubCategory { cell, cat, offset, count } => {
                self.lbl_fun = Box::new(move |idx, help, _state| {
                    match idx {
                        0 => Some("<Back"),
                        _ => {
                            let cur_idx = (idx - 1) + offset;

                            let next =
                                if idx == 6 {
                                    let next_idx = cur_idx + 1;
                                    next_idx < count
                                } else {
                                    false
                                };

                            if next {
                                Some("Next>")
                            } else {
                                let mut s : Option<&'static str> = None;

                                let mut i = 0;
                                cat.get_node_ids(offset, |node_id| {
                                    if i == (idx - 1) {
                                        if help {
                                            s = Some(NodeInfo::from_node_id(node_id).desc());
                                        } else {
                                            s = Some(node_id.name());
                                        }
                                    }
                                    i += 1;
                                });

                                s
                            }
                        }
                    }
                });
                self.act_fun = Box::new(move |idx, _state, hdl| {
                    match idx {
                        0 => { *pa.borrow_mut() = PostAction::back(); },
                        _ => {
                            let cur_idx = (idx - 1) + offset;

                            let next =
                                if idx == 6 {
                                    let next_idx = cur_idx + 1;
                                    next_idx < count
                                } else {
                                    false
                                };

                            if next {
                                *pa.borrow_mut() =
                                    PostAction::next_state(
                                        MenuState::SubCategory {
                                            cell,
                                            cat,
                                            offset: cur_idx,
                                            count,
                                        });
                            } else {
                                let mut i = 0;
                                let mut out_node_id = None;
                                cat.get_node_ids(offset, |node_id| {
                                    if i == (idx - 1) {
                                        out_node_id = Some(node_id);
                                    }
                                    i += 1;
                                });

                                if let Some(node_id) = out_node_id {
                                    hdl.assign_cell_new_node(
                                        cell.clone(), node_id);
                                }
                                *pa.borrow_mut() = PostAction::close();
                            }
                        },
                    }
                });
            },
            MenuState::CellDir { .. } => {
                self.lbl_fun = Box::new(|idx, _help, _state| {
                    match idx {
                        0 => Some("<Exit"),
                        1 => Some("In 1"),
                        2 => Some("Out 1"),
                        3 => Some("Out 2"),
                        4 => Some("Out 3"),
                        5 => Some("In 3"),
                        6 => Some("In 2"),
                        _ => None,
                    }
                });
                self.act_fun = Box::new(move |idx, state, _hdl| {

                    let mut cell_dir = None;

                    println!("CLICK CD {},{:?}", idx, state);

                    match idx {
                        0 => { *pa.borrow_mut() = PostAction::close(); },
                        1 => { cell_dir = Some(CellDir::T); },
                        2 => { cell_dir = Some(CellDir::TR); },
                        3 => { cell_dir = Some(CellDir::BR); },
                        4 => { cell_dir = Some(CellDir::B); },
                        5 => { cell_dir = Some(CellDir::BL); },
                        6 => { cell_dir = Some(CellDir::TL); },
                        _ => (),
                    }

                    if let Some(cell_dir) = cell_dir {
                        if let MenuState::CellDir { cell, node_info, .. } = state {
                            let ms =
                                MenuState::AssignPort {
                                    cell:      cell.clone(),
                                    node_info: node_info.clone(),
                                    offset:    0,
                                    cell_dir,
                                };

                            *pa.borrow_mut() = PostAction::next_state(ms);
                        }
                    }
                });
            },
            MenuState::AssignPort { .. } => {
                self.lbl_fun = Box::new(|idx, help, state| {
                    match idx {
                        0 => Some("<Back"),
                        _ => {
                            match state {
                                MenuState::AssignPort {
                                    cell_dir, node_info, offset, ..
                                } => {
                                    let cur_idx = (idx - 1) + offset;

                                    let max =
                                        if cell_dir.is_output() {
                                            node_info.out_count()
                                        } else {
                                            node_info.in_count()
                                        };

                                    let next =
                                        if idx == 6 {
                                            let next_idx = cur_idx + 1;
                                            next_idx < max
                                        } else {
                                            false
                                        };

                                    if next {
                                        Some("Next>")
                                    } else {
                                        if help {
                                            if cell_dir.is_output() {
                                                node_info.out_help(cur_idx)
                                            } else {
                                                node_info.in_help(cur_idx)
                                            }
                                        } else {
                                            if cell_dir.is_output() {
                                                node_info.out_name(cur_idx)
                                            } else {
                                                node_info.in_name(cur_idx)
                                            }
                                        }
                                    }
                                },
                                _ => None,
                            }
                        }
                    }
                });
                self.act_fun = Box::new(move |idx, state, hdl| {
                    match idx {
                        0 => { *pa.borrow_mut() = PostAction::back(); },
                        _ => {
                            match state {
                                MenuState::AssignPort {
                                    cell, cell_dir, offset, node_info, ..
                                } => {
                                    let cur_idx = (idx - 1) + offset;

                                    if idx == 6 {
                                        let max =
                                            if cell_dir.is_output() {
                                                node_info.out_count()
                                            } else {
                                                node_info.in_count()
                                            };

                                        let next_idx = cur_idx + 1;

                                        if next_idx < max {
                                            *pa.borrow_mut() =
                                                PostAction::next_state(
                                                    MenuState::AssignPort {
                                                        cell:      cell.clone(),
                                                        node_info: node_info.clone(),
                                                        cell_dir:  cell_dir.clone(),
                                                        offset:    cur_idx,
                                                    });
                                        } else {
                                            hdl.assign_cell_port(
                                                cell.clone(),
                                                cell_dir.clone(),
                                                Some(cur_idx));
                                            *pa.borrow_mut() = PostAction::close();
                                        }

                                    } else {
                                        hdl.assign_cell_port(
                                            cell.clone(),
                                            cell_dir.clone(),
                                            Some(cur_idx));
                                        *pa.borrow_mut() = PostAction::close();
                                    }
                                },
                                _ => ()
                            }
                        },
                    }
                });
            }
        }
    }
}

impl MenuControl for Menu {
    fn set_hover_index(&mut self, idx: usize) {
        self.hover_idx = idx;
    }

    fn update(&mut self) {
        if let Some(txt) = (*self.lbl_fun)(self.hover_idx, true, &self.cur) {
            self.handler.update_help_text(txt);
        } else {
            self.handler.update_help_text("");
        }
    }

    fn select(&mut self, idx: usize) {
        (*self.act_fun)(idx, &self.cur, &mut self.handler);

        let action =
            std::mem::replace(
                &mut *self.post_action.borrow_mut(),
                PostAction::None);

        match action {
            PostAction::None => {},
            PostAction::Close => { self.close(); }
            PostAction::Back => { self.activate_prev_state(); }
            PostAction::NextState(state) => {
                self.check_and_activate_next_state(state);
            },
        }
    }

    fn label(&self, idx: usize) -> Option<&str> {
        (*self.lbl_fun)(idx, false, &self.cur)
    }

    fn is_open(&self) -> bool {
        if let MenuState::None = self.cur {
            false
        } else {
            true
        }
    }

    fn open_select_node_category(&mut self, cell: Cell) {
        self.activate_init_state(
            MenuState::NodeCategory { cell });
    }

    fn open_select_cell_dir(&mut self, cell: Cell, node_info: NodeInfo) {
        self.activate_init_state(
            MenuState::CellDir {
                cell,
                node_info: Rc::new(node_info),
                dirs: vec![],
            });
    }

    fn close(&mut self) {
        self.cur = MenuState::None;
        self.load_fun();
        *self.post_action.borrow_mut() = PostAction::None;
        self.prev.clear();
    }
}
