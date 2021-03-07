use crate::dsp::{UICategory, NodeInfo, NodeId};
use crate::matrix::{Cell, CellDir};
use std::rc::Rc;

pub trait MenuActionHandler {
    fn update_help_text(&mut self, txt: &str);
}

pub trait MenuControl {
    fn set_hover_pos(&mut self, x: usize, y: usize);
    fn hover_pos(&self) -> (usize, usize);

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
    prev:       Vec<MenuState>,
    handler:    Box<dyn MenuActionHandler>,
    hover_pos:  (usize, usize),
}

impl Menu {
    pub fn new(handler: Box<dyn MenuActionHandler>) -> Self {
        Self {
            handler,
            cur:        MenuState::None,
            prev:       vec![],
            hover_pos:  (0, 0),
        }
    }
}

impl MenuControl for Menu {
    fn hover_pos(&self) -> (usize, usize) { self.hover_pos }
    fn set_hover_pos(&mut self, x: usize, y: usize) {
        self.hover_pos = (x, y);
    }

    fn update(&mut self) {
        // TODO: Update Setting the Text!
        self.handler.update_help_text("HELP");
    }

    fn select(&mut self, idx: usize) {
    }

    fn label(&self, idx: usize) -> Option<&str> {
        None
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
    }

    fn open_select_cell_dir(&mut self, cell: Cell, node_info: NodeInfo) {
        self.prev.clear();
        self.cur = MenuState::CellDir {
            cell,
            node_info: Rc::new(node_info),
            dirs: vec![],
        };
    }

    fn close(&mut self) {
        self.cur = MenuState::None;
        self.prev.clear();
    }
}
