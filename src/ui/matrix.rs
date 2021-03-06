use hexotk::widgets::hexgrid::HexGridModel;
use hexotk::{MButton, ActiveZone, UIPos, ParamID};
use hexotk::{Rect, WidgetUI, Painter, WidgetData, WidgetType, UIEvent};
use hexotk::constants::*;
use hexotk::widgets::{HexGrid, HexGridData, HexCell, HexEdge, HexDir};

use std::rc::Rc;
use std::cell::RefCell;

use crate::matrix::*;
use std::sync::Arc;
use std::sync::Mutex;

use crate::dsp::{UICategory, NodeInfo, NodeId};

struct MenuState {
    matrix:    Arc<Mutex<Matrix>>,
    cell:      Option<Cell>,
    cell_dir:  Option<CellDir>,
    node_info: Option<NodeInfo>,
    list:      Vec<MenuItem>,
    list_offs: usize,
    self_ref:  Option<std::rc::Weak<RefCell<MenuState>>>,
}

impl MenuState {
    fn new(matrix: Arc<Mutex<Matrix>>) -> Self {
        Self {
            matrix,
            cell:       None,
            node_info:  None,
            cell_dir:   None,
            list:       vec![],
            list_offs:  0,
            self_ref:   None,
        }
    }

    fn init_self_ref(&mut self, self_ref: MenuStateRef) {
        self.self_ref = Some(Rc::downgrade(&self_ref));
    }

    fn clear(&mut self) {
        self.cell      = None;
        self.cell_dir  = None;
        self.node_info = None;
        self.list_offs = 0;
        self.list.clear();
    }

    fn load_items_if_any(&self, out_items: &mut Vec<MenuItem>) {
        for (i, item) in self.list.iter().skip(self.list_offs).enumerate() {
            if i == 0 {
                out_items.clear();
                out_items.push(MenuItem::Back);
            }

            out_items.push(item.clone());
            if out_items.len() >= 7 {
                break;
            }
        }
    }

    fn select_cell_io(&mut self, dir: CellDir) {
        self.cell_dir = Some(dir);
        self.list.clear();
        self.list_offs = 0;

        if let Some(state) = self.self_ref.as_ref().unwrap().upgrade() {
            if let Some(node_info) = &self.node_info {
                if dir.is_input() {
                    for i in 0..node_info.in_count() {
                        self.list.push(MenuItem::NodeInput {
                            state: state.clone(),
                            inp: i
                        });
                    }
                } else {
                    for i in 0..node_info.out_count() {
                        self.list.push(MenuItem::NodeOutput {
                            state: state.clone(),
                            out: i
                        });
                    }
                }
            }
        }

    }

    fn set_matrix_cell(&mut self, cell: Cell, node_info: NodeInfo) {
        self.cell       = Some(cell);
        self.node_info  = Some(node_info);
    }

    fn action_set_cell_io(&self, idx: usize) {
        if let Some(dir) = self.cell_dir {
            if let Some(cell) = self.cell {
                let mut m = self.matrix.lock().unwrap();

                let mut cell = cell.clone();
                cell.set_io_dir(dir, idx);
                let pos = cell.pos();

                m.place(pos.0, pos.1, cell);
                m.sync();
            }
        }
    }
}

type MenuStateRef = Rc<RefCell<MenuState>>;

#[derive(Clone)]
enum MenuItem {
    Next,
    Back,
    Exit,
    CellDir     { state: MenuStateRef, dir: CellDir },
    NodeInput   { state: MenuStateRef, inp: usize },
    NodeOutput  { state: MenuStateRef, out: usize },
    Category    { state: MenuStateRef, lbl: &'static str, cat: UICategory },
}

impl MenuItem {
    pub fn as_str<'a>(&'a self) -> &'a str {
        match self {
            MenuItem::Category { lbl, .. } => lbl,
            MenuItem::Next                 => "Next>",
            MenuItem::Back                 => "<Back",
            MenuItem::Exit                 => "<Exit",
            MenuItem::CellDir { dir, .. } => {
                match dir {
                    CellDir::TR => "Out 1",
                    CellDir::BR => "Out 2",
                    CellDir::B  => "Out 3",
                    CellDir::BL => "In 3",
                    CellDir::TL => "In 2",
                    CellDir::T  => "In 1",
                    CellDir::C  => "Node",
                }
            },
            MenuItem::NodeInput { inp, state } => {
                if let Some(ni) = &state.borrow().node_info {
                    ni.in_name(*inp).unwrap_or("inUK")
                } else {
                    ""
                }
            },
            MenuItem::NodeOutput { out, state } => {
                if let Some(ni) = &state.borrow().node_info {
                    ni.out_name(*out).unwrap_or("outUK")
                } else {
                    ""
                }
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum MenuMode {
    None,
    CategorySelect,
    IOSelect,
}

pub struct MatrixUIMenu {
    matrix:     Arc<Mutex<Matrix>>,
    items:      RefCell<Vec<MenuItem>>,
    mode:       RefCell<MenuMode>,
    state:      MenuStateRef,
}

impl MatrixUIMenu {
    pub fn new(matrix: Arc<Mutex<Matrix>>) -> Self {
        let state  = Rc::new(RefCell::new(MenuState::new(matrix.clone())));
        let state2 = state.clone();
        state.borrow_mut().init_self_ref(state2);

        Self {
            matrix,
            items:  RefCell::new(vec![]),
            mode:   RefCell::new(MenuMode::None),
            state,
        }
    }

    pub fn exit_menu(&self) {
        *self.mode.borrow_mut() = MenuMode::None;
        self.state.borrow_mut().clear();
    }

    pub fn with_item_at<F: FnMut(Option<&MenuItem>)>(
        &self, x: usize, y: usize, f: &mut F)
    {
        let items : std::cell::Ref<'_, Vec<MenuItem>> = self.items.borrow();
        match (x, y) {
            // Center
            (1, 1) => f(items.get(0)),
            // TR
            (2, 1) => f(items.get(2)),
            // BR
            (2, 2) => f(items.get(3)),
            // B
            (1, 2) => f(items.get(4)),
            // BL
            (0, 2) => f(items.get(5)),
            // TL
            (0, 1) => f(items.get(6)),
            // T
            (1, 0) => f(items.get(1)),
            _      => (),
        }
    }

    pub fn set_edge_assign_mode(&self, cell: Cell, node_info: NodeInfo) {
        self.state.borrow_mut().set_matrix_cell(cell, node_info);

        (*self.mode.borrow_mut()) = MenuMode::IOSelect;
        let mut items = self.items.borrow_mut();
        items.clear();

        let state = self.state.clone();
        items.push(MenuItem::Exit);
        items.push(MenuItem::CellDir { state: state.clone(), dir: CellDir::T  });
        items.push(MenuItem::CellDir { state: state.clone(), dir: CellDir::TR });
        items.push(MenuItem::CellDir { state: state.clone(), dir: CellDir::BR });
        items.push(MenuItem::CellDir { state: state.clone(), dir: CellDir::B  });
        items.push(MenuItem::CellDir { state: state.clone(), dir: CellDir::BL });
        items.push(MenuItem::CellDir { state,                dir: CellDir::TL });
    }

    pub fn set_category_mode(&self) {
        (*self.mode.borrow_mut()) = MenuMode::CategorySelect;
        let mut items = self.items.borrow_mut();
        items.clear();

        let state = self.state.clone();
        items.push(MenuItem::Exit);
        items.push(MenuItem::Category {
            state: state.clone(),
            lbl: "Osc",
            cat: UICategory::Oscillators
        });
        items.push(MenuItem::Category {
            state: state.clone(),
            lbl: "X->Y",
            cat: UICategory::XtoY,
        });
        items.push(MenuItem::Category {
            state: state.clone(),
            lbl: "Time",
            cat: UICategory::Time,
        });
        items.push(MenuItem::Category {
            state: state.clone(),
            lbl: "N->M",
            cat: UICategory::NtoM,
        });
        items.push(MenuItem::Category {
            state,
            lbl: "I/O",
            cat: UICategory::IOUtil,
        });
    }
}

/* Menu Modes:


- Empty Cell
  - {RMB} Paste
  - New Instance (first categories in edges)
    - <Category> (first sub categories in edges)
      - <Sub Category> (first nodes in edges)
        - <Nodes>
          (Implicit Show UI)
          - <In / Out Assign: 3 In, 3 Out, Ok, Cancel>
  - Existing Instance
    - <List of existing instances>
      (Implicit Show UI)
      - <In / Out Assign: 3 In, 3 Out, Ok, Cancel>
  - Paste (Instance ID in edge label?)
    - <In / Out Assign: 3 In, 3 Out, Ok, Cancel>

- Filled Cell
  - {RMB} Edge Config
  - {MMB} Show UI
  - {LMB} (implicit Show UI)
      - Copy
      - Paste
      - Remove
*/

impl HexGridModel for MatrixUIMenu {
    fn width(&self)  -> usize { 3 }
    fn height(&self) -> usize { 3 }

    fn cell_click(&self, x: usize, y: usize, btn: MButton) {
        self.with_item_at(x, y, &mut |item| {
            println!("MENU CLICK CELL: {},{}: {:?}", x, y, btn);
            if let Some(item) = item {
                match item {
                    MenuItem::CellDir { dir, state }  => {
                        state.borrow_mut().select_cell_io(*dir);
                    },
                    MenuItem::NodeInput { inp, state } => {
                        state.borrow().action_set_cell_io(*inp);
                        self.exit_menu();
                    },
                    MenuItem::NodeOutput { out, state } => {
                        state.borrow().action_set_cell_io(*out);
                        self.exit_menu();
                    },
                    MenuItem::Back => {
                        self.exit_menu();
                    },
                    MenuItem::Exit => {
                        self.exit_menu();
                    },
                    _ => {},
                }
            }
        });

        self.state.borrow().load_items_if_any(&mut *self.items.borrow_mut());
    }

    fn cell_empty(&self, x: usize, y: usize) -> bool {
        if x >= 3 || y >= 3 { return true; }
        false
    }

    fn cell_visible(&self, x: usize, y: usize) -> bool {
        if x >= 3 || y >= 3 { return false; }
        if x == 0 && y == 0 || x == 2 && y == 0 { return false; }
        true
    }

    fn cell_label<'a>(&self, x: usize, y: usize, mut buf: &'a mut [u8]) -> Option<(&'a str, HexCell)> {
        if x >= 3 || y >= 3 { return None; }
        let mut len = 0;

        let mut hc = HexCell::Plain;

        self.with_item_at(x, y, &mut |item| {
            if let Some(item) = item {
                let lbl = item.as_str();
                len = buf.len().min(lbl.as_bytes().len());
                buf[0..len].copy_from_slice(&lbl.as_bytes()[0..len]);
            }
        });

        if let Ok(s) = std::str::from_utf8(&buf[0..len]) {
            Some((s, HexCell::Plain))
        } else {
            None
        }
    }

    fn cell_edge<'a>(&self, x: usize, y: usize, edge: HexDir, out: &'a mut [u8]) -> Option<(&'a str, HexEdge)> {
        None
    }
}

pub struct MatrixUIModel {
    matrix: Arc<Mutex<Matrix>>,
    menu:   Rc<MatrixUIMenu>,
    w:      usize,
    h:      usize,
}

impl HexGridModel for MatrixUIModel {
    fn width(&self) -> usize { self.w }
    fn height(&self) -> usize { self.h }

    fn cell_click(&self, x: usize, y: usize, btn: MButton) {

        println!("MATRIX CLICK CELL: {},{}: {:?}", x, y, btn);
        if MenuMode::None != *self.menu.mode.borrow() {
            *self.menu.mode.borrow_mut() = MenuMode::None;
        } else {
            match btn {
                MButton::Right => {
                    let mut m = self.matrix.lock().unwrap();
                    if let Some(mut cell) = m.get(x, y).copied() {
                        if let Some(node_info) = m.info_for(&cell.node_id()) {
                            self.menu.set_edge_assign_mode(cell, node_info);
                        }
                    }
                },
                _ => { self.menu.set_category_mode(); },
            }
        }
    }

    fn cell_empty(&self, x: usize, y: usize) -> bool {
        if x >= self.w || y >= self.h { return true; }
        false
    }

    fn cell_visible(&self, x: usize, y: usize) -> bool {
        if x >= self.w || y >= self.h { return false; }
        true
    }

    fn cell_label<'a>(&self, x: usize, y: usize, buf: &'a mut [u8]) -> Option<(&'a str, HexCell)> {
        if x >= self.w || y >= self.h { return None; }
        let m = self.matrix.lock().unwrap();
        if let Some(cell) = m.get(x, y) {
            if cell.node_id() != NodeId::Nop {
                println!("CELL {},{} => {:?}", x, y, cell);
            }
            Some((cell.label(buf)?, HexCell::Normal))
        } else {
            None
        }
    }

    fn cell_edge<'a>(&self, x: usize, y: usize, edge: HexDir, buf: &'a mut [u8]) -> Option<(&'a str, HexEdge)> {
        let m = self.matrix.lock().unwrap();
        if let Some(cell) = m.get(x, y) {
            if let Some((lbl, is_connected)) = m.edge_label(&cell, edge.into(), buf) {
                if is_connected {
                    Some((lbl, HexEdge::Arrow))
                } else {
            if cell.node_id() != NodeId::Nop {
                println!("CELL EDGE UNCON {},{} => (EDGE {:?}) {:?}", x, y, edge, cell);
            }
                    Some((lbl, HexEdge::NoArrow))
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct NodeMatrixData {
    hex_grid:     Box<WidgetData>,
    hex_menu:     Box<WidgetData>,

    matrix_model: Rc<MatrixUIModel>,

    grid_click_pos: Option<(f64, f64)>,
}

impl NodeMatrixData {
    pub fn new(matrix: Arc<Mutex<Matrix>>, pos: UIPos, node_id: u32) -> WidgetData {
        let wt_nmatrix  = Rc::new(NodeMatrix::new());

        let size = {
            let m = matrix.lock().unwrap();
            m.size()
        };

        let menu_model   = Rc::new(MatrixUIMenu::new(matrix.clone()));
        let matrix_model = Rc::new(MatrixUIModel {
            matrix,
            menu: menu_model.clone(),
            w: size.0,
            h: size.1,
        });

        let wt_hexgrid =
            Rc::new(HexGrid::new(14.0, 10.0));
        let wt_hexgrid_menu =
            Rc::new(HexGrid::new_y_offs(14.0, 10.0).bg_color(UI_GRID_BG2_CLR));

        WidgetData::new(
            wt_nmatrix,
            ParamID::new(node_id, 1),
            pos,
            Box::new(Self {
                hex_grid: WidgetData::new_tl_box(
                    wt_hexgrid.clone(),
                    ParamID::new(node_id, 1),
                    HexGridData::new(matrix_model.clone())),
                hex_menu: WidgetData::new_tl_box(
                    wt_hexgrid_menu.clone(),
                    ParamID::new(node_id, 2),
                    HexGridData::new(menu_model)),
                matrix_model,
                grid_click_pos: None,
            }))
    }
}

#[derive(Debug, Clone)]
pub struct NodeMatrix {
}

impl NodeMatrix {
    pub fn new() -> Self {
        Self { }
    }
}

impl WidgetType for NodeMatrix {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        data.with(|data: &mut NodeMatrixData| {
            (*data.hex_grid).draw(ui, p, pos);

            if let Some(mouse_pos) = data.grid_click_pos {
                if MenuMode::None != *data.matrix_model.menu.mode.borrow() {
                    let menu_w = 270.0;
                    let menu_h = 280.0;

                    let menu_rect =
                        Rect::from(
                            mouse_pos.0 - menu_w * 0.5,
                            mouse_pos.1 - menu_h * 0.5,
                            menu_w,
                            menu_h)
                        .move_into(&pos);

                    (*data.hex_menu).draw(ui, p, menu_rect);
                }
            }
        });
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Click { id, button, x, y, .. } => {
                println!("EV: {:?} id={}, data.id={}", ev, *id, data.id());
                data.with(|data: &mut NodeMatrixData| {
                    if *id == data.hex_grid.id() {
                        data.grid_click_pos = Some((*x, *y));
                        data.hex_grid.event(ui, ev);

                    } else if *id == data.hex_menu.id() {
                        data.hex_menu.event(ui, ev);
                    }

                    ui.queue_redraw();
                });
//                    data.with(|data: &mut NodeMatrixData| {
//                        if MenuMode::None != *data.matrix_model.menu.mode.borrow() {
//                        } else {
//                        }

//                        if let Some(_) = data.grid_click_pos {
//                            data.hex_menu.event(ui, ev);
//                            data.grid_click_pos = None;
//                        } else {
//                            match button {
//                                MButton::Right => {
//                                    data.matrix_model.menu.set_edge_mode();
//                                },
//                                _ => {
//                                    data.matrix_model.menu.set_category_mode();
//                                },
//                            }
//
//                        }
//                    });
//                }
            },
            UIEvent::FieldDrag { id, button, src, dst } => {
                data.with(|data: &mut NodeMatrixData| {
                    let mut m = data.matrix_model.matrix.lock().unwrap();
                    if let Some(mut src_cell) = m.get(src.0, src.1).copied() {
                        if let Some(dst_cell) = m.get(dst.0, dst.1).copied() {
                            match button {
                                MButton::Left => {
                                    m.place(dst.0, dst.1, src_cell);
                                    m.place(src.0, src.1, dst_cell);
                                    m.sync();
                                },
                                MButton::Right => {
                                    m.place(dst.0, dst.1, src_cell);
                                    m.sync();
                                },
                                MButton::Middle => {
                                    let unused_id = m.get_unused_instance_node_id(src_cell.node_id());
                                    src_cell.set_node_id(unused_id);
                                    m.place(dst.0, dst.1, src_cell);
                                    m.sync();
                                },
                            }
                        }
                    }
                });
                ui.queue_redraw();
            },
            _ => {
                println!("EV: {:?}", ev);
            },
        }
    }
}
