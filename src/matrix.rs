// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use crate::nodes::{NodeOp, NodeConfigurator, NodeProg};
use crate::dsp::{NodeInfo, NodeId, ParamId, SAtom};
pub use crate::CellDir;
pub use crate::nodes::MinMaxMonitorSamples;
pub use crate::monitor::MON_SIG_CNT;
use crate::matrix_repr::*;
use crate::dsp::tracker::PatternData;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    node_id:  NodeId,
    x:        u8,
    y:        u8,
    /// Top-Right output
    out1:     Option<u8>,
    /// Bottom-Right output
    out2:     Option<u8>,
    /// Bottom output
    out3:     Option<u8>,
    /// Top input
    in1:      Option<u8>,
    /// Top-Left input
    in2:      Option<u8>,
    /// Bottom-Left input
    in3:      Option<u8>,
}

impl Cell {
    pub fn empty(node_id: NodeId) -> Self {
        Self {
            node_id,
            x: 0,
            y: 0,
            out1: None,
            out2: None,
            out3: None,
            in1: None,
            in2: None,
            in3: None,
        }
    }

    pub fn to_repr(&self) -> CellRepr {
        CellRepr {
            node_id: self.node_id,
            x: self.x as usize,
            y: self.y as usize,
            out: [
                self.out1.map(|v| v as i16).unwrap_or(-1),
                self.out2.map(|v| v as i16).unwrap_or(-1),
                self.out3.map(|v| v as i16).unwrap_or(-1)
            ],
            inp: [
                self.in1.map(|v| v as i16).unwrap_or(-1),
                self.in2.map(|v| v as i16).unwrap_or(-1),
                self.in3.map(|v| v as i16).unwrap_or(-1)
            ],
        }
    }

    pub fn from_repr(repr: &CellRepr) -> Self {
        Self {
            node_id: repr.node_id,
            x:       repr.x as u8,
            y:       repr.y as u8,
            out1:    if repr.out[0] < 0 { None }
                     else { Some(repr.out[0] as u8) },
            out2:    if repr.out[1] < 0 { None }
                     else { Some(repr.out[1] as u8) },
            out3:    if repr.out[2] < 0 { None }
                     else { Some(repr.out[2] as u8) },
            in1:     if repr.inp[0] < 0 { None }
                     else { Some(repr.inp[0] as u8) },
            in2:     if repr.inp[1] < 0 { None }
                     else { Some(repr.inp[1] as u8) },
            in3:     if repr.inp[2] < 0 { None }
                     else { Some(repr.inp[2] as u8) },
        }
    }

    pub fn with_pos_of(&self, other: Cell) -> Self {
       let mut new = *self;
       new.x = other.x;
       new.y = other.y;
       new
    }

    pub fn is_empty(&self) -> bool { self.node_id == NodeId::Nop }

    pub fn node_id(&self) -> NodeId { self.node_id }

    pub fn set_node_id(&mut self, new_id: NodeId) {
        self.node_id = new_id;
    }

    pub fn label<'a>(&self, buf: &'a mut [u8]) -> Option<&'a str> {
        use std::io::Write;
        let mut cur = std::io::Cursor::new(buf);

        if self.node_id == NodeId::Nop {
            return None;
        }

//        let node_info = infoh.from_node_id(self.node_id);

        match write!(cur, "{}", self.node_id) {
            Ok(_)  => {
                let len = cur.position() as usize;
                Some(
                    std::str::from_utf8(&(cur.into_inner())[0..len])
                    .unwrap())
            },
            Err(_) => None,
        }
    }

    pub fn pos(&self) -> (usize, usize) {
        (self.x as usize, self.y as usize)
    }

    pub fn has_dir_set(&self, dir: CellDir) -> bool {
        match dir {
            CellDir::TR => self.out1.is_some(),
            CellDir::BR => self.out2.is_some(),
            CellDir::B  => self.out3.is_some(),
            CellDir::BL => self.in3.is_some(),
            CellDir::TL => self.in2.is_some(),
            CellDir::T  => self.in1.is_some(),
            CellDir::C  => false,
        }
    }

    pub fn clear_io_dir(&mut self, dir: CellDir) {
        match dir {
            CellDir::TR => { self.out1 = None; },
            CellDir::BR => { self.out2 = None; },
            CellDir::B  => { self.out3 = None; },
            CellDir::BL => { self.in3  = None; },
            CellDir::TL => { self.in2  = None; },
            CellDir::T  => { self.in1  = None; },
            CellDir::C  => {},
        }
    }

    pub fn set_io_dir(&mut self, dir: CellDir, idx: usize) {
        match dir {
            CellDir::TR => { self.out1 = Some(idx as u8); },
            CellDir::BR => { self.out2 = Some(idx as u8); },
            CellDir::B  => { self.out3 = Some(idx as u8); },
            CellDir::BL => { self.in3  = Some(idx as u8); },
            CellDir::TL => { self.in2  = Some(idx as u8); },
            CellDir::T  => { self.in1  = Some(idx as u8); },
            CellDir::C  => {},
        }
    }

    pub fn input(mut self, i1: Option<u8>, i2: Option<u8>, i3: Option<u8>) -> Self {
        self.in1 = i1;
        self.in2 = i2;
        self.in3 = i3;
        self
    }

    pub fn out(mut self, o1: Option<u8>, o2: Option<u8>, o3: Option<u8>) -> Self {
        self.out1 = o1;
        self.out2 = o2;
        self.out3 = o3;
        self
    }
}

use std::rc::Rc;
use std::cell::RefCell;

pub struct Matrix {
    config:      NodeConfigurator,
    matrix:      Vec<Cell>,
    w:           usize,
    h:           usize,

    /// Holds the currently monitored cell.
    monitored_cell: Cell,

    /// A counter that increases for each sync(), it can be used
    /// by other components of the application to detect changes in
    /// the matrix to resync their own data.
    gen_counter: usize,
}

unsafe impl Send for Matrix {}

impl Matrix {
    pub fn new(config: NodeConfigurator, w: usize, h: usize) -> Self {
        let mut matrix : Vec<Cell> = Vec::new();
        matrix.resize(w * h, Cell::empty(NodeId::Nop));

        Self {
            monitored_cell: Cell::empty(NodeId::Nop),
            gen_counter: 0,
            config,
            w,
            h,
            matrix,
        }
    }

    pub fn size(&self) -> (usize, usize) { (self.w, self.h) }

    pub fn unique_index_for(&self, node_id: &NodeId) -> Option<usize> {
        self.config.unique_index_for(node_id)
    }

    pub fn info_for(&self, node_id: &NodeId) -> Option<NodeInfo> {
        self.config.node_by_id(&node_id)?.0.cloned()
    }

    pub fn phase_value_for(&self, node_id: &NodeId) -> f32 {
        self.config.phase_value_for(node_id)
    }

    pub fn led_value_for(&self, node_id: &NodeId) -> f32 {
        self.config.led_value_for(node_id)
    }

    pub fn get_pattern_data(&self, tracker_id: usize)
        -> Option<Rc<RefCell<PatternData>>>
    {
        self.config.get_pattern_data(tracker_id)
    }

    pub fn check_pattern_data(&mut self, tracker_id: usize) {
        self.config.check_pattern_data(tracker_id)
    }

    pub fn place(&mut self, x: usize, y: usize, mut cell: Cell) {
        cell.x = x as u8;
        cell.y = y as u8;
        self.matrix[x * self.h + y] = cell;
    }

    pub fn clear(&mut self) {
        for cell in self.matrix.iter_mut() {
            *cell = Cell::empty(NodeId::Nop);
        }

        self.config.delete_nodes();
        self.monitor_cell(Cell::empty(NodeId::Nop));
        self.sync();
    }

    pub fn for_each_atom<F: FnMut(usize, ParamId, &SAtom)>(&self, mut f: F) {
        self.config.for_each_param(f);
    }

    pub fn get_generation(&self) -> usize { self.gen_counter }

    pub fn to_repr(&self) -> MatrixRepr {
        let (params, atoms) = self.config.dump_param_values();

        let mut cells : Vec<CellRepr> = vec![];
        self.for_each(|_x, _y, cell|
            if cell.node_id() != NodeId::Nop {
                cells.push(cell.to_repr())
            });

        let mut patterns : Vec<Option<PatternRepr>> = vec![];
        let mut tracker_id = 0;
        while let Some(pdata) = self.get_pattern_data(tracker_id) {
            patterns.push(
                if pdata.borrow().is_unset() { None }
                else { Some(pdata.borrow().to_repr()) });

            tracker_id += 1;
        }

        MatrixRepr {
            cells,
            params,
            atoms,
            patterns,
        }
    }

    pub fn from_repr(&mut self, repr: &MatrixRepr) {
        self.clear();

        self.config.load_dumped_param_values(
            &repr.params[..],
            &repr.atoms[..]);

        for cell_repr in repr.cells.iter() {
            let cell = Cell::from_repr(cell_repr);
            self.place(cell.x as usize, cell.y as usize, cell);
        }

        for (tracker_id, pat) in repr.patterns.iter().enumerate() {
            if let Some(pat) = pat {
                if let Some(pd) = self.get_pattern_data(tracker_id) {
                    pd.borrow_mut().from_repr(pat);
                }
            }
        }

        self.sync();
    }

    /// Receives the most recent data for the monitored signal at index `idx`.
    /// Might introduce a short wait, because internally a mutex is still locked.
    /// If this leads to stuttering in the UI, we need to change the internal
    /// handling to a triple buffer.
    pub fn get_minmax_monitor_samples(&mut self, idx: usize) -> &MinMaxMonitorSamples {
        self.config.get_minmax_monitor_samples(idx)
    }

    /// Returns the currently monitored cell.
    pub fn monitored_cell(&self) -> &Cell { &self.monitored_cell }

    /// Sets the cell to monitor next. Please bear in mind, that you need to
    /// call `sync` before retrieving the cell from the matrix, otherwise
    /// the node instance might not have been created in the backend yet and
    /// we can not start monitoring the cell.
    pub fn monitor_cell(&mut self, cell: Cell) {
        self.monitored_cell = cell;

        let inputs  = [
            cell.in1,
            cell.in2,
            cell.in3,
        ];
        let outputs = [
            cell.out1,
            cell.out2,
            cell.out3,
        ];

        self.config.monitor(&cell.node_id, &inputs, &outputs);
    }

    /// Is called by [Matrix::sync] to refresh the monitored cell.
    /// In case the matrix has changed (inputs/outputs of a cell)
    /// we show the current state.
    ///
    /// Note, that if the UI actually moved a cell, it needs to
    /// monitor the newly moved cell anyways.
    fn remonitor_cell(&mut self) {
        let m = self.monitored_cell();
        if let Some(cell) = self.get(m.x as usize, m.y as usize).copied() {
            self.monitor_cell(cell);
        }
    }

    /// Assign [SAtom] values to input parameters and atoms.
    pub fn set_param(&mut self, param: ParamId, at: SAtom) {
        self.config.set_param(param, at);
    }

    pub fn get_adjacent_output(&self, x: usize, y: usize, dir: CellDir)
        -> Option<(NodeId, u8)>
    {
        if dir.is_output() {
            return None;
        }

        let cell = self.get_adjacent(x, y, dir)?;

        if cell.node_id == NodeId::Nop {
            return None;
        }

        let cell_out =
            match dir {
                CellDir::T  => cell.out3?,
                CellDir::TL => cell.out2?,
                CellDir::BL => cell.out1?,
                _           => { return None; }
            };

        Some((cell.node_id, cell_out))
    }

    pub fn get_adjacent(&self, x: usize, y: usize, dir: CellDir) -> Option<&Cell> {
        let offs : (i32, i32) = dir.to_offs(x);
        let x = x as i32 + offs.0;
        let y = y as i32 + offs.1;

        if x < 0 || y < 0 || (x as usize) >= self.w || (y as usize) >= self.h {
            return None;
        }

        Some(&self.matrix[(x as usize) * self.h + (y as usize)])
    }

    pub fn adjacent_edge_has_input(&self, x: usize, y: usize, edge: CellDir) -> bool {
        if let Some(cell) = self.get_adjacent(x, y, edge) {
            //d// println!("       ADJ CELL: {},{} ({})", cell.x, cell.y, cell.node_id());
            match edge {
                CellDir::TR => cell.in3.is_some(),
                CellDir::BR => cell.in2.is_some(),
                CellDir::B  => cell.in1.is_some(),
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn for_each<F: FnMut(usize, usize, &Cell)>(&self, mut f: F) {
        for x in 0..self.w {
            for y in 0..self.h {
                let cell = &self.matrix[x * self.h + y];
                f(x, y, cell);
            }
        }
    }

    pub fn edge_label<'a>(&self, cell: &Cell, edge: CellDir, buf: &'a mut [u8]) -> Option<(&'a str, bool)> {
        use std::io::Write;
        let mut cur = std::io::Cursor::new(buf);

        if cell.node_id == NodeId::Nop {
            return None;
        }

        let out_idx =
            match edge {
                CellDir::TR => Some(cell.out1),
                CellDir::BR => Some(cell.out2),
                CellDir::B  => Some(cell.out3),
                _ => None,
            };
        let in_idx =
            match edge {
                CellDir::BL => Some(cell.in3),
                CellDir::TL => Some(cell.in2),
                CellDir::T  => Some(cell.in1),
                _ => None,
            };

        let info = self.info_for(&cell.node_id)?;

        let mut is_connected_edge = false;

        let edge_str =
            if let Some(out_idx) = out_idx {
                //d// println!("    CHECK ADJ EDGE {},{} @ {:?}", cell.x, cell.y, edge);
                is_connected_edge =
                    self.adjacent_edge_has_input(
                        cell.x as usize, cell.y as usize, edge);

                info.out_name(out_idx? as usize)

            } else if let Some(in_idx) = in_idx {
                info.in_name(in_idx? as usize)

            } else {
                None
            };

        let edge_str = edge_str?;

        match write!(cur, "{}", edge_str) {
            Ok(_)  => {
                let len = cur.position() as usize;
                Some((
                    std::str::from_utf8(&(cur.into_inner())[0..len])
                    .unwrap(),
                    is_connected_edge))
            },
            Err(_) => None,
        }
    }

    pub fn get_copy(&self, x: usize, y: usize) -> Option<Cell> {
        if x >= self.w || y >= self.h {
            return None;
        }

        let mut cell = self.matrix[x * self.h + y];
        cell.x = x as u8;
        cell.y = y as u8;
        Some(cell)
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        if x >= self.w || y >= self.h {
            return None;
        }

        Some(&self.matrix[x * self.h + y])
    }

    pub fn get_unused_instance_node_id(&self, id: NodeId) -> NodeId {
        self.config.unused_instance_node_id(id)
    }

    pub fn sync(&mut self) {
        // Scan through the matrix and check if (backend) nodes need to be created
        // for new unknown nodes:
        for x in 0..self.w {
            for y in 0..self.h {
                let cell = &mut self.matrix[x * self.h + y];

                if cell.node_id == NodeId::Nop {
                    continue;
                }

                // - check if each NodeId has a corresponding entry in NodeConfigurator
                //   - if not, create a new one on the fly
                if self.config.unique_index_for(&cell.node_id).is_none() {
                    // - check if the previous node exist, if not,
                    //   create them on the fly now:
                    for inst in 0..cell.node_id.instance() {
                        let new_hole_filler_node_id =
                            cell.node_id.to_instance(inst);

                        if self.config
                            .unique_index_for(&new_hole_filler_node_id)
                            .is_none()
                        {
                            let (info, _idx) =
                                self.config.create_node(new_hole_filler_node_id)
                                    .expect("NodeInfo existent in Matrix");
                        }
                    }

                    let (info, _idx) =
                        self.config.create_node(cell.node_id)
                            .expect("NodeInfo existent in Matrix");
                }
            }
        }

        self.config.rebuild_node_ports();

        // Create the node program and set the execution order of the
        // nodes and their corresponding inputs/outputs.
        let mut prog = NodeProg::new(out_len, in_len, at_len);

        for x in 0..self.w {
            for y in 0..self.h {
                let cell = self.matrix[x * self.h + y];
                if cell.node_id == NodeId::Nop {
                    continue;
                }

                let in1_output = self.get_adjacent_output(x, y, CellDir::T);
                let in2_output = self.get_adjacent_output(x, y, CellDir::TL);
                let in3_output = self.get_adjacent_output(x, y, CellDir::BL);

                match (cell.in1, in1_output) {
                    (Some(in1_idx), Some(in1_output)) => {
                        self.config.set_prog_node_exec_connection(
                            &mut prog, (cell.node_id, in1_idx), in1_output);
                    },
                    _ => {},
                }

                match (cell.in2, in2_output) {
                    (Some(in2_idx), Some(in2_output)) => {
                        self.config.set_prog_node_exec_connection(
                            &mut prog, (cell.node_id, in2_idx), in2_output);
                    },
                    _ => {},
                }

                match (cell.in3, in3_output) {
                    (Some(in3_idx), Some(in3_output)) => {
                        self.config.set_prog_node_exec_connection(
                            &mut prog, (cell.node_id, in3_idx), in3_output);
                    },
                    _ => {},
                }
            }
        }

        self.config.upload_prog(prog, true); // true => copy_old_out

        self.gen_counter += 1;

        // Refresh the input/outputs of the monitored cell, just in case.
        self.remonitor_cell();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_matrix_3_sine() {
        use crate::nodes::new_node_engine;

        let (node_conf, mut node_exec) = new_node_engine();
        let mut matrix = Matrix::new(node_conf, 3, 3);

        matrix.place(0, 0,
            Cell::empty(NodeId::Sin(0))
            .out(None, Some(0), None));
        matrix.place(1, 0,
            Cell::empty(NodeId::Sin(1))
            .input(None, Some(0), None)
            .out(None, None, Some(0)));
        matrix.place(1, 1,
            Cell::empty(NodeId::Sin(2))
            .input(Some(0), None, None));
        matrix.sync();

        node_exec.process_graph_updates();

        let nodes = node_exec.get_nodes();
        assert!(nodes[0].to_id(0) == NodeId::Sin(0));
        assert!(nodes[1].to_id(1) == NodeId::Sin(1));
        assert!(nodes[2].to_id(2) == NodeId::Sin(2));

        let prog = node_exec.get_prog();
        assert_eq!(prog.prog[0].to_string(), "Op(i=0 out=(0-1) in=(0-1) at=(0-0))");
        assert_eq!(prog.prog[1].to_string(), "Op(i=1 out=(1-2) in=(1-2) at=(0-0) cpy=(o0 => i1))");
        assert_eq!(prog.prog[2].to_string(), "Op(i=2 out=(2-3) in=(2-3) at=(0-0) cpy=(o1 => i2))");
    }

    #[test]
    fn check_matrix_filled() {
        use crate::nodes::new_node_engine;
        use crate::dsp::{NodeId, Node};

        let (node_conf, mut node_exec) = new_node_engine();
        let mut matrix = Matrix::new(node_conf, 9, 9);

        let mut i = 1;
        for x in 0..9 {
            for y in 0..9 {
                matrix.place(x, y, Cell::empty(NodeId::Sin(i)));
                i += 1;
            }
        }
        matrix.sync();

        node_exec.process_graph_updates();

        let nodes = node_exec.get_nodes();
        let ex_nodes : Vec<&Node> =
            nodes.iter().filter(|n| n.to_id(0) != NodeId::Nop).collect();
        assert_eq!(ex_nodes.len(), 9 * 9 + 1);
    }

    #[test]
    fn check_matrix_into_output() {
        use crate::nodes::new_node_engine;

        let (node_conf, mut node_exec) = new_node_engine();
        let mut matrix = Matrix::new(node_conf, 3, 3);

        matrix.place(0, 0,
            Cell::empty(NodeId::Sin(0))
            .out(None, Some(0), None));
        matrix.place(1, 0,
            Cell::empty(NodeId::Out(0))
            .input(None, Some(0), None)
            .out(None, None, Some(0)));
        matrix.sync();

        node_exec.set_sample_rate(44100.0);
        node_exec.process_graph_updates();

        let nodes = node_exec.get_nodes();
        assert!(nodes[0].to_id(0) == NodeId::Sin(0));
        assert!(nodes[1].to_id(0) == NodeId::Out(0));

        let prog = node_exec.get_prog();
        assert_eq!(prog.prog.len(), 2);
        assert_eq!(prog.prog[0].to_string(), "Op(i=0 out=(0-1) in=(0-1) at=(0-0))");
        assert_eq!(prog.prog[1].to_string(), "Op(i=1 out=(1-1) in=(1-3) at=(0-1) cpy=(o0 => i1))");
    }

    #[test]
    fn check_matrix_skip_instance() {
        use crate::nodes::new_node_engine;

        let (node_conf, mut node_exec) = new_node_engine();
        let mut matrix = Matrix::new(node_conf, 3, 3);

        matrix.place(0, 0,
            Cell::empty(NodeId::Sin(2))
            .out(None, Some(0), None));
        matrix.place(1, 0,
            Cell::empty(NodeId::Out(0))
            .input(None, Some(0), None)
            .out(None, None, Some(0)));
        matrix.sync();

        node_exec.set_sample_rate(44100.0);
        node_exec.process_graph_updates();

        let nodes = node_exec.get_nodes();
        assert!(nodes[0].to_id(0) == NodeId::Sin(0));
        assert!(nodes[1].to_id(0) == NodeId::Sin(0));
        assert!(nodes[2].to_id(0) == NodeId::Sin(0));
        assert!(nodes[3].to_id(0) == NodeId::Out(0));

        let prog = node_exec.get_prog();
        assert_eq!(prog.prog.len(), 2);
        assert_eq!(prog.prog[0].to_string(), "Op(i=2 out=(2-3) in=(2-3) at=(0-0))");
        assert_eq!(prog.prog[1].to_string(), "Op(i=3 out=(3-3) in=(3-5) at=(0-1) cpy=(o2 => i3))");
    }
}
