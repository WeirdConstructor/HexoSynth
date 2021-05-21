// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use crate::UICtrlRef;
use crate::matrix::*;
use crate::dsp::NodeId;
use crate::ui::menu::{Menu, MenuControl, MenuActionHandler};
use crate::ui::node_panel::{NodePanel, NodePanelData};
use crate::ui::util_panel::{UtilPanel, UtilPanelData};

use hexotk::widgets::hexgrid::HexGridModel;
use hexotk::{MButton, UIPos, AtomId};
use hexotk::{Rect, WidgetUI, Painter, WidgetData, WidgetType, UIEvent, wbox};
use hexotk::constants::*;
use hexotk::widgets::{
    HexGrid, HexGridData, HexCell, HexEdge, HexDir,
    Container, ContainerData,
    Text, TextSourceRef, TextData,
    DialogModel,
};

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

pub struct MatrixActionHandler {
    ui_ctrl:      UICtrlRef,
    help_txt:     Rc<TextSourceRef>,
}

impl MatrixActionHandler {
    pub fn new(ui_ctrl: UICtrlRef, help_txt: Rc<TextSourceRef>) -> Self {
        Self {
            ui_ctrl,
            help_txt,
        }
    }
}

impl MenuActionHandler for MatrixActionHandler {
    fn update_help_text(&mut self, txt: &str) {
        self.help_txt.set(txt);
    }

    fn assign_cell_port(
        &mut self, mut cell: Cell, cell_dir: CellDir, idx: Option<usize>)
    {
        self.ui_ctrl.assign_cell_port(cell, cell_dir, idx);
    }

    fn assign_cell_new_node(
        &mut self, mut cell: Cell, node_id: NodeId)
    {
        self.ui_ctrl.assign_cell_new_node(cell, node_id);
    }
}

pub struct MatrixUIMenu {
    menu: Rc<RefCell<dyn MenuControl>>,
}

impl MatrixUIMenu {
    pub fn new(ui_ctrl: UICtrlRef,
               help_txt: Rc<TextSourceRef>)
        -> Self
    {
        Self {
            menu: Rc::new(RefCell::new(
                Menu::new(
                    Box::new(MatrixActionHandler::new(
                        ui_ctrl,
                        help_txt))))),
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

    fn cell_click(&self, x: usize, y: usize, _btn: MButton, _modkey: bool) {
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

    fn cell_label<'a>(&self, x: usize, y: usize, buf: &'a mut [u8]) -> Option<(&'a str, HexCell, Option<(f32, f32)>)> {
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
            Some((s, HexCell::Plain, None))
        } else {
            None
        }
    }

    fn cell_edge<'a>(&self, _x: usize, _y: usize, _edge: HexDir, _out: &'a mut [u8]) -> Option<(&'a str, HexEdge)> {
        None
    }
}

pub struct MatrixUIModel {
    ui_ctrl: UICtrlRef,
    matrix: Arc<Mutex<Matrix>>,
    menu:   Rc<MatrixUIMenu>,

    dialog_model: Rc<RefCell<DialogModel>>,

    w:      usize,
    h:      usize,
}

impl HexGridModel for MatrixUIModel {
    fn width(&self) -> usize { self.w }
    fn height(&self) -> usize { self.h }

    fn cell_click(&self, x: usize, y: usize, btn: MButton, modkey: bool) {

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
                            if modkey {
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
                            self.ui_ctrl.clear_focus();
                        } else {
                            self.ui_ctrl.set_focus(cell);
                        }
                    } else {
                        self.ui_ctrl.clear_focus();
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

    fn cell_label<'a>(&self, x: usize, y: usize, buf: &'a mut [u8]) -> Option<(&'a str, HexCell, Option<(f32, f32)>)> {
        if x >= self.w || y >= self.h { return None; }
        let mut m = self.matrix.lock().unwrap();

        let hl =
            if self.ui_ctrl.is_cell_focussed(x, y) {
                HexCell::HLight
            } else {
                HexCell::Normal
            };

        let cell    = m.get(x, y)?;
        let label   = cell.label(buf)?;
        let node_id = cell.node_id();
        let v       = m.filtered_led_for(&node_id);
        Some((label, hl, Some(v)))
    }

    fn cell_edge<'a>(&self, x: usize, y: usize, edge: HexDir, buf: &'a mut [u8]) -> Option<(&'a str, HexEdge)> {
        let mut m = self.matrix.lock().unwrap();

        let mut edge_lbl = None;
        let mut out_fb_info = None;

        if let Some(cell) = m.get(x, y) {
            let cell_dir = edge.into();

            if let Some((lbl, is_connected)) = m.edge_label(&cell, cell_dir, buf) {
                edge_lbl = Some(lbl);

                if is_connected {
                    if let Some(out_idx) = cell.local_port_idx(cell_dir) {
                        out_fb_info = Some((cell.node_id(), out_idx));
                    }
                }
            }
        }

        if let Some(lbl) = edge_lbl {
            if let Some((node_id, out)) = out_fb_info {
                let val =
                    m.filtered_out_fb_for(&node_id, out);

                Some((lbl, HexEdge::ArrowValue { value: val }))
            } else {
                Some((lbl, HexEdge::NoArrow))
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
    util_panel:   Box<WidgetData>,

    matrix_model: Rc<MatrixUIModel>,
    ui_ctrl:      UICtrlRef,

    grid_click_pos: Option<(f64, f64)>,
}

const HEX_MATRIX_ID         : u32 = 1;
const HEX_GRID_ID           : u32 = 2;
const HEX_MENU_CONT_ID      : u32 = 3;
const HEX_MENU_HELP_TEXT_ID : u32 = 4;
const HEX_MENU_ID           : u32 = 5;
const NODE_PANEL_ID         : u32 = 11;
const UTIL_PANEL_ID         : u32 = 12;

impl NodeMatrixData {
    pub fn new(
        ui_ctrl: UICtrlRef,
        matrix: Arc<Mutex<Matrix>>,
        dialog_model: Rc<RefCell<DialogModel>>,
        pos: UIPos,
        node_id: u32)
    -> WidgetData
    {
        let wt_nmatrix  = Rc::new(NodeMatrix::new());

        let size = ui_ctrl.with_matrix(|m| m.size());

        let txtsrc = Rc::new(TextSourceRef::new(30));

        let menu_model =
            Rc::new(MatrixUIMenu::new(
                ui_ctrl.clone(),
                txtsrc.clone()));

        let matrix_model = Rc::new(MatrixUIModel {
            ui_ctrl:        ui_ctrl.clone(),
            matrix:         matrix.clone(),
            dialog_model,
            menu:           menu_model.clone(),
            w:              size.0,
            h:              size.1,
        });

        let wt_node_panel = Rc::new(NodePanel::new());
        let wt_hexgrid =
            Rc::new(HexGrid::new(14.0, 10.0, 54.0));
        let wt_hexgrid_menu =
            Rc::new(HexGrid::new_y_offs_pinned(12.0, 10.0, 45.0)
                    .bg_color(UI_GRID_BG2_CLR));
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
                ui_ctrl: ui_ctrl.clone(),
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
                    NodePanelData::new(ui_ctrl.clone(), node_id, matrix.clone()))),
                util_panel: Box::new(wbox!(
                    UtilPanel::new_ref(),
                    AtomId::new(node_id, UTIL_PANEL_ID),
                    center(12, 12),
                    UtilPanelData::new(ui_ctrl.clone()))),
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

            if let Ok(mut m) = data.matrix_model.matrix.lock() {
                m.update_filters();
            }

            let panel_pos = pos.resize(360.0, pos.h);
            let util_pos =
                pos.resize(355.0, pos.h - 5.0)
                   .offs(pos.w - 360.0, 0.0);

            let hex_pos = pos.shrink(365.0, 0.0);
            (*data.hex_grid).draw(ui, p, hex_pos);
            (*data.node_panel).draw(ui, p, panel_pos);
            (*data.util_panel).draw(ui, p, util_pos);

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
                                    data.util_panel.event(ui, ev);
                                }
                            },
                            _ => {
                                if *id == data.hex_grid.id() {
                                    data.hex_grid.event(ui, ev);
                                } else {
                                    data.node_panel.event(ui, ev);
                                    data.util_panel.event(ui, ev);
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
                                if data.matrix_model.ui_ctrl.is_cell_focussed(src.0, src.1) {
                                    data.matrix_model.ui_ctrl.set_focus(
                                        src_cell.with_pos_of(dst_cell));
                                }

                                crate::ui_ctrl::handle_matrix_change(
                                    &data.matrix_model.dialog_model, ||
                                {
                                    match button {
                                        MButton::Left => {
                                            m.change_matrix(|m| {
                                                m.place(dst.0, dst.1, src_cell);
                                                m.place(src.0, src.1, dst_cell);
                                            })?;
                                           m.sync()?;
                                        },
                                        MButton::Right => {
                                            m.change_matrix(|m| {
                                                m.place(dst.0, dst.1, src_cell);
                                            })?;
                                            m.sync()?;
                                        },
                                        MButton::Middle => {
                                            let unused_id = m.get_unused_instance_node_id(src_cell.node_id());
                                            src_cell.set_node_id(unused_id);
                                            m.change_matrix(|m| {
                                                m.place(dst.0, dst.1, src_cell);
                                            })?;
                                            m.sync()?;
                                        },
                                    }

                                    Ok(())
                                });
                            }
                        }
                    }
                });
                ui.queue_redraw();
            },
            UIEvent::Key { key, .. } => {
                use keyboard_types::Key;

                println!("KEY!");

                match key {
                    Key::F4 => {
                        data.with(|data: &mut NodeMatrixData| {
                            use crate::matrix_repr::save_patch_to_file;

                            let mut m =
                                data.matrix_model.matrix.lock().unwrap();

                            println!("SAVE!");
                            save_patch_to_file(
                                &mut m,
                                "init.hxy").unwrap();
                        });
                    },
                    _ => {
                        data.with(|data: &mut NodeMatrixData| {
                            data.util_panel.event(ui, ev);
                        });
                    },
                }

            },
            _ => {
            println!("FOOEFO");
                data.with(|data: &mut NodeMatrixData| {
                    data.util_panel.event(ui, ev);
                });
            },
        }
    }
}
