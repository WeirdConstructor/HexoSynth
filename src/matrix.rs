// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use crate::nodes::{
    NodeConfigurator,
    NodeGraphOrdering,
    NodeProg,
    MAX_ALLOCATED_NODES
};
use crate::dsp::{NodeInfo, NodeId, ParamId, SAtom};
pub use crate::CellDir;
pub use crate::nodes::MinMaxMonitorSamples;
pub use crate::monitor::MON_SIG_CNT;
use crate::matrix_repr::*;
use crate::dsp::tracker::PatternData;

use triple_buffer::Output;

/// This is a cell/tile of the hexagonal [Matrix].
///
/// The [Matrix] stores it to keep track of the graphical representation
/// of the hexagonal tilemap. Using [Matrix::place] you can place new cells.
///
///```
/// use hexosynth::*;
///
/// let (node_conf, mut node_exec) = new_node_engine();
/// let mut matrix = Matrix::new(node_conf, 3, 3);
///
/// matrix.place(
///     2, 2,
///     Cell::empty(NodeId::Sin(0))
///     .input(Some(0), None, None)
///     .out(None, None, Some(0)));
///
/// matrix.sync().unwrap();
///```
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
    /// This is the main contructor of a [Cell].
    /// Empty means that there is no associated position of this cell
    /// and no inputs/outputs have been assigned. Use the methods [Cell::input] and [Cell::out]
    /// to assign inputs / outputs.
    ///
    ///```
    /// use hexosynth::*;
    ///
    /// let some_cell =
    ///     Cell::empty(NodeId::Sin(0))
    ///     .input(None, Some(0), Some(0))
    ///     .out(None, Some(0), Some(0));
    ///```
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

    /// Returns a serializable representation of this [Matrix] [Cell].
    ///
    /// See also [CellRepr].
    ///
    ///```
    /// use hexosynth::*;
    ///
    /// let some_cell =
    ///     Cell::empty(NodeId::Sin(0))
    ///     .input(None, Some(0), Some(0))
    ///     .out(None, Some(0), Some(0));
    ///
    /// let repr = some_cell.to_repr();
    /// assert_eq!(
    ///     repr.serialize().to_string(),
    ///     "[\"sin\",0,0,0,[-1,0,0],[-1,0,0]]");
    ///```
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

    pub fn local_port_idx(&self, dir: CellDir) -> Option<u8> {
        match dir {
            CellDir::TR => { self.out1 },
            CellDir::BR => { self.out2 },
            CellDir::B  => { self.out3 },
            CellDir::BL => { self.in3 },
            CellDir::TL => { self.in2 },
            CellDir::T  => { self.in1 },
            CellDir::C  => None,
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

/// To report back cycle errors from [Matrix::check] and [Matrix::sync].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatrixError {
    CycleDetected,
    DuplicatedInput {
        output1: (NodeId, u8),
        output2: (NodeId, u8),
    },
}

/// An intermediate data structure to store a single edge in the [Matrix].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Edge {
    from:      NodeId,
    from_out:  u8,
    to:        NodeId,
    to_input:  u8,
}

pub struct Matrix {
    /// The node configurator to control the backend.
    config:      NodeConfigurator,
    /// Holds the actual 2 dimensional matrix cells in one big vector.
    matrix:      Vec<Cell>,
    /// Width of the matrix.
    w:           usize,
    /// Height of the matrix.
    h:           usize,

    /// The retained data structure of the graph topology.
    /// This is used by `sync()` and `check()` to determine the
    /// order and cycle freeness of the graph.
    /// We store it in this field, so we don't have to reallocate it
    /// all the time.
    graph_ordering: NodeGraphOrdering,

    /// Holds a saved version of the `matrix` field
    /// to roll back changes that might introduce cycles or
    /// other invalid topology.
    saved_matrix: Option<Vec<Cell>>,

    /// Stores the edges which are extracted from the `matrix` field
    /// by [Matrix::update_graph_ordering_and_edges], which is used
    /// by [Matrix::sync] and [Matrix::check].
    edges: Vec<Edge>,

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
            gen_counter:    0,
            saved_matrix:   None,
            graph_ordering: NodeGraphOrdering::new(),
            edges:          Vec::with_capacity(MAX_ALLOCATED_NODES * 2),
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
        Some(self.config.node_by_id(&node_id)?.0.clone())
    }

    pub fn phase_value_for(&self, node_id: &NodeId) -> f32 {
        self.config.phase_value_for(node_id)
    }

    pub fn led_value_for(&self, node_id: &NodeId) -> f32 {
        self.config.led_value_for(node_id)
    }

    pub fn update_filters(&mut self) {
        self.config.update_filters();
    }

    pub fn filtered_led_for(&mut self, ni: &NodeId) -> (f32, f32) {
        self.config.filtered_led_for(ni)
    }

    pub fn filtered_out_fb_for(&mut self, ni: &NodeId, out: u8) -> (f32, f32) {
        self.config.filtered_out_fb_for(ni, out)
    }

    pub fn get_pattern_data(&self, tracker_id: usize)
        -> Option<Rc<RefCell<PatternData>>>
    {
        self.config.get_pattern_data(tracker_id)
    }

    /// Checks if pattern data updates need to be sent to the
    /// DSP thread.
    pub fn check_pattern_data(&mut self, tracker_id: usize) {
        self.config.check_pattern_data(tracker_id)
    }

    /// Saves the state of the hexagonal grid layout.
    /// This is usually used together with [Matrix::check]
    /// and [Matrix::restore_matrix] to try if changes on
    /// the matrix using [Matrix::place] (or other grid changing
    /// functions).
    ///
    /// It is advised to use convenience functions such as [Matrix::change_matrix].
    ///
    /// See also [Matrix::change_matrix], [Matrix::check] and [Matrix::sync].
    pub fn save_matrix(&mut self) {
        let matrix = self.matrix.clone();
        self.saved_matrix = Some(matrix);
    }

    /// Restores the previously via [Matrix::save_matrix] saved matrix.
    ///
    /// It is advised to use convenience functions such as [Matrix::change_matrix].
    ///
    /// See also [Matrix::change_matrix], [Matrix::check].
    pub fn restore_matrix(&mut self) {
        if let Some(matrix) = self.saved_matrix.take() {
            self.matrix = matrix;
        }
    }

    /// Helps encapsulating changes of the matrix and wraps them into
    /// a [Matrix::save_matrix], [Matrix::check] and [Matrix::restore_matrix].
    ///
    ///```
    /// use hexosynth::*;
    ///
    /// let (node_conf, mut node_exec) = new_node_engine();
    /// let mut matrix = Matrix::new(node_conf, 3, 3);
    ///
    /// let res = matrix.change_matrix(|matrix| {
    ///     matrix.place(0, 1,
    ///         Cell::empty(NodeId::Sin(1))
    ///         .input(Some(0), None, None));
    ///     matrix.place(0, 0,
    ///         Cell::empty(NodeId::Sin(1))
    ///         .out(None, None, Some(0)));
    /// });
    ///
    /// // In this examples case there is an error, as we created
    /// // a cycle:
    /// assert!(res.is_err());
    ///```
    pub fn change_matrix<F>(&mut self, mut f: F)
        -> Result<(), MatrixError>
        where F: FnMut(&mut Self)
    {
        self.save_matrix();

        f(self);

        if let Err(e) = self.check() {
            self.restore_matrix();
            Err(e)
        } else {
            Ok(())
        }
    }

    /// Inserts a cell into the hexagonal grid of the matrix.
    /// You have to make sure that the resulting DSP graph topology
    /// does not have cycles, otherwise an upload to the DSP thread via
    /// [Matrix::sync] will fail.
    ///
    /// You can safely check the DSP topology of changes using
    /// the convenience function [Matrix::change_matrix]
    /// or alternatively: [Matrix::save_matrix], [Matrix::restore_matrix]
    /// and [Matrix::check].
    ///
    /// See also the example in [Matrix::change_matrix] and [Matrix::check].
    pub fn place(&mut self, x: usize, y: usize, mut cell: Cell) {
        cell.x = x as u8;
        cell.y = y as u8;
        self.matrix[x * self.h + y] = cell;
    }

    pub fn clear(&mut self) {
        for cell in self.matrix.iter_mut() {
            *cell = Cell::empty(NodeId::Nop);
        }

        self.graph_ordering.clear();
        self.edges.clear();
        self.saved_matrix = None;

        self.config.delete_nodes();
        self.monitor_cell(Cell::empty(NodeId::Nop));
        let _ = self.sync();
    }

    pub fn for_each_atom<F: FnMut(usize, ParamId, &SAtom)>(&self, f: F) {
        self.config.for_each_param(f);
    }

    /// Returns the DSP graph generation, which is increased
    /// after each call to [Matrix::sync].
    ///
    /// This can be used by external components to track if they
    /// should update their knowledge of the nodes in the DSP
    /// graph. Such as parameter values.
    ///
    /// HexoSynth for instance updates the UI parameters
    /// by tracking this value and calling [Matrix::for_each_atom]
    /// to retrieve the most current set of parameter values.
    /// In case new nodes were created and their default
    /// parameter/atom values were added.
    pub fn get_generation(&self) -> usize { self.gen_counter }

    /// Returns a serializable representation of the matrix.
    /// This representation contains all parameters,
    /// created nodes, connections and the tracker's pattern data.
    ///
    ///```
    /// use hexosynth::*;
    ///
    /// let (node_conf, mut _node_exec) = new_node_engine();
    /// let mut matrix = Matrix::new(node_conf, 3, 3);
    ///
    /// let sin = NodeId::Sin(2);
    ///
    /// matrix.place(0, 0,
    ///     Cell::empty(sin)
    ///     .out(None, Some(0), None));
    ///
    /// let freq_param = sin.inp_param("freq").unwrap();
    /// matrix.set_param(freq_param, SAtom::param(-0.1));
    ///
    /// let mut serialized = matrix.to_repr().serialize().to_string();
    ///
    /// assert!(serialized.find("\"sin\",2,0,0,[-1,-1,-1],[-1,0,-1]").is_some());
    /// assert!(serialized.find("\"freq\",-0.100").is_some());
    ///```
    ///
    /// See also [MatrixRepr::serialize].
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

    /// Loads the matrix from a previously my [Matrix::to_repr]
    /// generated matrix representation.
    ///
    /// This function will call [Matrix::sync] after loading and
    /// overwriting the current matrix contents.
    pub fn from_repr(&mut self, repr: &MatrixRepr) -> Result<(), MatrixError> {
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

        self.sync()
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

    fn create_intermediate_nodes(&mut self) {
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
                            self.config.create_node(new_hole_filler_node_id)
                                .expect("NodeInfo existent in Matrix");
                        }
                    }

                    self.config.create_node(cell.node_id)
                        .expect("NodeInfo existent in Matrix");
                }
            }
        }
    }

    fn update_graph_ordering_and_edges(&mut self) {
        self.graph_ordering.clear();
        self.edges.clear();

        for x in 0..self.w {
            for y in 0..self.h {
                let cell = self.matrix[x * self.h + y];
                if cell.node_id == NodeId::Nop {
                    continue;
                }

                self.graph_ordering.add_node(cell.node_id);

                let in1_output = self.get_adjacent_output(x, y, CellDir::T);
                let in2_output = self.get_adjacent_output(x, y, CellDir::TL);
                let in3_output = self.get_adjacent_output(x, y, CellDir::BL);

                match (cell.in1, in1_output) {
                    (Some(in1_idx), Some(in1_output)) => {
                        self.edges.push(Edge {
                            to:       cell.node_id,
                            to_input: in1_idx,
                            from:     in1_output.0,
                            from_out: in1_output.1,
                        });
                        self.graph_ordering.add_edge(
                            in1_output.0, cell.node_id);
                    },
                    _ => {},
                }

                match (cell.in2, in2_output) {
                    (Some(in2_idx), Some(in2_output)) => {
                        self.edges.push(Edge {
                            to:       cell.node_id,
                            to_input: in2_idx,
                            from:     in2_output.0,
                            from_out: in2_output.1,
                        });
                        self.graph_ordering.add_edge(
                            in2_output.0, cell.node_id);
                    },
                    _ => {},
                }

                match (cell.in3, in3_output) {
                    (Some(in3_idx), Some(in3_output)) => {
                        self.edges.push(Edge {
                            to:       cell.node_id,
                            to_input: in3_idx,
                            from:     in3_output.0,
                            from_out: in3_output.1,
                        });
                        self.graph_ordering.add_edge(
                            in3_output.0, cell.node_id);
                    },
                    _ => {},
                }
            }
        }
    }

    /// Compiles a [NodeProg] from the data collected by the previous
    /// call to [Matrix::update_graph_ordering_and_edges].
    ///
    /// May return an error if the graph topology is invalid (cycles)
    /// or something else happened.
    fn build_prog(&mut self) -> Result<NodeProg, MatrixError> {
        let mut ordered_nodes = vec![];
        if !self.graph_ordering.calculate_order(&mut ordered_nodes) {
            return Err(MatrixError::CycleDetected);
        }

        let mut prog = self.config.rebuild_node_ports();

        for node_id in ordered_nodes.iter() {
            self.config.add_prog_node(&mut prog, node_id);
        }

        for edge in self.edges.iter() {
            self.config.set_prog_node_exec_connection(
                &mut prog,
                (edge.to, edge.to_input),
                (edge.from, edge.from_out));
        }

        Ok(prog)
    }

    /// Checks the topology of the DSP graph represented by the
    /// hexagonal matrix.
    ///
    /// Use [Matrix::save_matrix] and [Matrix::restore_matrix]
    /// for trying out changes before committing them to the
    /// DSP thread using [Matrix::sync].
    ///
    /// Note that there is a convenience function with [Matrix::change_matrix]
    /// to make it easier to test and rollback changes if they are faulty.
    ///
    ///```
    /// use hexosynth::*;
    ///
    /// let (node_conf, mut node_exec) = new_node_engine();
    /// let mut matrix = Matrix::new(node_conf, 3, 3);
    ///
    /// matrix.save_matrix();
    ///
    /// // ...
    /// matrix.place(0, 1,
    ///     Cell::empty(NodeId::Sin(1))
    ///     .input(Some(0), None, None));
    /// matrix.place(0, 0,
    ///     Cell::empty(NodeId::Sin(1))
    ///     .out(None, None, Some(0)));
    /// // ...
    ///
    /// let error =
    ///     if let Err(_) = matrix.check() {
    ///        matrix.restore_matrix();
    ///        true
    ///     } else {
    ///        matrix.sync().unwrap();
    ///        false
    ///     };
    ///
    /// // In this examples case there is an error, as we created
    /// // a cycle:
    /// assert!(error);
    ///```
    pub fn check(&mut self) -> Result<(), MatrixError> {
        self.update_graph_ordering_and_edges();

        let mut edge_map = std::collections::HashMap::new();
        for edge in self.edges.iter() {
            if let Some((out1_node_id, out1_idx)) = edge_map.get(&(edge.to, edge.to_input)) {
                return Err(MatrixError::DuplicatedInput {
                    output1: (*out1_node_id, *out1_idx),
                    output2: (edge.from, edge.from_out),
                });
            } else {
                edge_map.insert(
                    (edge.to, edge.to_input),
                    (edge.from, edge.from_out));
            }
        }

        let mut ordered_nodes = vec![];
        if !self.graph_ordering.calculate_order(&mut ordered_nodes) {
            return Err(MatrixError::CycleDetected);
        }

        Ok(())
    }

    /// Synchronizes the matrix with the DSP thread.
    /// Call this everytime you changed any of the matrix [Cell]s
    /// eg. with [Matrix::place] and want to publish the
    /// changes to the DSP thread.
    ///
    /// This method might return an error, for instance if the
    /// DSP graph topology contains cycles or has other errors.
    ///
    /// You can check any changes and roll them back
    /// using the method [Matrix::change_matrix].
    pub fn sync(&mut self) -> Result<(), MatrixError> {
        self.create_intermediate_nodes();

        self.update_graph_ordering_and_edges();
        let prog = self.build_prog()?;

        self.config.upload_prog(prog, true); // true => copy_old_out

        // Update the generation counter which is used
        // by external data structures to sync their state with
        // the Matrix.
        self.gen_counter += 1;

        // Refresh the input/outputs of the monitored cell,
        // just in case something has changed with that monitored cell.
        self.remonitor_cell();

        Ok(())
    }

    /// Retrieves the output port feedback for a specific output
    /// of the given [NodeId].
    ///
    /// See also [NodeConfigurator::out_fb_for].
    pub fn out_fb_for(&self, node_id: &NodeId, out: u8) -> Option<f32> {
        self.config.out_fb_for(node_id, out)
    }

    /// Updates the output port feedback. Call this every UI frame
    /// or whenever you want to get the most recent values from
    /// [Matrix::out_fb_for].
    ///
    /// See also [NodeConfigurator::update_output_feedback].
    pub fn update_output_feedback(&mut self) {
        self.config.update_output_feedback();
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
        matrix.sync().unwrap();

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
        matrix.sync().unwrap();

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
        matrix.sync().unwrap();

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
        matrix.sync().unwrap();

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

    #[test]
    fn check_matrix_check_cycle() {
        use crate::nodes::new_node_engine;

        let (node_conf, _node_exec) = new_node_engine();
        let mut matrix = Matrix::new(node_conf, 3, 3);

        matrix.save_matrix();
        matrix.place(0, 1,
            Cell::empty(NodeId::Sin(1))
            .input(Some(0), None, None));
        matrix.place(0, 0,
            Cell::empty(NodeId::Sin(1))
            .out(None, None, Some(0)));
        let error =
            if let Err(_) = matrix.check() {
               matrix.restore_matrix();
               true
            } else {
               matrix.sync().unwrap();
               false
            };

        // In this examples case there is an error, as we created
        // a cycle:
        assert!(error);
    }

    #[test]
    fn check_matrix_check_duplicate_input() {
        use crate::nodes::new_node_engine;

        let (node_conf, _node_exec) = new_node_engine();
        let mut matrix = Matrix::new(node_conf, 5, 5);

        matrix.save_matrix();
        matrix.place(0, 1,
            Cell::empty(NodeId::Sin(0))
            .input(Some(0), None, None));
        matrix.place(0, 0,
            Cell::empty(NodeId::Sin(1))
            .out(None, None, Some(0)));

        matrix.place(0, 3,
            Cell::empty(NodeId::Sin(0))
            .input(Some(0), None, None));
        matrix.place(0, 2,
            Cell::empty(NodeId::Sin(2))
            .out(None, None, Some(0)));

        assert_eq!(matrix.check(), Err(MatrixError::DuplicatedInput {
            output1: (NodeId::Sin(1), 0),
            output2: (NodeId::Sin(2), 0),
        }));
    }
}
