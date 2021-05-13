// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

pub const MAX_ALLOCATED_NODES : usize = 256;
pub const MAX_SMOOTHERS       : usize = 36 + 4; // 6 * 6 modulator inputs + 4 UI Knobs
pub const MAX_AVAIL_TRACKERS  : usize = 128;

mod node_prog;
mod node_exec;
mod node_conf;
mod drop_thread;
mod node_graph_ordering;

pub use node_exec::*;
pub use node_prog::*;
pub use node_conf::*;
pub use node_graph_ordering::NodeGraphOrdering;

pub use crate::monitor::MinMaxMonitorSamples;
use crate::monitor::MON_SIG_CNT;
use crate::dsp::{Node, SAtom};

#[derive(Debug)]
pub(crate) enum DropMsg {
    Node { node: Node },
    Prog { prog: NodeProg },
    Atom { atom: SAtom },
}

/// Big messages for updating the NodeExecutor thread.
/// Usually used for shoveling NodeProg and Nodes to and from
/// the NodeExecutor thread.
#[derive(Debug)]
pub enum GraphMessage {
    NewNode { index: u8, node: Node },
    NewProg { prog: NodeProg, copy_old_out: bool },
    Clear   { prog: NodeProg },
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

/// Creates a NodeConfigurator and a NodeExecutor which are interconnected
/// by ring buffers.
pub fn new_node_engine() -> (NodeConfigurator, NodeExecutor) {
    let (nc, shared_exec) = NodeConfigurator::new();
    let ne = NodeExecutor::new(shared_exec);

    // XXX: This is one of the earliest and most consistent points
    //      in runtime to do this kind of initialization:
    crate::dsp::helpers::init_cos_tab();

    (nc, ne)
}

