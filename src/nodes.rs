const MAX_ALLOCATED_NODES : usize = 256;
const MAX_NODE_PROG_OPS   : usize = 256;

/// Holds information about the node type that was allocated.
/// Also holds the current parameter values for the UI of the corresponding
/// Node. See also `NodeConfigurator` which holds the information for all
/// the nodes.
#[derive(Debug, Clone, Copy)]
enum NodeInfo {
    Nop,
}

use ringbuf::{RingBuffer, Producer, Consumer};

/// Big messages for updating the NodeExecutor thread.
/// Usually used for shoveling NodeProg and Nodes to and from
/// the NodeExecutor thread.
enum GraphMessage {
    NewNode     { node: Node },
    NewNodeProg { prog: [NodeOp; MAX_NODE_PROG_OPS] },
}

/// Messages for small updates between the NodeExecutor thread
/// and the NodeConfigurator.
enum QuickMessage {
    ParamUpdate { node_id: u8, param_id: u8, value: f32 },
    Feedback    { node_id: u8, feedback_id: u8, value: f32 },
}

/// This struct holds the frontend node configuration.
///
/// It stores which nodes are allocated and where.
/// Allocation of new nodes is done here, and parameter management
/// and synchronization is also done by this. It generally acts
/// as facade for the executed node graph in the backend.
struct NodeConfigurator {
    /// Holds all the nodes, their parameters and type.
    nodes:  [NodeInfo; MAX_ALLOCATED_NODES],
    /// For updating the NodeExecutor with graph updates.
    graph_update_prod:  Producer<GraphMessage>,
    /// For quick updates like UI paramter changes.
    quick_update_prod:  Producer<QuickMessage>,
    /// For receiving deleted/overwritten nodes from the backend thread.
    graph_drop_con:     Consumer<Node>,
    /// For receiving feedback from the backend thread.
    feedback_con:       Consumer<QuickMessage>,
}

/// Creates a NodeConfigurator and a NodeExecutor which are interconnected
/// by ring buffers.
fn new_node_engine() -> (NodeConfigurator, NodeExecutor) {
    let rb_graph     = RingBuffer::new(MAX_ALLOCATED_NODES * 2);
    let rb_quick     = RingBuffer::new(MAX_ALLOCATED_NODES * 8);
    let rb_drop      = RingBuffer::new(MAX_ALLOCATED_NODES);
    let rb_feedback  = RingBuffer::new(MAX_ALLOCATED_NODES);

    let (rb_graph_prod, rb_graph_con) = rb_graph.split();
    let (rb_quick_prod, rb_quick_con) = rb_quick.split();
    let (rb_drop_prod,  rb_drop_con)  = rb_drop.split();
    let (rb_fb_prod,    rb_fb_con)    = rb_feedback.split();

    let nc = NodeConfigurator {
        nodes:             [NodeInfo::Nop; MAX_ALLOCATED_NODES],
        graph_update_prod: rb_graph_prod,
        quick_update_prod: rb_quick_prod,
        graph_drop_con:    rb_drop_con,
        feedback_con:      rb_fb_con,
    };

    let mut nodes = Vec::new();
    nodes.resize_with(MAX_ALLOCATED_NODES, || Node::Nop);

    let ne = NodeExecutor {
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
enum OutOp {
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
struct NodeOp {
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
struct NodeExecutor {
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
}

/// Holds the complete node program.
#[derive(Debug, Clone)]
enum Node {
    /// An empty node that does nothing. It's a placeholder
    /// for non allocated nodes.
    Nop,
}
