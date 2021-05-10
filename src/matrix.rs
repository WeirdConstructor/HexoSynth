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

struct NodeInstance {
    id:         NodeId,
    in_use:     bool,
    prog_idx:   usize,
    out_start:  usize,
    out_end:    usize,
    in_start:   usize,
    in_end:     usize,
    at_start:   usize,
    at_end:     usize,
}

impl NodeInstance {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            in_use:    false,
            prog_idx:  0,
            out_start: 0,
            out_end:   0,
            in_start:  0,
            in_end:    0,
            at_start:  0,
            at_end:    0,
        }
    }

    pub fn mark_used(&mut self) { self.in_use = true; }
    pub fn is_used(&self) -> bool { self.in_use }

    pub fn to_op(&self) -> NodeOp {
        NodeOp {
            idx:        self.prog_idx as u8,
            out_idxlen: (self.out_start, self.out_end),
            in_idxlen:  (self.in_start, self.in_end),
            at_idxlen:  (self.at_start, self.at_end),
            inputs:     vec![],
        }
    }

    pub fn in_local2global(&self, idx: u8) -> Option<usize> {
        let idx = self.in_start + idx as usize;
        if idx < self.in_end { Some(idx) }
        else { None }
    }

    pub fn out_local2global(&self, idx: u8) -> Option<usize> {
        let idx = self.out_start + idx as usize;
        if idx < self.out_end { Some(idx) }
        else { None }
    }

    pub fn set_index(mut self, idx: usize) -> Self {
        self.prog_idx = idx;
        self
    }

    pub fn set_output(mut self, s: usize, e: usize) -> Self {
        self.out_start = s;
        self.out_end   = e;
        self
    }

    pub fn set_input(mut self, s: usize, e: usize) -> Self {
        self.in_start = s;
        self.in_end   = e;
        self
    }

    pub fn set_atom(mut self, s: usize, e: usize) -> Self {
        self.at_start = s;
        self.at_end   = e;
        self
    }
}

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct MatrixNodeParam {
    param_id:   ParamId,
    input_idx:  usize,
    value:      f32,
}

#[derive(Debug, Clone)]
struct MatrixNodeAtom {
    param_id:   ParamId,
    at_idx:     usize,
    value:      SAtom,
}

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

    /// Bookkeeping of [NodeInstance] in the [NodeConfigurator].
    instances:   Rc<RefCell<std::collections::HashMap<NodeId, NodeInstance>>>,
    /// Bookkeeping for [Matrix::sync], to check if new instances appeared on the matrix.
    inst_diff:   Rc<RefCell<std::collections::HashMap<NodeId, bool>>>,
    /// Storing some runtime information about the instanciated node 
    infos:       Rc<RefCell<std::collections::HashMap<NodeId, NodeInfo>>>,

    /// Contains (automateable) parameters
    params:      Rc<RefCell<std::collections::HashMap<ParamId, MatrixNodeParam>>>,
    /// Stores the most recently set parameter values
    param_values:Rc<RefCell<std::collections::HashMap<ParamId, f32>>>,
    /// Contains non automateable atom data for the nodes
    atoms:       Rc<RefCell<std::collections::HashMap<ParamId, MatrixNodeAtom>>>,
    /// Stores the most recently set atoms
    atom_values: Rc<RefCell<std::collections::HashMap<ParamId, SAtom>>>,
}

unsafe impl Send for Matrix {}

impl Matrix {
    pub fn new(config: NodeConfigurator, w: usize, h: usize) -> Self {
        let mut matrix : Vec<Cell> = Vec::new();
        matrix.resize(w * h, Cell::empty(NodeId::Nop));

        Self {
            instances:   Rc::new(RefCell::new(std::collections::HashMap::new())),
            inst_diff:   Rc::new(RefCell::new(std::collections::HashMap::new())),
            infos:       Rc::new(RefCell::new(std::collections::HashMap::new())),
            params:      Rc::new(RefCell::new(std::collections::HashMap::new())),
            param_values:Rc::new(RefCell::new(std::collections::HashMap::new())),
            atoms:       Rc::new(RefCell::new(std::collections::HashMap::new())),
            atom_values: Rc::new(RefCell::new(std::collections::HashMap::new())),
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
        self.infos.borrow().get(&node_id).cloned()
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

        self.params      .borrow_mut().clear();
        self.param_values.borrow_mut().clear();
        self.atoms       .borrow_mut().clear();
        self.atom_values .borrow_mut().clear();
        self.instances   .borrow_mut().clear();
        self.inst_diff   .borrow_mut().clear();
        self.infos       .borrow_mut().clear();

        self.config.delete_nodes();
        self.monitor_cell(Cell::empty(NodeId::Nop));
        self.sync();
    }

    pub fn for_each_atom<F: FnMut(usize, ParamId, &SAtom)>(&self, mut f: F) {
        for (_, matrix_param) in self.atoms.borrow().iter() {
            if let Some(unique_idx) =
                self.config.unique_index_for(&matrix_param.param_id.node_id())
            {
                f(unique_idx, matrix_param.param_id, &matrix_param.value);
            }
        }

        for (_, matrix_param) in self.params.borrow().iter() {
            if let Some(unique_idx) =
                self.config.unique_index_for(&matrix_param.param_id.node_id())
            {
                f(unique_idx, matrix_param.param_id,
                  &SAtom::param(matrix_param.value));
            }
        }
    }

    pub fn get_generation(&self) -> usize { self.gen_counter }

    pub fn to_repr(&self) -> MatrixRepr {
        let params : Vec<(ParamId, f32)> =
            self.param_values.borrow()
                .iter()
                .map(|(param_id, value)| (*param_id, *value))
                .collect();
        let atoms : Vec<(ParamId, SAtom)> =
            self.atom_values.borrow()
                .iter()
                .map(|(param_id, value)| (*param_id, value.clone()))
                .collect();

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

        for (param_id, val) in repr.params.iter() {
            self.param_values.borrow_mut().insert(*param_id, *val);
        }

        for (param_id, val) in repr.atoms.iter() {
            self.atom_values.borrow_mut().insert(*param_id, val.clone());
        }

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
        use crate::nodes::UNUSED_MONITOR_IDX;
        let mut mon_idxes = [UNUSED_MONITOR_IDX; MON_SIG_CNT];

        self.monitored_cell = cell;

        let instances = self.instances.borrow();
        if let Some(ni) = instances.get(&cell.node_id) {
            if let Some(in1) = cell.in1 {
                mon_idxes[0] =
                    ni.in_local2global(in1)
                    .unwrap_or(UNUSED_MONITOR_IDX);
            }
            if let Some(in2) = cell.in2 {
                mon_idxes[1] =
                    ni.in_local2global(in2)
                    .unwrap_or(UNUSED_MONITOR_IDX);
            }
            if let Some(in3) = cell.in3 {
                mon_idxes[2] =
                    ni.in_local2global(in3)
                    .unwrap_or(UNUSED_MONITOR_IDX);
            }

            if let Some(out1) = cell.out1 {
                mon_idxes[3] =
                    ni.out_local2global(out1)
                    .unwrap_or(UNUSED_MONITOR_IDX);
            }
            if let Some(out2) = cell.out2 {
                mon_idxes[4] =
                    ni.out_local2global(out2)
                    .unwrap_or(UNUSED_MONITOR_IDX);
            }
            if let Some(out3) = cell.out3 {
                mon_idxes[5] =
                    ni.out_local2global(out3)
                    .unwrap_or(UNUSED_MONITOR_IDX);
            }
        }

        self.config.monitor(&mon_idxes);
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

    pub fn set_param(&mut self, param: ParamId, at: SAtom) {
        // XXX: The atoms and params maps are created when
        //      the matrix is sync()'ed. Only call this function
        //      if it was actually synced before!
        if param.is_atom() {
            self.atom_values.borrow_mut().insert(param, at.clone());

            if let Some(nparam) = self.atoms.borrow_mut().get_mut(&param) {
                nparam.value = at.clone();
                self.config.set_atom(nparam.at_idx, at);
            }
        } else {
            self.param_values.borrow_mut().insert(param, at.f());

            if let Some(nparam) = self.params.borrow_mut().get_mut(&param) {
                let value = at.f();
                nparam.value = value;
                self.config.set_param(nparam.input_idx, value);
            }
        }
    }

    pub fn get_adjacent_out_vec_index(&self, x: usize, y: usize, dir: CellDir)
        -> Option<usize>
    {
        if dir.is_output() {
            return None;
        }

        if let Some(cell) = self.get_adjacent(x, y, dir) {
            //d// println!("       ADJ CELL: {},{} ({})", x, y, cell.node_id());

            if cell.node_id != NodeId::Nop {
                //d// println!("GETADJ {},{} @ {:?} => {:?}", x, y, dir, cell);
                // check output 3
                // - get the associated output index
                // - get the NodeInstance of this cell
                // - add the assoc output index to the output-index
                //   of the node instance

                let instances = self.instances.borrow();
                match dir {
                    CellDir::T => {
                        if let Some(cell_out_i) = cell.out3 {
                            let ni = instances.get(&cell.node_id).unwrap();
                            ni.out_local2global(cell_out_i)
                        } else {
                            None
                        }
                    },
                    CellDir::TL => {
                        if let Some(cell_out_i) = cell.out2 {
                            let ni = instances.get(&cell.node_id).unwrap();
                            ni.out_local2global(cell_out_i)
                        } else {
                            None
                        }
                    },
                    CellDir::BL => {
                        if let Some(cell_out_i) = cell.out1 {
                            let ni = instances.get(&cell.node_id).unwrap();
                            ni.out_local2global(cell_out_i)
                        } else {
                            None
                        }
                    },
                    _ => { None }
                }
            } else {
                None
            }
        } else {
            None
        }
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

        let infos = self.infos.borrow();
        let info = infos.get(&cell.node_id)?;

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

    pub fn get_unused_instance_node_id(&self, mut id: NodeId) -> NodeId {
        id = id.to_instance(id.instance());

        let instances = self.instances.borrow();

        while let Some(ni) = instances.get(&id) {
            if !ni.is_used() {
                return ni.id;
            }

            id = id.to_instance(id.instance() + 1);
            //d// println!("NODECHECK {}", id);
        }

        id
    }

    pub fn sync(&mut self) {
        self.inst_diff.borrow_mut().clear();

        // Build instance map, to find new nodes in the matrix later.
        self.config.for_each(|node_info, mut id, _i| {
            while let Some(_) = self.inst_diff.borrow().get(&id) {
                id = id.to_instance(id.instance() + 1);
            }

            self.inst_diff.borrow_mut().insert(id, true);
            self.infos.borrow_mut().insert(id, node_info.clone());
        });

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
                if self.inst_diff.borrow().get(&cell.node_id).is_none() {
                    // - check if the previous node inst_diff exist, if not,
                    //   create them on the fly now:
                    for inst in 0..cell.node_id.instance() {
                        let new_hole_filler_node_id =
                            cell.node_id.to_instance(inst);

                        if self.inst_diff.borrow()
                            .get(&new_hole_filler_node_id)
                            .is_none()
                        {
                            let (info, _idx) =
                                self.config.create_node(new_hole_filler_node_id)
                                    .expect("NodeInfo existent in Matrix");
                            self.infos.borrow_mut()
                                .insert(new_hole_filler_node_id, info.clone());
                            self.inst_diff.borrow_mut().insert(
                                new_hole_filler_node_id,
                                true);
                        }
                    }

                    let (info, _idx) =
                        self.config.create_node(cell.node_id)
                            .expect("NodeInfo existent in Matrix");
                    self.infos.borrow_mut()
                        .insert(cell.node_id, info.clone());
                    self.instances.borrow_mut().insert(
                        cell.node_id,
                        NodeInstance::new(cell.node_id));
                }
            }
        }

        // Rebuild the instances map, because new ones might have been created.
        // And this time calculate the output offsets.
        self.instances.borrow_mut().clear();

        // Regenerating the params and atoms in the next step:
        self.params.borrow_mut().clear();
        self.atoms.borrow_mut().clear();

        let mut out_len = 0;
        let mut in_len  = 0;
        let mut at_len  = 0;
        self.config.for_each(|node_info, id, i| {
            // - calculate size of output vector.
            let out_idx = out_len;
            out_len += node_info.out_count();

            // - calculate size of input vector.
            let in_idx = in_len;
            in_len += node_info.in_count();

            // - calculate size of atom vector.
            let at_idx = at_len;
            at_len += node_info.at_count();

            println!("{} = {}", i, id);

            // Create new parameters and initialize them if they did not
            // already exist from a previous matrix instance.
            for param_idx in in_idx..in_len {
                if let Some(param_id) = id.inp_param_by_idx(param_idx - in_idx) {
                    let value =
                        if let Some(value) = self.param_values.borrow().get(&param_id) {
                            *value
                        } else {
                            param_id.norm_def()
                        };

                    self.param_values.borrow_mut().insert(param_id, value);
                    self.params.borrow_mut().insert(param_id, MatrixNodeParam {
                        param_id,
                        value,
                        input_idx: param_idx,
                    });
                }
            }

            // Create new atom data and initialize it if it did not
            // already exist from a previous matrix instance.
            for atom_idx in at_idx..at_len {
                // XXX: See also the documentation of atom_param_by_idx about the
                // little param_id for an Atom weirdness here.
                if let Some(param_id) = id.atom_param_by_idx(atom_idx - at_idx) {
                    let value =
                        if let Some(atom) =
                            self.atom_values.borrow().get(&param_id)
                        {
                            atom.clone()
                        } else {
                            param_id.as_atom_def()
                        };

                    self.atom_values.borrow_mut().insert(param_id, value.clone());
                    self.atoms.borrow_mut().insert(param_id, MatrixNodeAtom {
                        param_id,
                        value,
                        at_idx:  atom_idx,
                    });
                }
            }

            println!("INSERT: {:?} outidx: {},{} inidx: {},{} atidx: {},{}",
                     id, out_idx, out_len, in_idx, in_len, at_idx, at_len);

            // - save offset and length of each node's
            //   allocation in the output vector.
            self.instances.borrow_mut().insert(id,
                NodeInstance::new(id)
                .set_index(i)
                .set_output(out_idx, out_len)
                .set_input(in_idx, in_len)
                .set_atom(at_idx, at_len));
        });

        let mut prog = NodeProg::new(out_len, in_len, at_len);

        for x in 0..self.w {
            for y in 0..self.h {
                let cell = self.matrix[x * self.h + y];
                if cell.node_id == NodeId::Nop {
                    continue;
                }

                println!("GET INPUT OUTIDXES for {} @ {},{}", cell.node_id, x, y);

                // Get the indices to the output vector for the
                // corresponding input ports.
                let in_1_out_idx = self.get_adjacent_out_vec_index(x, y, CellDir::T);
                let in_2_out_idx = self.get_adjacent_out_vec_index(x, y, CellDir::TL);
                let in_3_out_idx = self.get_adjacent_out_vec_index(x, y, CellDir::BL);

                println!("*** In1 OIdx: ({}) {:?}", cell.node_id, in_1_out_idx);
                println!("*** In2 OIdx: ({}) {:?}", cell.node_id, in_2_out_idx);
                println!("*** In3 OIdx: ({}) {:?}", cell.node_id, in_3_out_idx);

                let mut instances = self.instances.borrow_mut();
                let ni = instances.get_mut(&cell.node_id).unwrap();
                ni.mark_used();
                let op = ni.to_op();

                let in_1 =
                    if let Some(in1_idx) = cell.in1 {
                        if let Some(in1_out_idx) = in_1_out_idx {
                            if let Some(in1_global_idx) =
                                ni.in_local2global(in1_idx)
                            {
                                Some((in1_out_idx, in1_global_idx))
                            } else { None }
                        } else { None }
                    } else { None };

                let in_2 =
                    if let Some(in2_idx) = cell.in2 {
                        if let Some(in2_out_idx) = in_2_out_idx {
                            if let Some(in2_global_idx) =
                                ni.in_local2global(in2_idx)
                            {
                                Some((in2_out_idx, in2_global_idx))
                            } else { None }
                        } else { None }
                    } else { None };

                let in_3 =
                    if let Some(in3_idx) = cell.in3 {
                        if let Some(in3_out_idx) = in_3_out_idx {
                            if let Some(in3_global_idx) =
                                ni.in_local2global(in3_idx)
                            {
                                Some((in3_out_idx, in3_global_idx))
                            } else { None }
                        } else { None }
                    } else { None };

                prog.append_with_inputs(op, in_1, in_2, in_3);
            }
        }

        // Copy the parameter values and atom data into the program:
        // They are extracted by process_graph_updates() later to
        // reset the inp[] input value vector.
        for (_param_id, param) in self.params.borrow().iter() {
            prog.params_mut()[param.input_idx] = param.value;
        }

        // The atoms are referred to directly on process() call.
        for (_param_id, param) in self.atoms.borrow().iter() {
            prog.atoms_mut()[param.at_idx] = param.value.clone();
        }

        self.config.upload_prog(prog, true); // true => copy_old_out

        self.gen_counter += 1;

        // Refresh the input/outputs of the monitored cell, just in case.
        self.remonitor_cell();
    }
}


/*

Design of the highlevel Matrix API:

- NodeInfo (belongs to nothing, is the root of knowledge)
  - name
  - GUI type (Default, ModFunction, LFO+MF, 3xLFO+MF, ADSR+MF, ...)
  - output ports: number and name
  - input ports: number and name
    - input parameter range
    - input parameter normalization/denormalization
    - input parameter formatting

- NodeCollection (changes are transmitted to the backend!)
    - List all possible node types (NodeInfo) "Sin", "Amp", "Out"
    - List existing instances "Sin 1", "Sin 2", ... with their NodeInfo
        => NodeInstance
    - Instanciate new nodes (they get a global identifier)
    - Update an input parameter by Instance ID and input index.

- Matrix (has a NodeCollection)
    (changes are transmitted to the backend)
    - place instanciated nodes somewhere with an input/output configuration
      (=> Define a Cell, which comes with 3 in and 3 out indices)
    - clear a cell of the matrix
    - get a cell of the matrix
    - make a selection of cells
    - copy that selection
    - paste a selection to somewhere else

- Query Node instance state InstanceState:
    - frontend parameter values (knobs)
    - output state
      - the backend should just provide a triple buffer with this
        and the NodeCollection somehow makes the output ports
        accessible by instance

- Cells (belong to Matrix)
    - Come with an instance ID
    - Get the instance name
    - Get the name of the assigned output and input ports
    - Flag if the cell is selected
    - Assign new edge input/outputs


What the GUI needs:

- ?

I still need to decide how to refer to node instances:

- by global unique ID => how to recreate these IDs from a saved repr?
- By NodeType + Index
  - More generic in my gut feeling
  - Problem: NodeCollection needs to be able to check
             which internal index this node resides in.
             For this a linear scan over all nodes is necessary.
             But there are only ~100 nodes, so this should not
             take too much time!
  - Invariant: Don't delete nodes. Only delete them on a manual user
               initiated "garbage collect" which renames the nodes in the matrix.


*/

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
