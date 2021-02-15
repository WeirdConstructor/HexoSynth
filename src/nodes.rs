const MAX_ALLOCATED_NODES : usize = 256;
const MAX_NODE_PROG_OPS   : usize = 256;

use ringbuf::{RingBuffer, Producer, Consumer};
use crate::dsp::{node_factory, NodeInfo, Node};

/// Big messages for updating the NodeExecutor thread.
/// Usually used for shoveling NodeProg and Nodes to and from
/// the NodeExecutor thread.
pub enum GraphMessage {
    NewNode { index: u8, node: Node },
    NewProg { prog: [NodeOp; MAX_NODE_PROG_OPS] },
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
    fn new(mut graph_drop_con: Consumer<Node>) -> Self {
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
    pub fn create_node(&mut self, name: &str) -> Option<usize> {
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
            Some(index)
        } else {
            None
        }
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
        prog:              [NodeOp::empty(); MAX_NODE_PROG_OPS],
        graph_update_con:  rb_graph_con,
        quick_update_con:  rb_quick_con,
        graph_drop_prod:   rb_drop_prod,
        feedback_prod:     rb_fb_prod,
    };

    (nc, ne)
}

/// Operators for transmitting the output of a node
/// to the input of another node.
#[derive(Debug, Clone, Copy)]
pub enum OutOp {
    Nop,
    Transfer {
        out_port_idx:  u8,
        dst_param_idx: u8
    },
}

/// Step in a `NodeProg` that stores the to be
/// executed node and output operations.
/// If `calc` is false, the node is not executed and only
/// the output operations are executed.
#[derive(Debug, Clone, Copy)]
pub struct NodeOp {
    /// Stores the index of the node
    idx:  u8,
    /// If true, the node needs to be executed. Otherwise only
    /// the output operations are executed.
    calc: bool,
    /// Holds the output operations.
    out:  [OutOp; 3],
}

impl NodeOp {
    fn empty() -> Self {
        Self {
            idx:    0,
            calc:   false,
            out:    [OutOp::Nop; 3],
        }
    }
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
    prog:  [NodeOp; MAX_NODE_PROG_OPS],

    /// For receiving Node and NodeProg updates
    graph_update_con:  Consumer<GraphMessage>,
    /// For quick updates like UI paramter changes.
    quick_update_con:  Consumer<QuickMessage>,
    /// For receiving deleted/overwritten nodes from the backend thread.
    graph_drop_prod:   Producer<Node>,
    /// For receiving feedback from the backend thread.
    feedback_prod:     Producer<QuickMessage>,

    /// The sample rate
    sample_rate: f32,
}

impl NodeExecutor {
    pub fn process_graph_updates(&mut self) {
        while let Some(upd) = self.graph_update_con.pop() {
            match upd {
                GraphMessage::NewNode { index, node } => {
                    let prev_node =
                        std::mem::replace(
                            &mut self.nodes[index as usize],
                            node);
                    self.graph_drop_prod.push(prev_node);
                },
                GraphMessage::NewProg { prog } => {
                    self.prog = prog;
                },
            }
        }

        // TODO: Handle quick_update_con to start ramps for the
        //       passed parameters.
    }

    pub fn process(&mut self) {
        for n in self.nodes.iter_mut() {
            n.process();
        }
    }
}
