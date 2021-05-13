// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use crate::dsp::{ProcBuf, SAtom};

/// Step in a `NodeProg` that stores the to be
/// executed node and output operations.
#[derive(Debug, Clone)]
pub struct NodeOp {
    /// Stores the index of the node
    pub idx:  u8,
    /// Output index and length of the node:
    pub out_idxlen: (usize, usize),
    /// Input index and length of the node:
    pub in_idxlen: (usize, usize),
    /// Atom data index and length of the node:
    pub at_idxlen: (usize, usize),
    /// Input indices, (<out vec index>, <own node input index>)
    pub inputs: Vec<(usize, usize)>,
}

impl std::fmt::Display for NodeOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Op(i={} out=({}-{}) in=({}-{}) at=({}-{})",
               self.idx,
               self.out_idxlen.0,
               self.out_idxlen.1,
               self.in_idxlen.0,
               self.in_idxlen.1,
               self.at_idxlen.0,
               self.at_idxlen.1)?;

        for i in self.inputs.iter() {
            write!(f, " cpy=(o{} => i{})", i.0, i.1)?;
        }

        write!(f, ")")
    }
}

/// A node graph execution program. It comes with buffers
/// for the inputs, outputs and node parameters (knob values).
#[derive(Debug, Clone)]
pub struct NodeProg {
    /// The input vector stores the smoothed values of the params.
    /// It is not used directly, but will be merged into the `cur_inp`
    /// field together with the assigned outputs.
    pub inp:    Vec<ProcBuf>,

    /// The temporary input vector that is initialized from `inp`
    /// and is then merged with the associated outputs.
    pub cur_inp: Vec<ProcBuf>,

    /// The output vector, holding all the node outputs.
    pub out:    Vec<ProcBuf>,

    /// The param vector, holding all parameter inputs of the
    /// nodes, such as knob settings.
    pub params: Vec<f32>,

    /// The atom vector, holding all non automatable parameter inputs
    /// of the nodes, such as samples or integer settings.
    pub atoms:  Vec<SAtom>,

    /// The node operations that are executed in the order they appear in this
    /// vector.
    pub prog:   Vec<NodeOp>,

    /// A marker, that checks if we can still swap buffers with
    /// with other NodeProg instances. This is usally set if the ProcBuf pointers
    /// have been copied into `cur_inp`. You can call `unlock_buffers` to
    /// clear `locked_buffers`:
    pub locked_buffers: bool,
}

impl Drop for NodeProg {
    fn drop(&mut self) {
        for buf in self.inp.iter_mut() {
            buf.free();
        }

        for buf in self.out.iter_mut() {
            buf.free();
        }
    }
}


impl NodeProg {
    pub fn empty() -> Self {
        Self {
            out:     vec![],
            inp:     vec![],
            cur_inp: vec![],
            params:  vec![],
            atoms:   vec![],
            prog:    vec![],
            locked_buffers: false,
        }
    }

    pub fn new(out_len: usize, inp_len: usize, at_len: usize) -> Self {
        let mut out = vec![];
        out.resize_with(out_len, || ProcBuf::new());

        let mut inp = vec![];
        inp.resize_with(inp_len, || ProcBuf::new());
        let mut cur_inp = vec![];
        cur_inp.resize_with(inp_len, || ProcBuf::null());

        let mut params = vec![];
        params.resize(inp_len, 0.0);
        let mut atoms = vec![];
        atoms.resize(at_len, SAtom::setting(0));
        Self {
            out,
            inp,
            cur_inp,
            params,
            atoms,
            prog:           vec![],
            locked_buffers: false,
        }
    }

    pub fn params_mut(&mut self) -> &mut [f32] {
        &mut self.params
    }

    pub fn atoms_mut(&mut self) -> &mut [SAtom] {
        &mut self.atoms
    }

    pub fn append_op(&mut self, node_op: NodeOp) {
        for n_op in self.prog.iter_mut() {
            if n_op.idx == node_op.idx {
                return;
            }
        }

        self.prog.push(node_op);
    }

    pub fn append_edge(
        &mut self,
        mut node_op: NodeOp,
        inp_index: usize,
        out_index: usize)
    {
        for n_op in self.prog.iter_mut() {
            if n_op.idx == node_op.idx {
                n_op.inputs.push((out_index, inp_index));
                return;
            }
        }
    }

    pub fn append_with_inputs(
        &mut self,
        mut node_op: NodeOp,
        inp1: Option<(usize, usize)>,
        inp2: Option<(usize, usize)>,
        inp3: Option<(usize, usize)>)
    {
        for n_op in self.prog.iter_mut() {
            if n_op.idx == node_op.idx {
                if let Some(inp1) = inp1 { n_op.inputs.push(inp1); }
                if let Some(inp2) = inp2 { n_op.inputs.push(inp2); }
                if let Some(inp3) = inp3 { n_op.inputs.push(inp3); }
                return;
            }
        }

        if let Some(inp1) = inp1 { node_op.inputs.push(inp1); }
        if let Some(inp2) = inp2 { node_op.inputs.push(inp2); }
        if let Some(inp3) = inp3 { node_op.inputs.push(inp3); }
        self.prog.push(node_op);
    }

    pub fn initialize_input_buffers(&mut self) {
        for param_idx in 0..self.params.len() {
            let param_val = self.params[param_idx];
            self.inp[param_idx].fill(param_val);
        }
    }

    pub fn swap_previous_outputs(&mut self, prev_prog: &mut NodeProg) {
        if self.locked_buffers {
            self.unlock_buffers();
        }

        if prev_prog.locked_buffers {
            prev_prog.unlock_buffers();
        }

        // XXX: Swapping is now safe, because the `cur_inp` field
        //      no longer references to the buffers in `inp` or `out`.
        for (old_inp_pb, new_inp_pb) in
            prev_prog.inp.iter_mut().zip(
                self.inp.iter_mut())
        {
            std::mem::swap(old_inp_pb, new_inp_pb);
        }
    }

    pub fn unlock_buffers(&mut self) {
        for buf in self.cur_inp.iter_mut() {
            *buf = ProcBuf::null();
        }
        self.locked_buffers = false;
    }

    pub fn assign_outputs(&mut self) {
        for op in self.prog.iter() {

            // First step is copying the ProcBufs to the `cur_inp` current
            // input buffer vector. It holds the data for smoothed paramter
            // inputs or just constant values since the last smoothing.
            //
            // Next we overwrite the input ProcBufs which have an
            // assigned output buffer.
            //
            // ProcBuf has a raw pointer inside, and this copying
            // is therefor very fast.
            //
            // XXX: This requires, that the graph is not cyclic,
            // because otherwise we would write output buffers which
            // are already accessed in the current iteration.
            // This might lead to unexpected effects inside the process()
            // call of the nodes.
            let input_bufs = &mut self.cur_inp;
            let out_bufs   = &mut self.out;

            let inp = op.in_idxlen;

            // First step (refresh inputs):
            input_bufs[inp.0..inp.1]
                .copy_from_slice(&self.inp[inp.0..inp.1]);

            // Second step (assign outputs):
            for io in op.inputs.iter() {
                input_bufs[io.1] = out_bufs[io.0];
            }
        }

        self.locked_buffers = true;
    }
}

