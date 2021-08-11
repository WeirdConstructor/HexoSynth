// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::{UICtrlRef, Msg};
use crate::dsp::NodeId;
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

pub struct MatrixUIMenu {
    ui_ctrl:    UICtrlRef,
}

impl MatrixUIMenu {
    pub fn new(ui_ctrl: UICtrlRef, _help_txt: Rc<TextSourceRef>)
        -> Self
    {
        Self { ui_ctrl, }
    }

    pub fn grid2index(&self, x: usize, y: usize) -> Option<usize> {
        let size = self.width() * self.height();

        if x % 2 == 0 && y == 0 { return None; }

        let w0 = self.width() - 1;
        let w1 = self.width();

        if size <= 9 {
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

        } else if size <= 4 * 4 {
            if x < 4 && y < 4 {
                match x {
                    0 => Some(y - 1),
                    1 => Some(w0 + y),
                    2 => Some(w0 + w1 + y - 1),
                    3 => Some(w0 + w1 + w0 + y),
                    _ => None,
                }
            } else {
                None
            }

        } else if size <= 5 * 5 {
            if x < 5 && y < 5 {

                match x {
                    0 => Some(y - 1),
                    1 => Some(w0 + y),
                    2 => Some(w0 + w1 + y - 1),
                    3 => Some(w0 + w1 + w0 + y),
                    4 => Some(w0 + w1 + w0 + w1 + y - 1),
                    _ => None,
                }
            } else {
                None
            }

        } else if size <= 6 * 6 {
            if x < 6 && y < 6 {

                match x {
                    0 => Some(y - 1),
                    1 => Some(w0 + y),
                    2 => Some(w0 + w1 + y - 1),
                    3 => Some(w0 + w1 + w0 + y),
                    4 => Some(w0 + w1 + w0 + w1 + y - 1),
                    5 => Some(w0 + w1 + w0 + w1 + w0 + y),
                    _ => None,
                }
            } else {
                None
            }

        } else if size <= 7 * 7 {
            if x < 7 && y < 7 {

                match x {
                    0 => Some(y - 1),
                    1 => Some(w0 + y),
                    2 => Some(w0 + w1 + y - 1),
                    3 => Some(w0 + w1 + w0 + y),
                    4 => Some(w0 + w1 + w0 + w1 + y - 1),
                    5 => Some(w0 + w1 + w0 + w1 + w0 + y),
                    6 => Some(w0 + w1 + w0 + w1 + w0 + w1 + y - 1),
                    _ => None,
                }
            } else {
                None
            }


        } else {
            None
        }
    }
}

pub fn get_matrix_size(ui_ctrl: &UICtrlRef) -> (usize, usize) {
    let item_count = ui_ctrl.with_state(|s| s.menu_items.len());

    match item_count {
        0..=7   => (3, 3),
        8..=14  => (4, 4),
        15..=22 => (5, 5),
        23..=33 => (6, 6),
        _       => (7, 7),
    }
}

pub fn get_matrix_size_px(ui_ctrl: &UICtrlRef) -> (f64, f64) {
    let item_count = ui_ctrl.with_state(|s| s.menu_items.len());
    let (w, h) = get_matrix_size(ui_ctrl);
    let w = w as f64;
    let h = h as f64;

    let (w, h) =
        match item_count {
            0..=7   => (235.0, 240.0),
            8..=14  => (w * (78.0 + 9.0)       + 5.0, h * 78.0 + 5.0),
            15..=22 => (w * (78.0 + 2.0 * 9.0) + 5.0, h * 78.0 + 5.0),
            23..=33 => (w * (78.0 + 3.0 * 9.0) + 5.0, h * 78.0 + 5.0),
            _       => (w * (78.0 + 3.5 * 9.0) + 5.0, h * 78.0 + 5.0),
        };

    (w.floor(), h.floor())
}

impl HexGridModel for MatrixUIMenu {
    fn width(&self)  -> usize { get_matrix_size(&self.ui_ctrl).0 }
    fn height(&self) -> usize { get_matrix_size(&self.ui_ctrl).1 }

    fn cell_hover(&self, x: usize, y: usize) {
        if let Some(idx) = self.grid2index(x, y) {
            self.ui_ctrl.emit(Msg::menu_hover(idx));
        }
    }

    fn cell_click(&self, x: usize, y: usize, _btn: MButton, _modkey: bool) {
        if let Some(idx) = self.grid2index(x, y) {
            self.ui_ctrl.emit(Msg::menu_click(idx));
        }
    }

    fn cell_empty(&self, x: usize, y: usize) -> bool {
        if x >= self.width() || y >= self.height() { return true; }
        false
    }

    fn cell_visible(&self, x: usize, y: usize) -> bool {
        if x >= self.width() || y >= self.height() { return false; }
        if (x % 2 == 0) && y == 0 { return false; }
        true
    }

    fn cell_label<'a>(&self, x: usize, y: usize, buf: &'a mut [u8])
        -> Option<HexCell<'a>>
    {
        if x >= self.width() || y >= self.height() { return None; }
        let mut len = 0;

        let mut hlight = HexHLight::Plain;

        if let Some(idx) = self.grid2index(x, y) {
            self.ui_ctrl.with_state(|s| {
                if let Some(item) = s.menu_items.get(idx) {
                    len = buf.len().min(item.label.as_bytes().len());
                    buf[0..len].copy_from_slice(&item.label.as_bytes()[0..len]);
                }
            });

            if idx == 0 {
                hlight = HexHLight::Accent;
            }
        }

        if let Ok(s) = std::str::from_utf8(&buf[0..len]) {
            Some(HexCell {
                label:     s,
                hlight,
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
    w:      usize,
    h:      usize,
}

impl HexGridModel for MatrixUIModel {
    fn width(&self) -> usize { self.w }
    fn height(&self) -> usize { self.h }

    fn cell_click(&self, x: usize, y: usize, btn: MButton, modkey: bool) {
        self.ui_ctrl.emit(Msg::matrix_click(x, y, btn, modkey));
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
            if self.ui_ctrl.with_state(|s| s.is_cell_focussed(x, y)) {
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

    matrix_model: Rc<MatrixUIModel>,
    ui_ctrl:      UICtrlRef,
}

pub const HEX_MATRIX_ID         : u32 = 1;
pub const HEX_GRID_ID           : u32 = 2;
pub const HEX_MENU_CONT_ID      : u32 = 3;
pub const HEX_MENU_HELP_TEXT_ID : u32 = 4;
pub const HEX_MENU_ID           : u32 = 5;
pub const NODE_PANEL_ID         : u32 = 11;
pub const UTIL_PANEL_ID         : u32 = 12;
pub const HELP_TEXT_ID          : u32 = 13;
pub const HELP_TEXT_SHORTCUT_ID : u32 = 14;
pub const HELP_TEXT_ABOUT_ID    : u32 = 15;
pub const LOG_ID                : u32 = 16;

#[allow(clippy::new_ret_no_self)]
impl NodeMatrixData {
    pub fn new(
        ui_ctrl: UICtrlRef,
        pos: UIPos,
        node_id: u32)
    -> WidgetData
    {
        let wt_nmatrix  = Rc::new(NodeMatrix::new());

        let size   = ui_ctrl.with_matrix(|m| m.size());
        let txtsrc = ui_ctrl.with_state(|s| s.menu_help_text.clone());

        let menu_model =
            Rc::new(MatrixUIMenu::new(
                ui_ctrl.clone(),
                txtsrc.clone()));

        let matrix_model = Rc::new(MatrixUIModel {
            ui_ctrl:        ui_ctrl.clone(),
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
           .title_text(ui_ctrl.with_state(|s| s.menu_title.clone()))
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
Hex Grid / Node Matrix

The hex tile grid consists of so "cells", each cell can be
empty or contain a "node". A cell with a node has the following structure:

      _____________
     /    <i1>     \
    /<i2>  (L)  <o1>\      (L)           - Status Led
   /   <node type>   \     <i1> to <i3>  - 3 Input ports
   \  <instance id>  /     <o1> to <o3>  - 3 Output ports
    \<i3>       <o2>/      <node type>   - Type of the node
     \     <o3>    /       <instance id> - Instance ID of the node
      """""""""""""

The input ports correspond to the parameters of the node. You can assign
these to output ports of adjacent cells. A connection between cells
does only work or exist if there is an output port assigned and to the
adjacent cells edge a corresponding input port.

You can have multiple independent instances of a node without problems.
But you can also (linked) copy the instance of one cell to another with
a mouse gesture (LMB drag from empty non adjacent node). This means:

            ____
      _____/    \_____      Two (linked) copies of a "Sin" oscialltor
     / Sin \____/ Sin \     node with the same instance id (0).
     \  0  /    \  0  /     These are handles to the same oscillator node.
      """""\____/"""""

Linked copies make it possible to connect more than 3 inputs or output
of a node to other nodes.

            ____
      _____/    \_____      Two independent instances of a "Sin" oscillator.
     / Sin \____/ Sin \     One has the instance id 0 and the other 1.
     \  0  /    \  1  /     These are handles to different and independent
      """""\____/"""""      oscillator nodes.

Next page: Hex Grid Mouse Actions (Part 1)
---page---
Hex Grid Mouse Actions (Part 1)
The most basic actions are:

    LMB click on empty cell     - Open node selector menu.
    RMB click on any cell       - Open context menu for the cell.
    MMB drag grid               - Pan the grid around
    Scrollwheel Down/Up         - Zoom the grid in/out

Apart from these basics, there are multiple differnt mouse drag
gestures to change the node graph layout of the node matrix in the hex grid.

Some gestures do things with so called "node clusters". A "node cluster" is
a tree of connected nodes.

LMB (Left Mouse Button) Drag Actions:

    (LMB) Create two connected nodes with default ports:
      ..... <-- Drag Action
     _^_  .       ___       LMB Drag from empty to adjacent empty cell lets
    /   \_._  => /XXX\___   you select two new nodes from the node selector
    \___/ v \    \_0_/YYY\  menu. And connects these two nodes from their
        \___/        \_0_/  default output port to the default input port
                            of the other node.
                            (If you want to select the edges explicitly,
                             try dragging with RMB).

    (LMB) Create one new connected node with default ports:
      .....
     _^_  .       ___       LMB Drag from empty cell to adjacent node lets
    /   \_._  => /XXX\___   you select one new node from the node selector
    \___/YvY\    \_0_/YYY\  menu. And connects these two nodes from their
        \_0_/        \_0_/  default output ports to the default input port
                            of the other node.
                            (If you want to select the edges explicitly,
                             try dragging with RMB).

    (LMB) (Re)Connect Adjacent Cells:
       ......___            _____  Dragging a node to an adjacent node will
     __.__/ .   \     _____/     \ open the output/input port selection
    /  X  \_.___/ => / XXX \_____/ menu. You can connect two previously not
    \__0__/ vY  \    \__0_O/I Y  \ connected nodes with this or reconnect
          \__0__/          \__0__/ existing adjacent nodes.
---page---
Hex Grid Mouse Actions (Part 2)
LMB (Left Mouse Button) Drag Actions:

    (LMB) Move Cluster:
      .....      .........                    LMB drag from cell with a node
     _^_  .     _^_     _._      ___     ___  to any empty cell moves a
    /XXX\_._   /XXX\___/ v \    /   \___/XXX\ whole cluster of nodes. Keep
    \_1_/ v \  \_1_/   \___/ => \___/   \_1_/ in mind: a cluster is a tree
    /YYY\___/  /YYY\___/            \___/YYY\ of connected nodes. This will
    \_2_/      \_2_/                    \_2_/ not move adjacent but
                                              unconnected nodes!

    (LMB) Create Linked Copy close to destination:
      .........
     _^_     _._      ___     ___  LMB drag from cell with a node to any
    /XXX\___/ . \    /XXX\___/XXX\ other non adjacent cell with a
    \_0_/   \_._/ => \_0_/   \_0_/ node (YYY 1) creates a linked but
        \___/YvY\        \___/YvY\ unconnected copy of the dragged from
            \_1_/            \_1_/ node (XXX 0).
                                   (If you want to create a new instance
                                    instead, try dragging with RMB).

    (LMB) Create Linked Copy at empty drag source:
      .........
     _._     _._      ___     ___  LMB drag from an empty non adjacent
    /XvX\___/ . \    /XXX\___/   \ cell to a node will create a linked
    \_1_/   \_._/ => \_1_/   \___/ copy of that node.
        \___/ ^ \        \___/XXX\ (If you want to create a new instance
            \___/            \_1_/ instead, try dragging with RMB).

RMB (Right Mouse Button) Drag Actions:

    (RMB) Create two connected nodes with explicit port selection menu:
      ..... <-- Drag Action
     _^_  .       ___       RMB Drag from empty to adjacent empty cell lets
    /   \_._  => /XXX\___   you select two new nodes from the node selector
    \___/ v \    \_0_/YYY\  menu. After selecting the two nodes, you have
        \___/        \_0_/  to explicitly choose which ports to connect
                            (unless there is only one input or output port).
                            (If you want to use the default inputs/outputs
                             try dragging with LMB).
---page---
Hex Grid Mouse Actions (Part 2)
RMB (Right Mouse Button) Drag Actions:

    (RMB) Create one new connected node with explicit port selection menu:
      .....
     _^_  .       ___       RMB Drag from empty cell to adjacent node lets
    /   \_._  => /XXX\___   you select one new node from the node selector
    \___/YvY\    \_0_/YYY\  menu. And then requires you to explicitly
        \_0_/        \_0_/  select the input and output ports.
                            (If you want to use the default inputs/outputs
                             try dragging with LMB).

    (RMB) Move node:
      .....      .........
     _^_  .     _^_     _._      ___     ___  RMB drag from cell with a node
    /XXX\_._   /XXX\___/ v \    /   \___/XXX\ to any empty cell moves only
    \_1_/ v \  \_1_/   \___/ => \___/   \_1_/ the cell, ignoring any
    /YYY\___/  /YYY\___/        /YYY\___/   \ adjacent connected nodes.
    \_2_/      \_2_/            \_2_/   \___/

    (RMB) Create New Instance close to destination:
      .........
     _^_     _._      ___     ___  RMB drag from cell with a node to any
    /XXX\___/ . \    /XXX\___/XXX\ other non adjacent cell with a
    \_0_/   \_._/ => \_0_/   \_1_/ node (YYY 1) creates an unconnected new
        \___/YvY\        \___/YvY\ node instance with the same type as the
            \_1_/            \_1_/ drag source.
                                   (If you want to create a linked copy
                                    instead, try dragging with LMB).

    (RMB) Split a cluster
       .....
      _^_  .  ___       ___     ___  RMB drag between two connected nodes
     /XXX\_._/   \     /XXX\___/   \ will split the cluster (tree of
     \_1_/YvY\___/ =>  \_1_/   \___/ connected nodes) at that point and
     /   \_2_/   \     /   \___/YYY\ will make space for inserting a
     \___/   \___/     \___/   \_2_/ new node into that cluster.
---page---
Hex Grid Mouse Actions (Part 3)
RMB (Right Mouse Button) Drag Actions:

    (RMB) Create a New Instance at empty drag source:
      .........
     _._     _._      ___     ___  RMB drag from an empty non adjacent
    /XvX\___/ . \    /XXX\___/   \ cell to a node will create a new
    \_1_/   \_._/ => \_1_/   \___/ node instance of the type of the
        \___/ ^ \        \___/XXX\ drag destination node.
            \___/            \_2_/ (If you want to create a new instance
                                   instead, try dragging with RMB).
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
            "Node",
            wbox!(
                wt_help_txt,
                AtomId::new(node_id, HELP_TEXT_ID),
                center(12, 12),
                TextData::new(ui_ctrl.with_state(|s| s.help_text_src.clone()))));

        tdata.add(
            "Log",
            wbox!(
                wt_log_txt,
                AtomId::new(node_id, LOG_ID),
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
            let panel_pos = pos.resize(360.0, pos.h);
            let util_pos =
                pos.resize(355.0, pos.h - 5.0)
                   .offs(pos.w - 360.0, 0.0);

            let hex_pos = pos.shrink(365.0, 0.0);
            let show_help = data.ui_ctrl.with_state(|s| s.show_help);
            if !show_help {
                (*data.hex_grid).draw(ui, p, hex_pos);
            }

            (*data.node_panel).draw(ui, p, panel_pos);
            (*data.util_panel).draw(ui, p, util_pos);

            if show_help {
                (*data.help_text).draw(ui, p, hex_pos);
            }

            if data.ui_ctrl.with_state(|s| !s.menu_items.is_empty()) {
                let menu_pos = data.ui_ctrl.with_state(|s| s.menu_pos);
                let (w, h) = get_matrix_size_px(&data.ui_ctrl);
                let hex_w = w;
                let txt_w = 235.0;
                let menu_w = hex_w + txt_w;
                let menu_h = h + UI_ELEM_TXT_H + 2.0 * UI_BORDER_WIDTH;


                let menu_rect =
                    Rect::from(
                        menu_pos.0 - (hex_w * 0.5),
                        menu_pos.1 - menu_h * 0.5,
                        menu_w,
                        menu_h)
                    .move_into(&pos);

                let _hz = ui.hover_zone_for(data.hex_menu_id);
                //d// println!("HOVEER: {:?}", hz);

                (*data.hex_menu).draw(ui, p, menu_rect);
            }
        });
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        let menu_visible =
            data.with(|data: &mut NodeMatrixData| {
                data.ui_ctrl.with_state(|s| !s.menu_items.is_empty())
            }).unwrap_or(false);

        match ev {
            UIEvent::Click { id, x, y, button, .. } => {
                // println!("EV: {:?} id={}, btn={:?}, data.id={}",
                //          ev, *id, button, data.id());

                data.with(|data: &mut NodeMatrixData| {
                    let show_help = data.ui_ctrl.with_state(|s| s.show_help);

                    if show_help {
                        data.help_text.event(ui, ev);

                        if *id == data.hex_menu_id {
                            data.hex_menu.event(ui, ev);
                        } else {
                            data.node_panel.event(ui, ev);
                            data.util_panel.event(ui, ev);
                        }

                    } else if *id == data.hex_menu_id {
                        data.ui_ctrl.emit(
                            Msg::menu_mouse_click(
                                *x, *y, *button));
                        data.hex_menu.event(ui, ev);

                    } else if !menu_visible {
                        match button {
                            MButton::Right => {
                                if *id == data.hex_grid.id() {
                                    data.ui_ctrl.emit(
                                        Msg::matrix_mouse_click(
                                            *x, *y, *button));
                                    data.hex_grid.event(ui, ev);
                                } else {
                                    data.node_panel.event(ui, ev);
                                    data.util_panel.event(ui, ev);
                                }
                            },
                            _ => {
                                if *id == data.hex_grid.id() {
                                    data.ui_ctrl.emit(
                                        Msg::matrix_mouse_click(
                                            *x, *y, *button));
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
            UIEvent::FieldDrag { id, button, src, dst, mouse_pos, .. } => {
                data.with(|data: &mut NodeMatrixData| {
                    if !menu_visible && *id == data.hex_grid.id() {
                        data.matrix_model.ui_ctrl.emit(
                            Msg::cell_drag(*button, *src, *dst, *mouse_pos));
                    }
                });
                ui.queue_redraw();
            },
            UIEvent::Key { id, key, .. } => {
                use keyboard_types::Key;

                if *id == data.id() {
                    match key {
                        Key::F1 => {
                            data.with(|data: &mut NodeMatrixData| {
                                data.ui_ctrl.emit(Msg::key(key.clone()))
                            });
                        },
                        Key::Escape => {
                            data.with(|data: &mut NodeMatrixData| {
                                data.ui_ctrl.emit(Msg::key(key.clone()))
                            });
                        },
                        Key::F4 => {
                            data.with(|data: &mut NodeMatrixData| {
                                data.ui_ctrl.emit(Msg::key(key.clone()))
                            });
                        },
                        Key::Character(_) => {
                            data.with(|data: &mut NodeMatrixData| {
                                data.ui_ctrl.emit(Msg::key(key.clone()))
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
