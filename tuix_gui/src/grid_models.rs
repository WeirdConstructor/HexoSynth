// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::hexgrid::{HexGrid, HexGridModel, HexCell, HexDir, HexEdge, HexHLight};

use hexodsp::{Matrix, NodeId, Cell, CellDir, SAtom};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct MatrixUIModel {
    matrix:         Arc<Mutex<Matrix>>,
    w:              usize,
    h:              usize,
    node_colors:    HashMap<NodeId, u8>,
    focus:          (usize, usize),
}

impl MatrixUIModel {
    pub fn new(matrix: Arc<Mutex<Matrix>>) -> Self {
        let (w, h) = matrix.lock().expect("matrix is lockable").size();

        let mut s = Self {
            matrix,
            w,
            h,
            node_colors: HashMap::new(),
            focus:  (0, 0),
        };

        s.sync_from_matrix();

        s
    }

    pub fn set_focus_cell(&mut self, x: usize, y: usize) {
        self.focus = (x, y);
    }

    pub fn sync_to_matrix(&self) {
        let mut m = self.matrix.lock().expect("matrix lockable");

        let mut entries = vec![];

        for (k, v) in self.node_colors.iter() {
            entries.push(format!("{},{},{}",
                k.name(),
                k.instance(),
                v));
        }

        m.set_prop("node_colors", SAtom::str(&entries.join(";")));
    }

    pub fn sync_from_matrix(&mut self) {
        let mut m = self.matrix.lock().expect("matrix lockable");

        println!("SYNC FROM");
        if let Some(SAtom::Str(s)) = m.get_prop("node_colors") {
            println!("SYNC FROM {}", s);

            for entry in s.split(";") {
                let entry : Vec<&str> = entry.split(",").collect();

                let node_id = NodeId::from_str(entry[0]);
                let inst    = entry[1].parse::<usize>().unwrap_or(0);
                let node_id = node_id.to_instance(inst);
                let color   = entry[2].parse::<u8>().unwrap_or(0);

                self.node_colors.insert(node_id, color);
            }
        }
    }

    pub fn set_node_colors(&mut self, node_id: NodeId, color: u8) {
        self.node_colors.insert(node_id, color);
    }

    pub fn color_for_node(&self, node_id: NodeId) -> u8 {
        if let Some(clr) = self.node_colors.get(&node_id) {
            *clr
        } else {
            node_id.ui_category().default_color_idx()
        }
    }
}

impl HexGridModel for MatrixUIModel {
    fn width(&self) -> usize { self.w }
    fn height(&self) -> usize { self.h }

//    fn cell_click(&self, x: usize, y: usize, btn: MButton, modkey: bool) {
//        self.ui_ctrl.emit(Msg::matrix_click(x, y, btn, modkey));
//    }

    fn cell_empty(&self, x: usize, y: usize) -> bool {
        let mut m = self.matrix.lock().expect("matrix lockable");

        if let Some(cell) = m.get(x, y) {
            cell.node_id() == NodeId::Nop
        } else {
            true
        }
    }

    fn cell_visible(&self, x: usize, y: usize) -> bool {
        if x >= self.w || y >= self.h { return false; }
        true
    }

    fn cell_color(&self, x: usize, y: usize) -> u8 {
        if x >= self.w || y >= self.h { return 0; }

        let mut m = self.matrix.lock().expect("matrix lockable");

        let node_id : Option<NodeId> = m.get(x, y).map(|c| c.node_id());

        if let Some(node_id) = node_id {
            self.color_for_node(node_id)
        } else {
            0
        }
    }

    fn cell_label<'a>(&self, x: usize, y: usize, buf: &'a mut [u8])
        -> Option<HexCell<'a>>
    {
        if x >= self.w || y >= self.h { return None; }
        let (cell, led_value) = {
            let mut m = self.matrix.lock().expect("matrix lockable");

            let cell    = m.get_copy(x, y)?;
            let node_id = cell.node_id();
            let v       = m.filtered_led_for(&node_id);

            (cell, v)
        };

        let label = cell.label(buf)?;

        let hl =
            if self.focus == (x, y) { HexHLight::HLight }
            else                    { HexHLight::Normal };

        Some(HexCell {
            label,
            hlight: hl,
            rg_colors: Some(led_value)
        })
    }

    fn cell_edge<'a>(&self, x: usize, y: usize, edge: HexDir, buf: &'a mut [u8]) -> Option<(&'a str, HexEdge)> {
        let mut m = self.matrix.lock().expect("matrix lockable");

        let mut edge_lbl = None;
        let mut out_fb_info = None;

        if let Some(cell) = m.get(x, y) {
            let cell_dir = edge.into();

            if let Some((lbl, is_connected)) =
                m.edge_label(&cell, cell_dir, buf)
            {
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
                let val = m.filtered_out_fb_for(&node_id, out);

                Some((lbl, HexEdge::ArrowValue { value: val }))
            } else {
                Some((lbl, HexEdge::NoArrow))
            }
        } else {
            None
        }
    }
}

pub struct TestGridModel {
    last_click: (usize, usize),
    drag_to:    (usize, usize),
}

impl TestGridModel {
    pub fn new() -> Self {
        Self {
            last_click: (1000, 1000),
            drag_to: (1000, 1000),
        }
    }
}

impl HexGridModel for TestGridModel {
    fn width(&self) -> usize { 16 }
    fn height(&self) -> usize { 16 }
    fn cell_visible(&self, x: usize, y: usize) -> bool {
        x < self.width() && y < self.height()
    }
    fn cell_empty(&self, x: usize, y: usize) -> bool {
        !(x < self.width() && y < self.height())
    }
    fn cell_color(&self, x: usize, y: usize) -> u8 { 0 }
    fn cell_label<'a>(&self, x: usize, y: usize, out: &'a mut [u8])
        -> Option<HexCell<'a>>
    {
        let w = self.width();
        let h = self.height();
        if x >= w || y >= h { return None; }

        let mut hlight = HexHLight::Normal;

        use std::io::Write;
        let mut cur = std::io::Cursor::new(out);
        let len =
            if self.last_click == (x, y) {
                hlight = HexHLight::Select;
                match write!(cur, "CLICK") {
                    Ok(_)  => { cur.position() as usize },
                    Err(_) => 0,
                }
            } else if self.drag_to == (x, y) {
                hlight = HexHLight::HLight;
                match write!(cur, "DRAG") {
                    Ok(_)  => { cur.position() as usize },
                    Err(_) => 0,
                }
            } else {
                match write!(cur, "{}x{}", x, y) {
                    Ok(_)  => { cur.position() as usize },
                    Err(_) => 0,
                }
            };

        if len == 0 {
            return None;
        }

        Some(HexCell {
            label:
                std::str::from_utf8(&(cur.into_inner())[0..len])
                .unwrap(),
            hlight,
            rg_colors: Some(( 1.0, 1.0,)),
        })
    }

    /// Edge: 0 top-right, 1 bottom-right, 2 bottom, 3 bottom-left, 4 top-left, 5 top
    fn cell_edge<'a>(&self, x: usize, y: usize, edge: HexDir, out: &'a mut [u8])
        -> Option<(&'a str, HexEdge)>
    {
        let w = self.width();
        let h = self.height();
        if x >= w || y >= h { return None; }

        use std::io::Write;
        let mut cur = std::io::Cursor::new(out);
        match write!(cur, "{:?}", edge) {
            Ok(_)  => {
                let len = cur.position() as usize;
                Some((
                    std::str::from_utf8(&(cur.into_inner())[0..len])
                    .unwrap(),
                    HexEdge::ArrowValue { value: (1.0, 1.0) },
                ))
            },
            Err(_) => None,
        }
    }
//
//    fn cell_click(&mut self, x: usize, y: usize, btn: MButton) {
//        self.last_click = (x, y);
//        println!("CLICK! {:?} => {},{}", btn, x, y);
//    }
//
//    fn cell_drag(&mut self, x: usize, y: usize, x2: usize, y2: usize, btn: MButton) {
//        println!("DRAG! {:?} {},{} => {},{}", btn, x, y, x2, y2);
//        self.drag_to = (x2, y2);
//    }
}

