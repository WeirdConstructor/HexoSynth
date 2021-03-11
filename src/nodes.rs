const MAX_ALLOCATED_NODES : usize = 256;
const MAX_SMOOTHERS       : usize = 36 + 4; // 6 * 6 modulator inputs + 4 UI Knobs

use ringbuf::{RingBuffer, Producer, Consumer};
use crate::dsp::{node_factory, NodeInfo, Node, NodeId};
use crate::util::Smoother;

/// A node graph execution program. It comes with buffers
/// for the inputs, outputs and node parameters (knob values).
#[derive(Debug, Clone)]
pub struct NodeProg {
    /// The output vector, holding all the node outputs.
    pub out:    Vec<f32>,
    /// The param vector, holding all parameter inputs of the nodes, such as knob settings.
    pub params: Vec<f32>,
    /// The input vector that feeds the nodes. It's initialized from params
    /// and then changed by the program signal paths that overwrite them
    /// with the values in the node outputs.
    pub inp:    Vec<f32>,
    /// The node operations that are executed in the order they appear in this
    /// vector.
    pub prog:   Vec<NodeOp>,
}

impl NodeProg {
    pub fn empty() -> Self {
        Self {
            out:    vec![],
            inp:    vec![],
            params: vec![],
            prog:   vec![],
        }
    }

    pub fn new(out_len: usize, inp_len: usize) -> Self {
        let mut out = vec![];
        out.resize(out_len, 0.0);
        let mut inp = vec![];
        inp.resize(inp_len, 0.0);
        let mut params = vec![];
        params.resize(inp_len, 0.0);
        Self {
            out,
            inp,
            params,
            prog: vec![],
        }
    }

    pub fn params_mut(&mut self) -> &mut [f32] {
        &mut self.params
    }

    pub fn append_with_inputs(
        &mut self,
        mut node_op: NodeOp,
        inp1: Option<(usize, usize)>,
        inp2: Option<(usize, usize)>,
        inp3: Option<(usize, usize)>)
    {
        println!("PROG APPEND: {}", node_op);
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
pub enum QuickMessage {
    ParamUpdate { input_idx: usize, value: f32 },
    Feedback    { node_id: u8, feedback_id: u8, value: f32 },
}

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
        self.th.take().unwrap().join();
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
    /// For updating the NodeExecutor with graph updates.
    graph_update_prod:  Producer<GraphMessage>,
    /// For quick updates like UI paramter changes.
    quick_update_prod:  Producer<QuickMessage>,
    /// For receiving feedback from the backend thread.
    feedback_con:       Consumer<QuickMessage>,
    /// Handles deallocation
    drop_thread:        DropThread,
}

impl NodeConfigurator {
    pub fn drop_node(&mut self, idx: usize) {
        if idx >= self.nodes.len() {
            return;
        }

        match self.nodes[idx] {
            NodeInfo::Nop => { return; },
            _ => {},
        }

        self.nodes[idx] = NodeInfo::Nop;
        self.graph_update_prod.push(
            GraphMessage::NewNode {
                index: idx as u8,
                node: Node::Nop,
            });
    }

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

    pub fn set_param(&mut self, input_idx: usize, value: f32) {
        self.quick_update_prod.push(
            QuickMessage::ParamUpdate { input_idx, value });
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
                self.nodes[index] = info;
                self.graph_update_prod.push(
                    GraphMessage::NewNode { index: index as u8, node });
                Some((&self.nodes[index], index as u8))

            } else {
                let index = self.nodes.len();
                self.nodes.resize_with((self.nodes.len() + 1) * 2, || NodeInfo::Nop);
                self.nodes[index] = info;
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
        self.graph_update_prod.push(GraphMessage::NewProg { prog, copy_old_out });
    }
}

/// Creates a NodeConfigurator and a NodeExecutor which are interconnected
/// by ring buffers.
pub fn new_node_engine() -> (NodeConfigurator, NodeExecutor) {
    let rb_graph     = RingBuffer::new(MAX_ALLOCATED_NODES * 2);
    let rb_quick     = RingBuffer::new(MAX_ALLOCATED_NODES * 8);
    let rb_drop      = RingBuffer::new(MAX_ALLOCATED_NODES * 2);
    let rb_feedback  = RingBuffer::new(MAX_ALLOCATED_NODES);

    let (rb_graph_prod, rb_graph_con) = rb_graph.split();
    let (rb_quick_prod, rb_quick_con) = rb_quick.split();
    let (rb_drop_prod,  rb_drop_con)  = rb_drop.split();
    let (rb_fb_prod,    rb_fb_con)    = rb_feedback.split();

    let drop_thread = DropThread::new(rb_drop_con);

    let mut nodes = Vec::new();
    nodes.resize_with(MAX_ALLOCATED_NODES, || NodeInfo::Nop);

    let nc = NodeConfigurator {
        nodes,
        graph_update_prod: rb_graph_prod,
        quick_update_prod: rb_quick_prod,
        feedback_con:      rb_fb_con,
        drop_thread,
    };

    let mut nodes = Vec::new();
    nodes.resize_with(MAX_ALLOCATED_NODES, || Node::Nop);

    let mut smoothers = Vec::new();
    smoothers.resize_with(MAX_SMOOTHERS, || (0, Smoother::new()));

    let ne = NodeExecutor {
        sample_rate:       44100.0,
        nodes,
        smoothers,
        prog:              NodeProg::empty(),
        graph_update_con:  rb_graph_con,
        quick_update_con:  rb_quick_con,
        graph_drop_prod:   rb_drop_prod,
        feedback_prod:     rb_fb_prod,
    };

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
        write!(f, "Op(i={} out=({}-{}) in=({}-{})",
               self.idx,
               self.out_idxlen.0,
               self.out_idxlen.1,
               self.in_idxlen.0,
               self.in_idxlen.1)?;

        for i in self.inputs.iter() {
            write!(f, " cpy=(o{} => i{})", i.0, i.1)?;
        }

        write!(f, ")")
    }
}

impl NodeOp {
    fn empty() -> Self {
        Self {
            idx:        0,
            out_idxlen: (0, 0),
            in_idxlen:  (0, 0),
            inputs:     vec![],
        }
    }
}

#[derive(Debug)]
enum DropMsg {
    Node { node: Node },
    Prog { prog: NodeProg },
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
    /// For receiving feedback from the backend thread.
    feedback_prod:     Producer<QuickMessage>,

    /// The sample rate
    sample_rate: f32,
}

pub trait NodeAudioContext {
    fn output(&mut self, channel: usize, v: f32);
    fn input(&mut self, channel: usize) -> f32;
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
                    self.graph_drop_prod.push(DropMsg::Node { node: prev_node });
                },
                GraphMessage::NewProg { prog, copy_old_out } => {
                    let prev_prog = std::mem::replace(&mut self.prog, prog);

                    // XXX: Copying from the old vector works, because we only
                    //      append nodes to the _end_ of the node instance vector.
                    //      If we do a garbage collection, we can't do this.
                    //
                    // XXX: Also, we need to initialize the input parameter
                    //      vector, because we don't know if they are updated from
                    //      the new program outputs anymore. So we need to 
                    //      copy the old paramters to the inputs.
                    if copy_old_out {
                        let old_len = prev_prog.out.len();
                        self.prog.out[0..old_len].copy_from_slice(
                            &prev_prog.out[..]);


                        // First overwrite by the new paramters, to make sure
                        // _all_ inputs have a proper value (not just those
                        // that existed before):
                        self.prog.inp[0..self.prog.params.len()]
                            .copy_from_slice(&self.prog.params[..]);
                        // Then overwrite the inputs by the more current previous
                        // parameters:
                        self.prog.inp[0..prev_prog.params.len()]
                            .copy_from_slice(&prev_prog.params[..]);
                    }

                    self.graph_drop_prod.push(DropMsg::Prog { prog: prev_prog });
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
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T) {

        while let Some(upd) = self.quick_update_con.pop() {
            match upd {
                QuickMessage::ParamUpdate { input_idx, value } => {
                    self.set_param(input_idx, value);
                },
                _ => {},
            }
        }

        let nodes = &mut self.nodes;
        let prog  = &mut self.prog;

        for (idx, smoother) in self.smoothers.iter_mut().filter(|s| !s.1.is_done()) {
            let v = smoother.next();
            prog.inp[*idx]    = v;
            prog.params[*idx] = v;
        }

        // XXX: We can overwrite the inp input value vector with the outputs,
        // because we always do this the same way as long as the program
        // stays the same.
        //
        // If a new program comes, we need to initialize the new program inputs
        // with the old parameters, because we don't know if they are not
        // written by the following loop anymore.
        //
        // See also above in process_graph_updates().

        // TESTING:
        //d// prog.inp.copy_from_slice(&prog.params[..]);

        for op in prog.prog.iter() {
            //d// println!("EXEC> {}", op);

            let out = op.out_idxlen;
            let inp = op.in_idxlen;
            {
                let input = &mut prog.inp;
                let out   = &prog.out;
                for io in op.inputs.iter() {
                    //d// println!("IL={}, OL={}, {}<={}",
                    //             input.len(), out.len(), io.1, io.0);
                    input[io.1] = out[io.0];
                }
            }

            nodes[op.idx as usize]
            .process(
                ctx,
                &prog.inp[inp.0..inp.1],
                &mut prog.out[out.0..out.1]);
        }
    }
}
