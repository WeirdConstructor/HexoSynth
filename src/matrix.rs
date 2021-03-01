use crate::nodes::{NodeOp, NodeConfigurator, NodeProg};
use crate::dsp::{NodeInfo, NodeId, ParamId};

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

    pub fn node_id(&self) -> NodeId { self.node_id }

    pub fn label<'a>(&self, mut buf: &'a mut [u8]) -> Option<&'a str> {
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
    prog_idx:   usize,
    out_start:  usize,
    out_end:    usize,
    in_start:   usize,
    in_end:     usize,
}

impl NodeInstance {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            prog_idx:  0,
            out_start: 0,
            out_end:   0,
            in_start:  0,
            in_end:    0,
        }
    }

    pub fn to_op(&self) -> NodeOp {
        NodeOp {
            idx:        self.prog_idx as u8,
            out_idxlen: (self.out_start, self.out_end),
            in_idxlen:  (self.in_start, self.in_end),
            inputs:     vec![],
        }
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
}

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct MatrixNodeParam {
    param_id:   ParamId,
    input_idx:  usize,
    value:      f32,
}

pub struct Matrix {
    config:      NodeConfigurator,
    matrix:      Vec<Cell>,
    w:           usize,
    h:           usize,

    instances:   Rc<RefCell<std::collections::HashMap<NodeId, NodeInstance>>>,
    infos:       Rc<RefCell<std::collections::HashMap<NodeId, NodeInfo>>>,
    params:      Rc<RefCell<std::collections::HashMap<ParamId, MatrixNodeParam>>>,
    params_old:  Rc<RefCell<std::collections::HashMap<ParamId, MatrixNodeParam>>>,
}

unsafe impl Send for Matrix {}

impl Matrix {
    pub fn new(config: NodeConfigurator, w: usize, h: usize) -> Self {
        let mut matrix : Vec<Cell> = Vec::new();
        matrix.resize(w * h, Cell::empty(NodeId::Nop));

        Self {
            instances:   Rc::new(RefCell::new(std::collections::HashMap::new())),
            infos:       Rc::new(RefCell::new(std::collections::HashMap::new())),
            params:      Rc::new(RefCell::new(std::collections::HashMap::new())),
            params_old:  Rc::new(RefCell::new(std::collections::HashMap::new())),
            config,
            w,
            h,
            matrix,
        }
    }

    pub fn size(&self) -> (usize, usize) { (self.w, self.h) }

    pub fn into_conf(self) -> NodeConfigurator {
        self.config
    }

    pub fn place(&mut self, x: usize, y: usize, mut cell: Cell) {
        cell.x = x as u8;
        cell.y = y as u8;
        self.matrix[x * self.h + y] = cell;
    }

    pub fn set_param(&mut self, param: ParamId, v: f32) {
        if let Some(nparam) = self.params.borrow_mut().get_mut(&param) {
            nparam.value = v;
            self.config.set_param(nparam.input_idx, v);
        }
    }

    pub fn get_adjacent_out_vec_index(&self, x: usize, y: usize, dir: u8) -> Option<usize> {
        if dir > 5 || dir < 3 {
            return None;
        }

        if let Some(cell) = self.get_adjacent(x, y, dir) {
            if cell.node_id != NodeId::Nop {
                // check output 3
                // - get the associated output index
                // - get the NodeInstance of this cell
                // - add the assoc output index to the output-index
                //   of the node instance

                let instances = self.instances.borrow();
                match dir {
                    5 => {
                        if let Some(cell_out_i) = cell.out3 {
                            let ni = instances.get(&cell.node_id).unwrap();
                            Some(ni.out_start + cell_out_i as usize)
                        } else {
                            None
                        }
                    },
                    4 => {
                        if let Some(cell_out_i) = cell.out2 {
                            let ni = instances.get(&cell.node_id).unwrap();
                            Some(ni.out_start + cell_out_i as usize)
                        } else {
                            None
                        }
                    },
                    3 => {
                        if let Some(cell_out_i) = cell.out1 {
                            let ni = instances.get(&cell.node_id).unwrap();
                            Some(ni.out_start + cell_out_i as usize)
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

    pub fn get_adjacent(&self, x: usize, y: usize, dir: u8) -> Option<&Cell> {
        let offs : (i32, i32) =
            match dir {
                // out 1 - TR
                0 => (0, 1),
                // out 2 - BR
                1 => (1, 1),
                // out 3 - B
                2 => (0, 1),
                // in 3 - BL
                3 => (-1, 1),
                // in 2 - TL
                4 => (-1, 0),
                // in 1 - T
                5 => (0, -1),
                _ => (0, 0),
            };
        let x = x as i32 + offs.0;
        let y = y as i32 + offs.1;

        if x < 0 || y < 0 || (x as usize) >= self.w || (y as usize) >= self.h {
            return None;
        }

        Some(&self.matrix[(x as usize) * self.h + (y as usize)])
    }

    pub fn for_each<F: Fn(usize, usize, &Cell)>(&self, f: F) {
        for x in 0..self.w {
            for y in 0..self.h {
                let cell = &self.matrix[x * self.h + y];
                f(x, y, cell);
            }
        }
    }

    pub fn edge_label<'a>(&self, cell: &Cell, edge: u8, mut buf: &'a mut [u8]) -> Option<&'a str> {
        use std::io::Write;
        let mut cur = std::io::Cursor::new(buf);

        if cell.node_id == NodeId::Nop {
            return None;
        }

        let out_idx =
            match edge {
                // out 1 - TR
                0 => Some(cell.out1),
                // out 2 - BR
                1 => Some(cell.out2),
                // out 3 - B
                2 => Some(cell.out3),
                _ => None,
            };
        let in_idx =
            match edge {
                // in 3 - BL
                3 => Some(cell.in3),
                // in 2 - TL
                4 => Some(cell.in2),
                // in 1 - T
                5 => Some(cell.in1),
                _ => None,
            };

        let infos = self.infos.borrow();
        let info = infos.get(&cell.node_id)?;

        let edge_str =
            if let Some(out_idx) = out_idx {
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
                Some(
                    std::str::from_utf8(&(cur.into_inner())[0..len])
                    .unwrap())
            },
            Err(_) => None,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        if x >= self.w || y >= self.h {
            return None;
        }

        Some(&self.matrix[x * self.h + y])
    }

    pub fn sync(&mut self) {
        self.instances.borrow_mut().clear();

        println!("FOO");

        // Build instance map, to find new nodes in the matrix.
        self.config.for_each(|node_info, mut id, _i| {
            while let Some(_) = self.instances.borrow().get(&id) {
                println!("OOO {:?}", id);
                id = id.set_instance(id.instance() + 1);
            }

            self.instances.borrow_mut().insert(id, NodeInstance::new(id));
            self.infos.borrow_mut().insert(id, node_info.clone());
        });

        // Scan thought the matrix and check if (backend) nodes need to be created
        // for new unknown nodes:
        for x in 0..self.w {
            for y in 0..self.h {
                let mut cell = &mut self.matrix[x * self.h + y];

                if cell.node_id == NodeId::Nop {
                    continue;
                }

                // - check if each NodeId has a corresponding entry in NodeConfigurator
                //   - if not, create a new one on the fly
                if self.instances.borrow().get(&cell.node_id).is_none() {
                    // - check if the previous node instances exist, if not,
                    //   create them on the fly now:
                    for inst in 0..cell.node_id.instance() {
                        let new_hole_filler_node_id =
                            cell.node_id.set_instance(inst);

                        if self.instances.borrow()
                            .get(&new_hole_filler_node_id)
                            .is_none()
                        {
                            let (info, _idx) =
                                self.config.create_node(new_hole_filler_node_id)
                                    .expect("NodeInfo existent in Matrix");
                            self.infos.borrow_mut()
                                .insert(new_hole_filler_node_id, info.clone());
                            self.instances.borrow_mut().insert(
                                new_hole_filler_node_id,
                                NodeInstance::new(new_hole_filler_node_id));
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

        // Swap old and current parameter. Keep the old ones
        // as reference.
        std::mem::swap(&mut self.params_old, &mut self.params);
        self.params.borrow_mut().clear();

        let mut out_len = 0;
        let mut in_len  = 0;
        self.config.for_each(|node_info, mut id, i| {
            // - calculate size of output vector.
            let out_idx = out_len;
            out_len += node_info.out_count();

            // - calculate size of input vector.
            let in_idx = in_len;
            in_len += node_info.in_count();

            println!("{} = {}", i, id);

            // Create new parameters and initialize them if they did not
            // already exist from a previous matrix instance.
            for param_idx in in_idx..in_len {
                if let Some(param_id) = id.inp_param_by_idx(param_idx - in_idx) {
                    if let Some(old_param) = self.params_old.borrow().get(&param_id) {
                        self.params.borrow_mut().insert(param_id, *old_param);

                    } else {
                        self.params.borrow_mut().insert(param_id, MatrixNodeParam {
                            param_id,
                            input_idx:  param_idx,
                            value:      param_id.norm_def(),
                        });
                    }
                }
            }

            println!("INSERT: {:?} outidx: {},{} inidx: {},{}",
                     id, out_idx, out_len, in_idx, in_len);

            // - save offset and length of each node's
            //   allocation in the output vector.
            self.instances.borrow_mut().insert(id,
                NodeInstance::new(id)
                .set_index(i)
                .set_output(out_idx, out_len)
                .set_input(in_idx, in_len));
        });

        let mut prog = NodeProg::new(out_len, in_len);

        for x in 0..self.w {
            for y in 0..self.h {
                // Get the indices to the output vector for the
                // corresponding input ports.
                let in_1_out_idx = self.get_adjacent_out_vec_index(x, y, 5);
                let in_2_out_idx = self.get_adjacent_out_vec_index(x, y, 4);
                let in_3_out_idx = self.get_adjacent_out_vec_index(x, y, 3);

                let mut cell = &mut self.matrix[x * self.h + y];

                if cell.node_id == NodeId::Nop {
                    continue;
                }

                let instances = self.instances.borrow();
                let ni = instances.get(&cell.node_id).unwrap();
                let op = ni.to_op();

                let in_1 =
                    if let Some(in1_idx) = cell.in1 {
                        if let Some(in1_out_idx) = in_1_out_idx {
                            Some((in1_out_idx, ni.in_start + in1_idx as usize))
                        } else { None }
                    } else { None };

                let in_2 =
                    if let Some(in2_idx) = cell.in2 {
                        if let Some(in2_out_idx) = in_2_out_idx {
                            Some((in2_out_idx, ni.in_start + in2_idx as usize))
                        } else { None }
                    } else { None };

                let in_3 =
                    if let Some(in3_idx) = cell.in3 {
                        if let Some(in3_out_idx) = in_3_out_idx {
                            Some((in3_out_idx, ni.in_start + in3_idx as usize))
                        } else { None }
                    } else { None };

                prog.append_with_inputs(op, in_1, in_2, in_3);
            }
        }

        // Copy the parameter values into the program:
        // They are extracted by process_graph_updates() later to
        // reset the inp[] input value vector.
        for (param_id, param) in self.params.borrow().iter() {
            prog.params_mut()[param.input_idx] = param.value;
        }

        self.config.upload_prog(prog, true); // true => copy_old_out

        println!("FBAROO");
        // - after each node has been created, use the node ordering
        //   in NodeConfigurator to create an output vector.
        //      - When a new output vector is received in the backend,
        //        the backend needs to copy over the previous data.
        //        XXX: This works, because we don't delete nodes.
        //             If we do garbage collection, we can risk a short click
        //             Maybe ramp up the volume after a GC!
        //      - Store all nodes and their output vector offset and length
        //        in a list for reference.
        // - iterate through the matrix, column by column:
        //      - create program vector
        //          - If NodeId is not found, create a new NodeOp at the end
        //          - Append all inputs of the current Cell to the NodeOp
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

        let (mut node_conf, mut node_exec) = new_node_engine();
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
        assert_eq!(prog.prog[0].to_string(), "Op(i=0 out=(0-1) in=(0-1))");
        assert_eq!(prog.prog[1].to_string(), "Op(i=1 out=(1-2) in=(1-2) cpy=(o0 => i1))");
        assert_eq!(prog.prog[2].to_string(), "Op(i=2 out=(2-3) in=(2-3) cpy=(o1 => i2))");
    }

    #[test]
    fn check_matrix_filled() {
        use crate::nodes::new_node_engine;
        use crate::dsp::{NodeId, Node};

        let (mut node_conf, mut node_exec) = new_node_engine();
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

        let (mut node_conf, mut node_exec) = new_node_engine();
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
        assert_eq!(prog.prog[0].to_string(), "Op(i=0 out=(0-1) in=(0-1))");
        assert_eq!(prog.prog[1].to_string(), "Op(i=1 out=(1-1) in=(1-3) cpy=(o0 => i1))");
    }

    #[test]
    fn check_matrix_skip_instance() {
        use crate::nodes::new_node_engine;

        let (mut node_conf, mut node_exec) = new_node_engine();
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
        assert_eq!(prog.prog[0].to_string(), "Op(i=2 out=(2-3) in=(2-3))");
        assert_eq!(prog.prog[1].to_string(), "Op(i=3 out=(3-3) in=(3-5) cpy=(o2 => i3))");
    }
}
