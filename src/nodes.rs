const MAX_ALLOCATED_NODES : usize = 256;
const MAX_SMOOTHERS       : usize = 36 + 4; // 6 * 6 modulator inputs + 4 UI Knobs

use crate::monitor::{
    MON_SIG_CNT, new_monitor_processor,
    MonitorBackend, Monitor
};

pub use crate::monitor::MinMaxMonitorSamples;

use std::collections::HashMap;

use ringbuf::{RingBuffer, Producer, Consumer};
use crate::dsp::{
    node_factory, NodeInfo, Node,
    NodeId, SAtom, ProcBuf,
};
use crate::util::Smoother;

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

    pub fn assign_previous_outputs(&mut self, prev_prog: &mut NodeProg) {
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

/// Big messages for updating the NodeExecutor thread.
/// Usually used for shoveling NodeProg and Nodes to and from
/// the NodeExecutor thread.
#[derive(Debug, Clone)]
pub enum GraphMessage {
    NewNode { index: u8, node: Node },
    NewProg { prog: NodeProg, copy_old_out: bool },
}

/// Messages for small updates between the NodeExecutor thread
/// and the NodeConfigurator.
#[derive(Debug)]
pub enum QuickMessage {
    AtomUpdate  { at_idx: usize, value: SAtom },
    ParamUpdate { input_idx: usize, value: f32 },
    /// Sets the buffer indices to monitor with the FeedbackProcessor.
    SetMonitor  { bufs: [usize; MON_SIG_CNT], },
}

pub const UNUSED_MONITOR_IDX : usize = 99999;

/// For receiving deleted/overwritten nodes from the backend
/// thread and dropping them.
struct DropThread {
    terminate: std::sync::Arc<std::sync::atomic::AtomicBool>,
    th:        Option<std::thread::JoinHandle<()>>,
}

impl DropThread {
    fn new(mut graph_drop_con: Consumer<DropMsg>) -> Self {
        let terminate =
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let th_terminate = terminate.clone();

        let th = std::thread::spawn(move || {
            loop {
                if th_terminate.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }

                while let Some(_node) = graph_drop_con.pop() {
                    // drop it ...
                    println!("Dropped some shit...");
                }

                std::thread::sleep(std::time::Duration::from_millis(250));
            }
        });

        Self {
            th: Some(th),
            terminate,
        }
    }
}

impl Drop for DropThread {
    fn drop(&mut self) {
        self.terminate.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = self.th.take().unwrap().join();
    }
}

/// This struct holds the frontend node configuration.
///
/// It stores which nodes are allocated and where.
/// Allocation of new nodes is done here, and parameter management
/// and synchronization is also done by this. It generally acts
/// as facade for the executed node graph in the backend.
pub struct NodeConfigurator {
    /// Holds all the nodes, their parameters and type.
    nodes:              Vec<NodeInfo>,
    /// An index of all nodes ever instanciated.
    /// Be aware, that currently there is no cleanup implemented.
    /// That means, any instanciated NodeId will persist throughout
    /// the whole runtime. A garbage collector might be implemented
    /// when saving presets.
    node2idx:           HashMap<NodeId, usize>,
    /// For updating the NodeExecutor with graph updates.
    graph_update_prod:  Producer<GraphMessage>,
    /// For quick updates like UI paramter changes.
    quick_update_prod:  Producer<QuickMessage>,
    /// For receiving monitor data from the backend thread.
    monitor:            Monitor,
    /// Handles deallocation
    #[allow(dead_code)]
    drop_thread:        DropThread,
}

impl NodeConfigurator {
// FIXME: We can't drop nodes at runtime!
//        We need to reinitialize the whole engine for this.
//        There are too many things relying on the node index (UI).
//
//    pub fn drop_node(&mut self, idx: usize) {
//        if idx >= self.nodes.len() {
//            return;
//        }
//
//        match self.nodes[idx] {
//            NodeInfo::Nop => { return; },
//            _ => {},
//        }
//
//        self.nodes[idx] = NodeInfo::Nop;
//        let _ =
//            self.graph_update_prod.push(
//                GraphMessage::NewNode {
//                    index: idx as u8,
//                    node: Node::Nop,
//                });
//    }

    pub fn for_each<F: FnMut(&NodeInfo, NodeId, usize)>(&self, mut f: F) {
        let mut i = 0;
        for n in self.nodes.iter() {
            let nid = n.to_id();
            if NodeId::Nop == nid {
                break;
            }

            f(n, nid, i);
            i += 1;
        }
    }

    pub fn unique_index_for(&self, ni: NodeId) -> Option<usize> {
        self.node2idx.get(&ni).copied()
    }

    pub fn set_atom(&mut self, at_idx: usize, value: SAtom) {
        let _ =
            self.quick_update_prod.push(
                QuickMessage::AtomUpdate { at_idx, value });
    }

    pub fn set_param(&mut self, input_idx: usize, value: f32) {
        let _ =
            self.quick_update_prod.push(
                QuickMessage::ParamUpdate { input_idx, value });
    }

    pub fn monitor(&mut self, in_bufs: &[usize]) {
        let mut bufs = [0; MON_SIG_CNT];
        bufs.copy_from_slice(&in_bufs);
        let _ = self.quick_update_prod.push(QuickMessage::SetMonitor { bufs });
    }

    pub fn create_node(&mut self, ni: NodeId) -> Option<(&NodeInfo, u8)> {
        println!("create_node: {}", ni);

        if let Some((node, info)) = node_factory(ni) {
            let mut index : Option<usize> = None;

            for i in 0..self.nodes.len() {
                if let NodeInfo::Nop = self.nodes[i] {
                    index = Some(i);
                    break;

                } else if ni == self.nodes[i].to_id() {
                    return Some((&self.nodes[i], i as u8));
                }
            }

            if let Some(index) = index {
                self.node2idx.insert(ni, index);

                self.nodes[index] = info;
                let _ =
                    self.graph_update_prod.push(
                        GraphMessage::NewNode { index: index as u8, node });
                Some((&self.nodes[index], index as u8))

            } else {
                let index = self.nodes.len();
                self.node2idx.insert(ni, index);

                self.nodes.resize_with((self.nodes.len() + 1) * 2, || NodeInfo::Nop);
                self.nodes[index] = info;
                let _ =
                    self.graph_update_prod.push(
                        GraphMessage::NewNode { index: index as u8, node });
                Some((&self.nodes[index], index as u8))
            }
        } else {
            None
        }
    }

    /// Uploads a new NodeProg instance.
    ///
    /// The `copy_old_out` parameter should be set if there are only
    /// new nodes appended to the end of the node instances.
    /// It helps to prevent clicks when there is a feedback path somewhere.
    ///
    /// It must not be set when a completely new set of node instances
    /// was created, for instance when a completely new patch was loaded.
    pub fn upload_prog(&mut self, prog: NodeProg, copy_old_out: bool) {
        let _ =
            self.graph_update_prod.push(
                GraphMessage::NewProg { prog, copy_old_out });
    }

    pub fn get_minmax_monitor_samples(&mut self, idx: usize) -> &MinMaxMonitorSamples {
        self.monitor.get_minmax_monitor_samples(idx)
    }
}

/// Creates a NodeConfigurator and a NodeExecutor which are interconnected
/// by ring buffers.
pub fn new_node_engine() -> (NodeConfigurator, NodeExecutor) {
    let rb_graph     = RingBuffer::new(MAX_ALLOCATED_NODES * 2);
    let rb_quick     = RingBuffer::new(MAX_ALLOCATED_NODES * 8);
    let rb_drop      = RingBuffer::new(MAX_ALLOCATED_NODES * 2);

    let (rb_graph_prod, rb_graph_con) = rb_graph.split();
    let (rb_quick_prod, rb_quick_con) = rb_quick.split();
    let (rb_drop_prod,  rb_drop_con)  = rb_drop.split();

    let (monitor_backend, monitor) = new_monitor_processor();

    let drop_thread = DropThread::new(rb_drop_con);

    let mut nodes = Vec::new();
    nodes.resize_with(MAX_ALLOCATED_NODES, || NodeInfo::Nop);

    let nc = NodeConfigurator {
        nodes,
        graph_update_prod: rb_graph_prod,
        quick_update_prod: rb_quick_prod,
        node2idx:          HashMap::new(),
        monitor,
        drop_thread,
    };

    let mut nodes = Vec::new();
    nodes.resize_with(MAX_ALLOCATED_NODES, || Node::Nop);

    let mut smoothers = Vec::new();
    smoothers.resize_with(MAX_SMOOTHERS, || (0, Smoother::new()));

    let target_refresh = Vec::with_capacity(MAX_SMOOTHERS);

    let ne = NodeExecutor {
        sample_rate:       44100.0,
        nodes,
        smoothers,
        target_refresh,
        prog:              NodeProg::empty(),
        graph_update_con:  rb_graph_con,
        quick_update_con:  rb_quick_con,
        graph_drop_prod:   rb_drop_prod,
        monitor_backend,
        monitor_signal_cur_inp_indices: [UNUSED_MONITOR_IDX; MON_SIG_CNT],
    };

    // XXX: This is one of the earliest and most consistent points
    //      in runtime to do this kind of initialization:
    crate::dsp::helpers::init_cos_tab();

    (nc, ne)
}

/// Operator for transmitting the output of a node
/// to the input of another node.
#[derive(Debug, Clone, Copy)]
pub struct OutOp {
    pub out_port_idx:  u8,
    pub node_idx:      u8,
    pub dst_param_idx: u8
}

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

/*

Rewrite of the core for Vec<f32> (64 samples) buffers:

- There are no dedicated input buffers anymore
    - Input buffers are replaced by an Vec<&[f32]>
      they are cleared and copied into before process() is called.
- There are output buffers in the NodeProg
- There are parameter buffers (with smoothed contents) in the NodeProg
    - They are copied over from the old program!
    - Smoothing is written to the corresponding samples in one go.
- New: NodeProg stores not just input indices, but also
       an extra adjacency table for the inputs of the nodes
       that receive parameters buffers.
       enum SrcIdx {
            Output(usize),
            Param(usize),
       }
       pub inputs: Vec<(<out vec index>, SrcIdx)>

*/

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

#[derive(Debug)]
enum DropMsg {
    Node { node: Node },
    Prog { prog: NodeProg },
    Atom { atom: SAtom },
}

/// Holds the complete allocation of nodes and
/// the program. New Nodes or the program is
/// not newly allocated in the audio backend, but it is
/// copied from the input ring buffer.
/// If this turns out to be too slow, we might
/// have to push buffers of the program around.
///
pub struct NodeExecutor {
    /// Contains the nodes and their state.
    /// Is loaded from the input ring buffer when a corresponding
    /// message arrives.
    ///
    /// In case the previous node contained something that needs
    /// deallocation, the nodes are replaced and the contents
    /// is sent back using the free-ringbuffer.
    nodes: Vec<Node>,

    /// Contains the stand-by smoothing operators for incoming parameter changes.
    smoothers: Vec<(usize, Smoother)>,

    /// Contains target parameter values after a smoother finished,
    /// these will refresh the input buffers:
    target_refresh: Vec<(usize, f32)>,

    /// Contains the to be executed nodes and output operations.
    /// Is copied from the input ringbuffer when a corresponding
    /// message arrives.
    prog: NodeProg,

    /// For receiving Node and NodeProg updates
    graph_update_con:  Consumer<GraphMessage>,
    /// For quick updates like UI paramter changes.
    quick_update_con:  Consumer<QuickMessage>,
    /// For receiving deleted/overwritten nodes from the backend thread.
    graph_drop_prod:   Producer<DropMsg>,
    /// For sending feedback to the frontend thread.
    monitor_backend:   MonitorBackend,

    monitor_signal_cur_inp_indices: [usize; MON_SIG_CNT],

    /// The sample rate
    sample_rate: f32,
}

pub trait NodeAudioContext {
    fn nframes(&self) -> usize;
    fn output(&mut self, channel: usize, frame: usize, v: f32);
    fn input(&mut self, channel: usize, frame: usize) -> f32;
}

impl NodeExecutor {
    #[inline]
    pub fn process_graph_updates(&mut self) {
        while let Some(upd) = self.graph_update_con.pop() {
            match upd {
                GraphMessage::NewNode { index, mut node } => {
                    node.set_sample_rate(self.sample_rate);
                    let prev_node =
                        std::mem::replace(
                            &mut self.nodes[index as usize],
                            node);
                    let _ =
                        self.graph_drop_prod.push(
                            DropMsg::Node { node: prev_node });
                },
                GraphMessage::NewProg { prog, copy_old_out } => {
                    let mut prev_prog = std::mem::replace(&mut self.prog, prog);

                    // XXX: Copying from the old vector works, because we only
                    //      append nodes to the _end_ of the node instance vector.
                    //      If we do a garbage collection, we can't do this.
                    //
                    // XXX: Also, we need to initialize the input parameter
                    //      vector, because we don't know if they are updated from
                    //      the new program outputs anymore. So we need to 
                    //      copy the old paramters to the inputs.
                    //
                    //      => This does not apply to atom data, because that
                    //         is always sent with the new program and "should"
                    //         be up to date, even if we have a slight possible race
                    //         condition between GraphMessage::NewProg
                    //         and QuickMessage::AtomUpdate.
                    if copy_old_out {
                        // XXX: The following is commented out, because presisting
                        //      the output proc buffers does not make sense anymore.
                        //      Because we don't allow cycles, so there is no
                        //      way that a node can read from the previous
                        //      iteration anyways.
                        //
                        // // Swap the old out buffers into the new NodeProg
                        // // TODO: If we toss away most of the buffers anyways,
                        // //       we could optimize this step with more
                        // //       intelligence in the matrix compiler.
                        // for (old_pb, new_pb) in
                        //     prev_prog.out.iter_mut().zip(
                        //         self.prog.out.iter_mut())
                        // {
                        //     std::mem::swap(old_pb, new_pb);
                        // }

                        // First overwrite by the current input parameters,
                        // to make sure _all_ inputs have a proper value
                        // (not just those that existed before).
                        //
                        // We preserve the modulation history in the next step.
                        // This is also to make sure that new input ports
                        // have a proper value too.
                        self.prog.initialize_input_buffers();

                        // Then overwrite the inputs by the more current previous
                        // input processing buffers, so we keep any modulation
                        // (smoothed) history of the block too.
                        self.prog.assign_previous_outputs(&mut prev_prog);

                        self.prog.assign_outputs();
                    }

                    let _ =
                        self.graph_drop_prod.push(
                            DropMsg::Prog { prog: prev_prog });
                },
            }
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        for n in self.nodes.iter_mut() {
            n.set_sample_rate(sample_rate);
        }

        for sm in self.smoothers.iter_mut() {
            sm.1.set_sample_rate(sample_rate);
        }
    }

    #[inline]
    pub fn get_nodes(&self) -> &Vec<Node> { &self.nodes }

    #[inline]
    pub fn get_prog(&self) -> &NodeProg { &self.prog }

    #[inline]
    fn set_param(&mut self, input_idx: usize, value: f32) {
        let prog = &mut self.prog;

        if input_idx >= prog.params.len() {
            return;
        }

        // First check if we already have a running smoother for this param:
        for (sm_inp_idx, smoother) in
            self.smoothers
                .iter_mut()
                .filter(|s| !s.1.is_done())
        {
            if *sm_inp_idx == input_idx {
                smoother.set(prog.params[input_idx], value);
                //d// println!("RE-SET SMOOTHER {} {:6.3} (old = {:6.3})",
                //d//          input_idx, value, prog.params[input_idx]);
                return;
            }
        }

        // Find unused smoother and set it:
        if let Some(sm) =
            self.smoothers
                .iter_mut()
                .filter(|s| s.1.is_done())
                .next()
        {
            sm.0 = input_idx;
            sm.1.set(prog.params[input_idx], value);
            //d// println!("SET SMOOTHER {} {:6.3} (old = {:6.3})",
            //d//          input_idx, value, prog.params[input_idx]);
        }
    }

    #[inline]
    fn process_smoothers(&mut self, nframes: usize) {
        let prog  = &mut self.prog;

        while let Some((idx, v)) = self.target_refresh.pop() {
            prog.inp[idx].fill(v);
        }

        for (idx, smoother) in
            self.smoothers
                .iter_mut()
                .filter(|s|
                    !s.1.is_done())
        {

            let inp        = &mut prog.inp[*idx];
            let mut last_v = 0.0;

            for frame in 0..nframes {
                let v = smoother.next();

                inp.write(frame, v);
                last_v = v;
            }

            prog.params[*idx] = last_v;
            self.target_refresh.push((*idx, last_v));
        }

    }

    #[inline]
    pub fn process_param_updates(&mut self, nframes: usize) {
        while let Some(upd) = self.quick_update_con.pop() {
            match upd {
                QuickMessage::AtomUpdate { at_idx, value } => {
                    let prog = &mut self.prog;
                    let garbage =
                        std::mem::replace(
                            &mut prog.atoms[at_idx],
                            value);
                    let _ =
                        self.graph_drop_prod.push(
                            DropMsg::Atom { atom: garbage });
                },
                QuickMessage::ParamUpdate { input_idx, value } => {
                    self.set_param(input_idx, value);
                },
                QuickMessage::SetMonitor { bufs } => {
                    self.monitor_signal_cur_inp_indices = bufs;
                },
            }
        }

        self.process_smoothers(nframes);
    }

    #[inline]
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T) {
        // let tb = std::time::Instant::now();

        self.process_param_updates(ctx.nframes());

        let nodes = &mut self.nodes;
        let prog  = &mut self.prog;

        for op in prog.prog.iter() {
            let out = op.out_idxlen;
            let inp = op.in_idxlen;
            let at  = op.at_idxlen;

            nodes[op.idx as usize]
                .process(
                    ctx,
                    &prog.atoms[at.0..at.1],
                    &prog.inp[inp.0..inp.1],
                    &prog.cur_inp[inp.0..inp.1],
                    &mut prog.out[out.0..out.1]);
        }

        self.monitor_backend.check_recycle();

        // let ta = std::time::Instant::now();

        for (i, idx) in self.monitor_signal_cur_inp_indices.iter().enumerate() {
            if *idx == UNUSED_MONITOR_IDX {
                continue;
            }

            if let Some(mut mon) = self.monitor_backend.get_unused_mon_buf() {
                if i > 2 {
                    mon.feed(i, ctx.nframes(), &prog.out[*idx]);
                } else {
                    mon.feed(i, ctx.nframes(), &prog.cur_inp[*idx]);
                }

                self.monitor_backend.send_mon_buf(mon);
            }
        }

        // let ta = std::time::Instant::now().duration_since(ta);
        // let tb = std::time::Instant::now().duration_since(tb);
        // println!("ta Elapsed: {:?}", ta);
        // println!("tb Elapsed: {:?}", tb);
    }
}
