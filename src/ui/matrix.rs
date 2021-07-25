// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use crate::{UICtrlRef, UICellTrans};
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
    HexGrid, HexGridData, HexHLight, HexEdge, HexDir, HexCell,
    Container, ContainerData,
    Text, TextSourceRef, TextData,
    Tabs, TabsData,
};

use std::rc::Rc;
use std::cell::RefCell;

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
        &mut self, cell: Cell, cell_dir: CellDir, idx: Option<usize>)
    {
        self.ui_ctrl.assign_cell_port(cell, cell_dir, idx);
    }

    fn clear_cell_ports(&mut self, cell: Cell) {
        self.ui_ctrl.clear_cell_ports(cell);
    }

    fn assign_cell_new_node(&mut self, cell: Cell, node_id: NodeId) {
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
        if (x == 0 || x == 2) && y == 0 { return false; }
        true
    }

    fn cell_label<'a>(&self, x: usize, y: usize, buf: &'a mut [u8])
        -> Option<HexCell<'a>>
    {
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
            Some(HexCell {
                label:     s,
                hlight: HexHLight::Plain,
                rg_colors: None
            })
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
    menu:   Rc<MatrixUIMenu>,

    w:      usize,
    h:      usize,
}

impl HexGridModel for MatrixUIModel {
    fn width(&self) -> usize { self.w }
    fn height(&self) -> usize { self.h }

    fn cell_click(&self, x: usize, y: usize, btn: MButton, modkey: bool) {
        let mut menu = self.menu.menu.borrow_mut();

        if menu.is_open() {
            menu.close();

        } else {
            match btn {
                MButton::Right => {
                    self.ui_ctrl.with_matrix(|m| {
                        if let Some(cell) = m.get_copy(x, y) {
                            if let Some(node_info) = m.info_for(&cell.node_id()) {
                                if modkey {
                                    menu.open_select_cell_dir(cell, node_info);
                                } else {
                                    menu.open_node_context(cell, node_info);
                                }
                            } else {
                                menu.open_select_node_category(cell);
                            }
                        }
                    });
                },
                MButton::Left => {
                    let cell = self.ui_ctrl.with_matrix(|m| m.get_copy(x, y));

                    if let Some(cell) = cell {
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
        self.ui_ctrl.with_matrix(|m| {
            if let Some(cell) = m.get(x, y) {
                cell.node_id() == NodeId::Nop
            } else {
                true
            }
        })
    }

    fn cell_visible(&self, x: usize, y: usize) -> bool {
        if x >= self.w || y >= self.h { return false; }
        true
    }

    fn cell_label<'a>(&self, x: usize, y: usize, buf: &'a mut [u8])
        -> Option<HexCell<'a>>
    {
        if x >= self.w || y >= self.h { return None; }
        let (cell, led_value) =
            self.ui_ctrl.with_matrix(|m| {
                let cell    = m.get_copy(x, y)?;
                let node_id = cell.node_id();
                let v       = m.filtered_led_for(&node_id);

                Some((cell, v))
            })?;

        let label = cell.label(buf)?;

        let hl =
            if self.ui_ctrl.is_cell_focussed(x, y) {
                HexHLight::HLight
            } else {
                HexHLight::Normal
            };

        Some(HexCell { label, hlight: hl, rg_colors: Some(led_value) })
    }

    fn cell_edge<'a>(&self, x: usize, y: usize, edge: HexDir, buf: &'a mut [u8]) -> Option<(&'a str, HexEdge)> {
        self.ui_ctrl.with_matrix(move |m| {
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
        })
    }
}

pub struct NodeMatrixData {
    hex_grid:     Box<WidgetData>,
    hex_menu:     Box<WidgetData>,
    hex_menu_id:  AtomId,
    #[allow(dead_code)]
    node_panel:   Box<WidgetData>,
    util_panel:   Box<WidgetData>,
    help_text:    Box<WidgetData>,
    show_help:    bool,

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
const HELP_TEXT_ID          : u32 = 13;
const HELP_TEXT_SHORTCUT_ID : u32 = 14;
const HELP_TEXT_ABOUT_ID    : u32 = 15;

#[allow(clippy::new_ret_no_self)]
impl NodeMatrixData {
    pub fn new(
        ui_ctrl: UICtrlRef,
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
        let wt_cont         = Rc::new(Container::new());
        let wt_text         = Rc::new(Text::new(12.0));
        let wt_help_txt     = Rc::new(Text::new(14.0));
        let wt_log_txt      = Rc::new(Text::new(9.0));

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
                TextData::new(txtsrc)));

        let mut tdata = TabsData::new();

        let about_text =
            Rc::new(TextSourceRef::new(crate::ui::UI_MAIN_HELP_TEXT_WIDTH));
        about_text.set(r#"About HexoSynth
HexoSynth is a modular synthesizer where the graph is
represented as hexagonal tile map. The 6 edges of each tile
are the ports of the nodes (aka modules). The top and left edges
are the input edges, and the bottom and right edges are the outputs.

ATTENTION: For help please take a look at the other tabs of this about screen at the top!

-------------------------------

HexoSynth modular synthesizer
Copyright (C) 2021  Weird Constructor <weirdconstructor@gmail.com>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
"#);

        let key_text =
            Rc::new(TextSourceRef::new(crate::ui::UI_MAIN_HELP_TEXT_WIDTH));
        key_text.set(r#"Parameter Knobs
    Parameter knobs have two areas where you can grab them:
    * Center/Value label is the coarse area.
    * Name Label below center is the fine adjustment area.
      The fine adjustment area will highlight and display the
      raw signal value of the input parameter. This can be useful
      if you want to build modulators that reach exactly a certain value.

    Parameter Knobs are greyed out when it's corresponding input
    is connected to an output. That means, the parameter value is not
    been used. You can still change it if you want though, it just wont
    make a difference as long as the input is in use.

    Drag LMB Up/Down                - Adjust parameter.
    Drag RMB Up/Down                - Adjust parameter modulation amount.
    Hover over knob + Backspace     - Remove parameter modulation amount.
    Hover over knob + Delete        - Remove parameter modulation amount.
    Shift + Drag LMB Up/Down        - Fine adjust parameter.
    Shift + Drag BMB Up/Down        - Fine adjust parameter mod. amount.
    Ctrl  + Drag LMB Up/Down        - Disable parameter snap. (eg. for the
                                      detune parameter)
    Ctrl + Shift + Drag LMB Up/Down - Fine adjustment of parameters with
                                      disabled parameter snap.
    MMB                             - Reset Parameter to it's default value.
    MMB (Knob fine adj. area)       - Reset Parameter to it's default value
                                      and remove modulation amount.
    Hover over Knob + Enter         - Open the direct value entry.
    or:                               Coarse adjustment area will edit the
    Ctrl + RMB                        denormalized value. Fine adjustment
                                      area will edit the normalized
                                      -1..1 or 0..1 signal value. Hit 'Esc'
                                      to exit the value entry without change.

    Combining the fine adjustment areas with the Shift key allows a freedom
    of 4 resolutions to adjust parameters.

LMB = Left Mouse Button, RMB = Right Mouse Button, MMB = Middle Mouse Button
Next page: Hex Grid
---page---
Hex Grid

    RMB         - Open context menu
    Ctrl + RMB  - Assign edge menu to set inputs/outputs of clicked node.

    Drag LMB    - Move / Swap Nodes
    Drag RMB    - Linked clone of dragged node
    Drag MMB    - New instance of dragged node type

    Shift + Drag LMB Up/Down - Pan hex grid
    Shift + Drag RMB Up/Down - Zoom Out/IN

    w, q, a     - Assign input port to input 1, 2 or 3
    e, d, s     - Assign output port to output 1, 2 or 3

LMB = Left Mouse Button, RMB = Right Mouse Button, MMB = Middle Mouse Button
"#);

        let tracker_key_text =
            Rc::new(TextSourceRef::new(crate::ui::UI_MAIN_HELP_TEXT_WIDTH));
        tracker_key_text.set(r#"Tracker / Pattern Editor Keyboard Shortcuts
* Normal Mode

    Return              - Enter Edit Mode
    Escape              - Exit Edit Mode

    Home                - Cursor to first row
    End                 - Cursor to last row (within edit step)
    Page Up             - Cursor up by 2 edit steps
    Page Down           - Cursor down by 2 edit steps
    Up/Down/Left/Right  - Move Cursor
    'f'                 - Toggle cursor follow phase bar

    Del                 - Delete value in cell at cursor
    '+' / '-'           - In-/Decrease note enter mode octave
    '*' / '/' (Keypad)  - In-/Decrease edit step by 1
    'r'                 - Enter new pattern rows / length mode
    'e'                 - Enter new edit step mode
    'o'                 - Enter octave mode
    'c'                 - Change column type mode
    'd'                 - Delete col/row/step mode

    Shift + PgUp   - (+ 0x100) Increase 1st nibble of value under cursor
    Shift + PgDown - (- 0x100) Decrease 1st nibble of value under cursor
    Shift + Up     - (+ 0x010) Increase 2nd nibble of value under cursor
    Shift + Down   - (- 0x010) Decrease 2nd nibble of value under cursor
    Shift + Right  - (+ 0x001) Increase 3rd nibble of value under cursor
    Shift + Left   - (- 0x001) Decrease 3rd nibble of value under cursor

* Edit Mode

    Up/Down/Left/Right - Move Cursor

    '.'                - Enter most recently entered value
                         and advance one edit step.
    ','                - Remember the current cell value as most recently
                         used value and advance one edit step.
                         Useful for copying a value and paste it with '.'.
    Note Column  :    Note entering via keyboard "like Renoise".
    Other Columns:    '0'-'9', 'a'-'f' - Enter value
"#);
        tdata.add(
            "Matrix",
            wbox!(
                wt_help_txt,
                AtomId::new(node_id, HELP_TEXT_SHORTCUT_ID),
                center(12, 12),
                TextData::new(key_text)));

        tdata.add(
            "Tracker",
            wbox!(
                wt_help_txt,
                AtomId::new(node_id, HELP_TEXT_SHORTCUT_ID),
                center(12, 12),
                TextData::new(tracker_key_text)));


        tdata.add(
            "Module",
            wbox!(
                wt_help_txt,
                AtomId::new(node_id, HELP_TEXT_ID),
                center(12, 12),
                TextData::new(ui_ctrl.get_help_text_src())));

        tdata.add(
            "Log",
            wbox!(
                wt_log_txt,
                AtomId::new(node_id, HELP_TEXT_ID),
                center(12, 12),
                TextData::new(ui_ctrl.get_log_src())));

        tdata.add(
            "About",
            wbox!(
                wt_help_txt,
                AtomId::new(node_id, HELP_TEXT_ABOUT_ID),
                center(12, 12),
                TextData::new(about_text)));

        let help_text =
            WidgetData::new_tl_box(
                Tabs::new_ref(),
                AtomId::new(crate::HELP_TABS_ID, 0),
                tdata);

        WidgetData::new(
            wt_nmatrix,
            AtomId::new(node_id, HEX_MATRIX_ID),
            pos,
            Box::new(Self {
                show_help: false,
                ui_ctrl: ui_ctrl.clone(),
                hex_grid: WidgetData::new_tl_box(
                    wt_hexgrid,
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
                    NodePanelData::new(ui_ctrl.clone(), node_id))),
                util_panel: Box::new(wbox!(
                    UtilPanel::new_ref(),
                    AtomId::new(node_id, UTIL_PANEL_ID),
                    center(12, 12),
                    UtilPanelData::new(ui_ctrl))),
                help_text,
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

impl Default for NodeMatrix {
    fn default() -> Self { Self::new() }
}

impl WidgetType for NodeMatrix {
    fn draw(
        &self, ui: &mut dyn WidgetUI,
        data: &mut WidgetData, p: &mut dyn Painter, pos: Rect
    ) {
        data.with(|data: &mut NodeMatrixData| {

            data.ui_ctrl.with_matrix(|m| m.update_filters());

            if data.ui_ctrl.check_help_toggle() {
                data.show_help = !data.show_help;
            }

            let panel_pos = pos.resize(360.0, pos.h);
            let util_pos =
                pos.resize(355.0, pos.h - 5.0)
                   .offs(pos.w - 360.0, 0.0);

            let hex_pos = pos.shrink(365.0, 0.0);
            if !data.show_help {
                (*data.hex_grid).draw(ui, p, hex_pos);
            }

            (*data.node_panel).draw(ui, p, panel_pos);
            (*data.util_panel).draw(ui, p, util_pos);

            if data.show_help {
                (*data.help_text).draw(ui, p, hex_pos);
            }

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
                // println!("EV: {:?} id={}, btn={:?}, data.id={}",
                //          ev, *id, button, data.id());

                data.with(|data: &mut NodeMatrixData| {
                    if data.show_help {
                        data.help_text.event(ui, ev);
                        if *id == data.hex_menu_id {
                            data.hex_menu.event(ui, ev);
                        } else {
                            data.node_panel.event(ui, ev);
                            data.util_panel.event(ui, ev);
                        }

                    } else if *id == data.hex_menu_id {
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
                        data.matrix_model.ui_ctrl.cell_transform(
                            *src,
                            *dst,
                            match button {
                                MButton::Left   => UICellTrans::Swap,
                                MButton::Right  => UICellTrans::CopyTo,
                                MButton::Middle => UICellTrans::Instanciate,
                            });
                    }
                });
                ui.queue_redraw();
            },
            UIEvent::Key { id, key, mouse_pos, .. } => {
                use keyboard_types::Key;

                if *id == data.id() {
                    match key {
                        Key::F1 => {
                            data.with(|data: &mut NodeMatrixData| {
                                data.show_help = !data.show_help;
                            });
                        },
                        Key::Escape => {
                            data.with(|data: &mut NodeMatrixData| {
                                if data.show_help {
                                    data.show_help = false;
                                }
                            });
                        },
                        Key::F4 => {
                            data.with(|data: &mut NodeMatrixData| {
                                data.matrix_model.ui_ctrl.save_patch();
                            });
                        },
                        Key::Character(c) => {
                            data.with(|data: &mut NodeMatrixData| {
                                let ui_ctrl   = &data.matrix_model.ui_ctrl;
                                let cell      = ui_ctrl.get_recent_focus();
                                let node_info = ui_ctrl.get_focus_node_info();

                                let mut assign_port_dir = None;

                                match &c[..] {
                                    "w" => { assign_port_dir = Some(CellDir::T); },
                                    "q" => { assign_port_dir = Some(CellDir::TL); },
                                    "a" => { assign_port_dir = Some(CellDir::BL); },
                                    "e" => { assign_port_dir = Some(CellDir::TR); },
                                    "d" => { assign_port_dir = Some(CellDir::BR); },
                                    "s" => { assign_port_dir = Some(CellDir::B); },
                                    _ => {},
                                }

                                if let Some(dir) = assign_port_dir {
                                    if cell.node_id() != NodeId::Nop {
                                        data.matrix_model.menu.menu
                                            .borrow_mut()
                                            .open_assign_port(
                                                cell, node_info, dir);
                                        data.grid_click_pos = Some(*mouse_pos);
                                        ui.queue_redraw();
                                    }
                                }
                            });

                            data.with(|data: &mut NodeMatrixData| {
                                data.util_panel.event(ui, ev);
                            });
                        },
                        _ => {
                            data.with(|data: &mut NodeMatrixData| {
                                data.util_panel.event(ui, ev);
                            });
                        },
                    }
                } else {
                    data.with(|data: &mut NodeMatrixData| {
                        data.util_panel.event(ui, ev);
                    });
                }
            },
            _ => {
                data.with(|data: &mut NodeMatrixData| {
                    data.util_panel.event(ui, ev);
                });
            },
        }
    }
}
