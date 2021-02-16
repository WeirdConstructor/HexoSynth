const MAX_ALLOCATED_NODES : usize = 256;
const MAX_NODE_PROG_OPS   : usize = 256 * 3;

use ringbuf::{RingBuffer, Producer, Consumer};
use crate::dsp::{node_factory, NodeInfo, Node};

#[derive(Debug, Clone)]
pub struct NodeProg {
    pub out: Vec<f32>,
    pub prog: Vec<NodeOp>,
}

impl NodeProg {
    pub fn empty() -> Self {
        Self {
            out: vec![],
            prog: vec![],
        }
    }
}

/// Big messages for updating the NodeExecutor thread.
/// Usually used for shoveling NodeProg and Nodes to and from
/// the NodeExecutor thread.
pub enum GraphMessage {
    NewNode { index: u8, node: Node },
    NewProg { prog: NodeProg },
}

/// Messages for small updates between the NodeExecutor thread
/// and the NodeConfigurator.
pub enum QuickMessage {
    ParamUpdate { node_id: u8, param_id: u8, value: f32 },
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
    nodes:  [NodeInfo; MAX_ALLOCATED_NODES],
    /// For updating the NodeExecutor with graph updates.
    graph_update_prod:  Producer<GraphMessage>,
    /// For quick updates like UI paramter changes.
    quick_update_prod:  Producer<QuickMessage>,
    /// For receiving feedback from the backend thread.
    feedback_con:       Consumer<QuickMessage>,
    /// Handles deallocation
    drop_thread:        DropThread,
    sample_rate:        f32,
}

impl NodeConfigurator {
    pub fn create_node(&mut self, name: &str) -> Option<u8> {
        if let Some((node, info)) = node_factory(name, self.sample_rate) {
            let mut index = 0;
            for i in 0..self.nodes.len() {
                if let NodeInfo::Nop = self.nodes[i] {
                    index = i;
                    break;
                }
            }

            self.nodes[index] = info;
            self.graph_update_prod.push(
                GraphMessage::NewNode { index: index as u8, node });
            Some(index as u8)
        } else {
            None
        }
    }

    pub fn upload_prog(&mut self, prog: NodeProg) {
        self.graph_update_prod.push(GraphMessage::NewProg { prog });
    }
}

/// Creates a NodeConfigurator and a NodeExecutor which are interconnected
/// by ring buffers.
pub fn new_node_engine(sample_rate: f32) -> (NodeConfigurator, NodeExecutor) {
    let rb_graph     = RingBuffer::new(MAX_ALLOCATED_NODES * 2);
    let rb_quick     = RingBuffer::new(MAX_ALLOCATED_NODES * 8);
    let rb_drop      = RingBuffer::new(MAX_ALLOCATED_NODES * 2);
    let rb_feedback  = RingBuffer::new(MAX_ALLOCATED_NODES);

    let (rb_graph_prod, rb_graph_con) = rb_graph.split();
    let (rb_quick_prod, rb_quick_con) = rb_quick.split();
    let (rb_drop_prod,  rb_drop_con)  = rb_drop.split();
    let (rb_fb_prod,    rb_fb_con)    = rb_feedback.split();

    let drop_thread = DropThread::new(rb_drop_con);

    let nc = NodeConfigurator {
        nodes:             [NodeInfo::Nop; MAX_ALLOCATED_NODES],
        graph_update_prod: rb_graph_prod,
        quick_update_prod: rb_quick_prod,
        feedback_con:      rb_fb_con,
        drop_thread,
        sample_rate,
    };

    let mut nodes = Vec::new();
    nodes.resize_with(MAX_ALLOCATED_NODES, || Node::Nop);

    let ne = NodeExecutor {
        sample_rate,
        nodes,
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
/// If `calc` is false, the node is not executed and only
/// the output operations are executed.
#[derive(Debug, Clone)]
pub struct NodeOp {
    /// Stores the index of the node
    pub idx:  u8,
    /// Output index and length of the node:
    pub out_idxlen: (usize, usize),
    /// Input indices, (<out vec index>, <own node input index>)
    pub inputs: Vec<(usize, usize)>,
    /// If true, the node needs to be executed. Otherwise only
    /// the output operations are executed.
    pub calc: bool,
    /// Holds the output operations.
    pub out:  Vec<OutOp>,
}

impl NodeOp {
    fn empty() -> Self {
        Self {
            idx:        0,
            calc:       false,
            out_idxlen: (0, 0),
            inputs:     vec![],
            out:        vec![],
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
            println!("UPDATE GRAPH");
            match upd {
                GraphMessage::NewNode { index, node } => {
                    let prev_node =
                        std::mem::replace(
                            &mut self.nodes[index as usize],
                            node);
                    self.graph_drop_prod.push(DropMsg::Node { node: prev_node });
                },
                GraphMessage::NewProg { prog } => {
                    let prev_prog =
                        std::mem::replace(
                            &mut self.prog,
                            prog);
                    self.graph_drop_prod.push(DropMsg::Prog { prog: prev_prog });
                },
            }
        }

        // TODO: Handle quick_update_con to start ramps for the
        //       passed parameters.
    }

    #[inline]
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T) {
        for op in self.prog.prog.iter() {
//        for i in 0..self.prog_len {
//            let op = &self.prog[i];

            if op.calc {
                // TODO: implement a dynamic dispatch set_inputs() here
                //       it receives the prog.out vector and a precomputed
                //       list of index-pairs: (from_out_idx, my_input_index)
                //       Transmit this directly to process(), which then
                //       copies the values to the internal inputs.
                self.nodes[op.idx as usize].process(ctx, &op.inputs, &op.out_idxlen, &mut self.prog.out);
            }

            // TODO: Make the frontend compute the program in a way
            //       that the inputs are all collected into
            //       a vector that is then transmitted as OutOp => rename
            //       to "ReadOp".
            //
            // TODO: Try to move the outputs out of the nodes
            //       into a global vector that is preallocated
            //       with the program.
            //       The program allocates enough outputs of each
            //       executed node.
            //       Node tree and node programm need to be updated
            //       together.
            //       => the process() function gets a mutable output
            //       slice it should write to.
            //       **XXX: This should remove the get() dynamic dispatch!**
            //
            // TODO: Reduce the individual set()s to one big set, that receives
            //       the output vector and a slice of index pairs to
            //       copy the inputs from the outputs.
            //       => This reduces the dynamic lookups to one per node.

//            for out in op.out.iter() {
//                let v = 0.0;
//                let v = outslice[out.out_port_idx as usize];
//                self.nodes[out.node_idx as usize].set(out.dst_param_idx, v);
//            }
        }
    }
}
