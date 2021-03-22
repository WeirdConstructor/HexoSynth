// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use hexotk::widgets::hexgrid::HexGridModel;
use hexotk::{MButton, UIPos, AtomId};
use hexotk::{Rect, WidgetUI, Painter, WidgetData, WidgetType, UIEvent, wbox};
use hexotk::constants::*;
use hexotk::widgets::{
    HexGrid, HexGridData, HexCell, HexEdge, HexDir,
    Container, ContainerData,
    Text, TextSourceRef, TextData,
};

use std::rc::Rc;
use std::cell::RefCell;

use crate::matrix::*;
use std::sync::Arc;
use std::sync::Mutex;

use crate::dsp::NodeId;
use crate::ui::menu::{Menu, MenuControl, MenuActionHandler};
use crate::ui::node_panel::{NodePanel, NodePanelData};

pub struct MatrixActionHandler {
    matrix:   Arc<Mutex<Matrix>>,
    help_txt: Rc<TextSourceRef>,
}

impl MatrixActionHandler {
    pub fn new(help_txt: Rc<TextSourceRef>, matrix: Arc<Mutex<Matrix>>) -> Self {
        Self {
            matrix,
            help_txt,
        }
    }
}

impl MenuActionHandler for MatrixActionHandler {
    fn update_help_text(&mut self, txt: &str) {
        self.help_txt.set(txt);
    }

    fn assign_cell_port(&mut self, mut cell: Cell, cell_dir: CellDir, idx: Option<usize>) {
        let mut m = self.matrix.lock().unwrap();

        if let Some(idx) = idx {
            cell.set_io_dir(cell_dir, idx);
        } else {
            cell.clear_io_dir(cell_dir);
        }
        let pos = cell.pos();
        m.place(pos.0, pos.1, cell);
        m.sync();
    }

    fn assign_cell_new_node(&mut self, mut cell: Cell, node_id: NodeId) {
        let mut m = self.matrix.lock().unwrap();

        let node_id = m.get_unused_instance_node_id(node_id);
        cell.set_node_id(node_id);
        let pos = cell.pos();
        m.place(pos.0, pos.1, cell);
        m.sync();
    }
}

pub struct MatrixUIMenu {
    menu: Rc<RefCell<dyn MenuControl>>,
}

impl MatrixUIMenu {
    pub fn new(matrix: Arc<Mutex<Matrix>>, help_txt: Rc<TextSourceRef>) -> Self {
        Self {
            menu: Rc::new(RefCell::new(
                Menu::new(
                    Box::new(MatrixActionHandler::new(
                        help_txt,
                        matrix.clone()))))),
        }
    }

    pub fn grid2index(&self, x: usize, y: usize) -> Option<usize> {
        match (x, y) {
            // Center
            (1, 1) => Some(0),
            // TR
            (2, 1) => Some(2),
            // BR
            (2, 2) => Some(3),
            // B
            (1, 2) => Some(4),
            // BL
            (0, 2) => Some(5),
            // TL
            (0, 1) => Some(6),
            // T
            (1, 0) => Some(1),
            _      => None,
        }
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

    fn cell_hover(&self, x: usize, y: usize) {
        if let Some(idx) = self.grid2index(x, y) {
            self.menu.borrow_mut().set_hover_index(idx);
        }
        self.menu.borrow_mut().update();
    }

    fn cell_click(&self, x: usize, y: usize, _btn: MButton, _shift: bool) {
        if let Some(idx) = self.grid2index(x, y) {
            self.menu.borrow_mut().select(idx);
        }
        self.menu.borrow_mut().update();
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

    fn cell_label<'a>(&self, x: usize, y: usize, buf: &'a mut [u8]) -> Option<(&'a str, HexCell)> {
        if x >= 3 || y >= 3 { return None; }
        let mut len = 0;

        if let Some(idx) = self.grid2index(x, y) {
            let menu = self.menu.borrow_mut();
            if let Some(lbl) = menu.label(idx) {
                len = buf.len().min(lbl.as_bytes().len());
                buf[0..len].copy_from_slice(&lbl.as_bytes()[0..len]);
            }
        }

        if let Ok(s) = std::str::from_utf8(&buf[0..len]) {
            Some((s, HexCell::Plain))
        } else {
            None
        }
    }

    fn cell_edge<'a>(&self, _x: usize, _y: usize, _edge: HexDir, _out: &'a mut [u8]) -> Option<(&'a str, HexEdge)> {
        None
    }
}

#[derive(Debug)]
pub struct MatrixEditor {
    focus_cell: Cell,
}

impl MatrixEditor {
    pub fn new() -> Self {
        Self {
            focus_cell: Cell::empty(NodeId::Nop),
        }
    }
}

#[derive(Clone)]
pub struct MatrixEditorRef(Rc<RefCell<MatrixEditor>>);

impl std::fmt::Debug for MatrixEditorRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MatrixEditorRef({:?})", *self.0.borrow())
    }
}

impl MatrixEditorRef {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(MatrixEditor::new())))
    }

    pub fn get_recent_focus(&self) -> Cell {
        self.0.borrow().focus_cell
    }

    pub fn is_cell_focussed(&self, x: usize, y: usize) -> bool {
        let cell = self.0.borrow().focus_cell;

        if cell.node_id() == NodeId::Nop {
            return false;
        }

        let (cx, cy) = cell.pos();
        cx == x && cy == y
    }

    pub fn clear_focus(&self) {
        self.0.borrow_mut().focus_cell = Cell::empty(NodeId::Nop);
    }

    pub fn set_focus(&self, cell: Cell) {
        self.0.borrow_mut().focus_cell = cell;
    }
}


pub struct MatrixUIModel {
    matrix: Arc<Mutex<Matrix>>,
    menu:   Rc<MatrixUIMenu>,

    editor: MatrixEditorRef,

    w:      usize,
    h:      usize,
}

impl HexGridModel for MatrixUIModel {
    fn width(&self) -> usize { self.w }
    fn height(&self) -> usize { self.h }

    fn cell_click(&self, x: usize, y: usize, btn: MButton, shift: bool) {

        println!("MATRIX CLICK CELL: {},{}: {:?}", x, y, btn);
        let mut menu = self.menu.menu.borrow_mut();

        if menu.is_open() {
            menu.close();

        } else {
            match btn {
                MButton::Right => {
                    let m = self.matrix.lock().unwrap();
                    if let Some(cell) = m.get_copy(x, y) {
                        if let Some(node_info) = m.info_for(&cell.node_id()) {
                            if shift {
                                menu.open_select_cell_dir(cell, node_info);
                            } else {
                            }
                        } else {
                            menu.open_select_node_category(cell);
                        }
                    }
                },
                MButton::Left => {
                    let m = self.matrix.lock().unwrap();
                    if let Some(cell) = m.get_copy(x, y) {
                        if cell.node_id() == NodeId::Nop {
                            self.editor.clear_focus();
                        } else {
                            self.editor.set_focus(cell);
                        }
                    } else {
                        self.editor.clear_focus();
                    }
                },
                _ => {},
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

        let hl =
            if self.editor.is_cell_focussed(x, y) {
                HexCell::HLight
            } else {
                HexCell::Normal
            };

        if let Some(cell) = m.get(x, y) {
            Some((cell.label(buf)?, hl))
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
    hex_menu_id:  AtomId,
    #[allow(dead_code)]
    node_panel:   Box<WidgetData>,

    matrix_model: Rc<MatrixUIModel>,

    grid_click_pos: Option<(f64, f64)>,
}

const HEX_MATRIX_ID         : u32 = 1;
const HEX_GRID_ID           : u32 = 2;
const HEX_MENU_CONT_ID      : u32 = 3;
const HEX_MENU_HELP_TEXT_ID : u32 = 4;
const HEX_MENU_ID           : u32 = 5;
const NODE_PANEL_ID         : u32 = 11;

impl NodeMatrixData {
    pub fn new(matrix: Arc<Mutex<Matrix>>, pos: UIPos, node_id: u32) -> WidgetData {
        let wt_nmatrix  = Rc::new(NodeMatrix::new());

        let size = {
            let m = matrix.lock().unwrap();
            m.size()
        };

        let txtsrc = Rc::new(TextSourceRef::new(30));

        let editor = MatrixEditorRef::new();

        let menu_model   = Rc::new(MatrixUIMenu::new(matrix.clone(), txtsrc.clone()));
        let matrix_model = Rc::new(MatrixUIModel {
            matrix:         matrix.clone(),
            menu:           menu_model.clone(),
            editor:         editor.clone(),
            w:              size.0,
            h:              size.1,
        });

        let wt_node_panel = Rc::new(NodePanel::new());
        let wt_hexgrid =
            Rc::new(HexGrid::new(14.0, 10.0, 54.0));
        let wt_hexgrid_menu =
            Rc::new(HexGrid::new_y_offs(12.0, 10.0, 45.0).bg_color(UI_GRID_BG2_CLR));
        let wt_cont = Rc::new(Container::new());
        let wt_text = Rc::new(Text::new(12.0));

        let hex_menu_id = AtomId::new(node_id, HEX_MENU_ID);
        let mut hex_menu = ContainerData::new();
        hex_menu.contrast_border()
           .title("Menu")
           .new_row()
           .add(wbox!(
                wt_hexgrid_menu,
                hex_menu_id,
                center(6, 12),
                HexGridData::new(menu_model)))
           .add(wbox!(wt_text,
                AtomId::new(node_id, HEX_MENU_HELP_TEXT_ID),
                center(6, 12),
                TextData::new(txtsrc.clone())));

        WidgetData::new(
            wt_nmatrix,
            AtomId::new(node_id, HEX_MATRIX_ID),
            pos,
            Box::new(Self {
                hex_grid: WidgetData::new_tl_box(
                    wt_hexgrid.clone(),
                    AtomId::new(node_id, HEX_GRID_ID),
                    HexGridData::new(matrix_model.clone())),
                hex_menu: WidgetData::new_tl_box(
                    wt_cont,
                    AtomId::new(node_id, HEX_MENU_CONT_ID),
                    hex_menu),
                node_panel: Box::new(wbox!(
                    wt_node_panel,
                    AtomId::new(node_id, NODE_PANEL_ID),
                    center(12, 12),
                    NodePanelData::new(node_id, matrix.clone(), editor))),
                hex_menu_id,
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

            let panel_pos = pos.resize(360.0, pos.h);

            let hex_pos = pos.shrink(365.0, 0.0);
            (*data.hex_grid).draw(ui, p, hex_pos);
            (*data.node_panel).draw(ui, p, panel_pos);

            if let Some(mouse_pos) = data.grid_click_pos {
                if data.matrix_model.menu.menu.borrow().is_open() {
                    let hex_w = 235.0;
                    let txt_w = (hex_w / 6.0) * 6.0;
                    let menu_w = hex_w + txt_w;
                    let menu_h = 240.0 + UI_ELEM_TXT_H + 2.0 * UI_BORDER_WIDTH;

                    let menu_rect =
                        Rect::from(
                            mouse_pos.0 - (hex_w * 0.5),
                            mouse_pos.1 - menu_h * 0.5,
                            menu_w,
                            menu_h)
                        .move_into(&pos);

                    let _hz = ui.hover_zone_for(data.hex_menu_id);
                    //d// println!("HOVEER: {:?}", hz);

                    (*data.hex_menu).draw(ui, p, menu_rect);
                }
            }
        });
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Click { id, x, y, button, .. } => {
                println!("EV: {:?} id={}, btn={:?}, data.id={}",
                         ev, *id, button, data.id());

                data.with(|data: &mut NodeMatrixData| {
                    if *id == data.hex_menu_id {
                        data.hex_menu.event(ui, ev);

                    } else {
                        match button {
                            MButton::Right => {
                                if *id == data.hex_grid.id() {
                                    data.grid_click_pos = Some((*x, *y));
                                    data.hex_grid.event(ui, ev);
                                    data.matrix_model.menu.menu.borrow_mut().update();
                                } else {
                                    data.node_panel.event(ui, ev);
                                }
                            },
                            _ => {
                                if *id == data.hex_grid.id() {
                                    data.hex_grid.event(ui, ev);
                                } else {
                                    data.node_panel.event(ui, ev);
                                }
                            },
                        }
                    }

                    ui.queue_redraw();
                });
            },
            UIEvent::FieldDrag { id, button, src, dst, .. } => {
                data.with(|data: &mut NodeMatrixData| {
                    if *id == data.hex_grid.id() {
                        let mut m = data.matrix_model.matrix.lock().unwrap();
                        if let Some(mut src_cell) = m.get(src.0, src.1).copied() {
                            if let Some(dst_cell) = m.get(dst.0, dst.1).copied() {
                                if data.matrix_model.editor.is_cell_focussed(src.0, src.1) {
                                    data.matrix_model.editor.set_focus(
                                        src_cell.with_pos_of(dst_cell));
                                }

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
