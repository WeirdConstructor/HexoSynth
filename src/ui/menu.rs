use crate::dsp::{UICategory, NodeInfo, NodeId};
use crate::matrix::{Cell, CellDir};
use std::rc::Rc;
use std::cell::RefCell;

pub trait MenuActionHandler {
    fn update_help_text(&mut self, txt: &str);
}

pub trait MenuControl {
    fn set_hover_index(&mut self, index: usize);

    fn update(&mut self);
    fn select(&mut self, idx: usize);
    fn label(&self, idx: usize) -> Option<&str>;

    fn is_open(&self) -> bool;
    fn open_select_node_category(&mut self);
    fn open_select_cell_dir(&mut self, cell: Cell, node_info: NodeInfo);
    fn close(&mut self);
}

enum MenuEvent {
}

#[derive(Debug, Clone)]
enum MenuState {
    None,
    NodeCategory {
        lbls: Vec<NodeId>
    },
    CellDir {
        cell:       Cell,
        node_info:  Rc<NodeInfo>,
        dirs:       Vec<CellDir>,
    },
    InputPort {
        cell:       Cell,
        cell_dir:   CellDir,
        node_info:  Rc<NodeInfo>,
        offset:     usize,
    },
    OutputPort {
        cell:       Cell,
        cell_dir:   CellDir,
        node_info:  Rc<NodeInfo>,
        offset:     usize,
    },
}

pub struct Menu {
    cur:        MenuState,
    lbl_fun:    MenuLblFun,
    act_fun:    MenuActionFun,

    prev:       Vec<MenuState>,
    handler:    Box<dyn MenuActionHandler>,
    hover_idx:  usize,

    just_closed: Rc<RefCell<bool>>,
}

impl Menu {
    pub fn new(handler: Box<dyn MenuActionHandler>) -> Self {
        Self {
            handler,
            cur:        MenuState::None,
            prev:       vec![],
            lbl_fun:    Box::new(|_idx: usize| { None }),
            act_fun:    Box::new(|_idx: usize, _ms: &MenuState, _hdl: &mut Box<MenuActionHandler>| {}),
            hover_idx:  0,
            just_closed: Rc::new(RefCell::new(false)),
        }
    }
}

type MenuLblFun    = Box<dyn Fn(usize) -> Option<&'static str>>;
type MenuActionFun = Box<dyn FnMut(usize, &MenuState, &mut Box<dyn MenuActionHandler>)>;

impl Menu {
    fn load_fun(&mut self) {
        match self.cur {
            MenuState::None  => {
                self.lbl_fun = Box::new(|idx| { None });
                self.act_fun = Box::new(|idx, state, hdl| { });
            }
            MenuState::NodeCategory { .. } => {
                self.lbl_fun = Box::new(|idx| {
                    match idx {
                        0 => Some("<Exit"),
                        1 => Some("Osc"),
                        2 => Some("X->Y"),
                        3 => Some("Time"),
                        4 => Some("N->M"),
                        5 => Some("I/O"),
                        _ => None,
                    }
                });
                let jc = self.just_closed.clone();
                self.act_fun = Box::new(move |idx, state, hdl| {
                    match idx {
                        0 => {
                            let mut jc = jc.borrow_mut();
                            *jc = true;
                        },
                        _ => {},
                    }
                });
            },
            MenuState::CellDir { .. } => { },
            MenuState::InputPort { .. } => { },
            MenuState::OutputPort { .. } => { },
        }
    }
}

impl MenuControl for Menu {
    fn set_hover_index(&mut self, idx: usize) {
        self.hover_idx = idx;
    }

    fn update(&mut self) {
        // TODO: Update Setting the Text!
        self.handler.update_help_text("HELP");
    }

    fn select(&mut self, idx: usize) {
        (*self.act_fun)(idx, &self.cur, &mut self.handler);
        if *self.just_closed.borrow() {
            self.close();
        }
    }

    fn label(&self, idx: usize) -> Option<&str> {
        (*self.lbl_fun)(idx)
    }

    fn is_open(&self) -> bool {
        if let MenuState::None = self.cur {
            false
        } else {
            true
        }
    }

    fn open_select_node_category(&mut self) {
        self.prev.clear();
        self.cur = MenuState::NodeCategory {
            lbls: vec![],
        };
        self.load_fun();
    }

    fn open_select_cell_dir(&mut self, cell: Cell, node_info: NodeInfo) {
        self.prev.clear();
        self.cur = MenuState::CellDir {
            cell,
            node_info: Rc::new(node_info),
            dirs: vec![],
        };
        self.load_fun();
    }

    fn close(&mut self) {
        self.cur = MenuState::None;
        self.load_fun();
        *self.just_closed.borrow_mut() = false;
        self.prev.clear();
    }
}
