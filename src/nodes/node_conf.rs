// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use super::{
    GraphMessage, QuickMessage,
    NodeProg, MAX_ALLOCATED_NODES, MAX_AVAIL_TRACKERS,
};
use crate::nodes::drop_thread::DropThread;
use crate::dsp::{NodeId, NodeInfo, Node, SAtom, node_factory};
use crate::util::AtomicFloat;
use crate::monitor::{
    Monitor, MON_SIG_CNT, new_monitor_processor, MinMaxMonitorSamples
};
use crate::dsp::tracker::{Tracker, PatternData};

use ringbuf::{RingBuffer, Producer};
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::collections::HashMap;

/// This struct holds the frontend node configuration.
///
/// It stores which nodes are allocated and where.
/// Allocation of new nodes is done here, and parameter management
/// and synchronization is also done by this. It generally acts
/// as facade for the executed node graph in the backend.
pub struct NodeConfigurator {
    /// Holds all the nodes, their parameters and type.
    pub(crate) nodes:              Vec<NodeInfo>,
    /// An index of all nodes ever instanciated.
    /// Be aware, that currently there is no cleanup implemented.
    /// That means, any instanciated NodeId will persist throughout
    /// the whole runtime. A garbage collector might be implemented
    /// when saving presets.
    pub(crate) node2idx:           HashMap<NodeId, usize>,
    /// Holding the tracker sequencers
    pub(crate) trackers:           Vec<Tracker>,
    /// The shared parts of the [NodeConfigurator] and the [crate::nodes::NodeExecutor].
    pub(crate) shared:             SharedNodeConf,
}

pub(crate) struct SharedNodeConf {
    /// Holds the LED values of the nodes
    pub(crate) node_ctx_values:    Vec<Arc<AtomicFloat>>,
    /// For updating the NodeExecutor with graph updates.
    pub(crate) graph_update_prod:  Producer<GraphMessage>,
    /// For quick updates like UI paramter changes.
    pub(crate) quick_update_prod:  Producer<QuickMessage>,
    /// For receiving monitor data from the backend thread.
    pub(crate) monitor:            Monitor,
    /// Handles deallocation of dead nodes from the backend.
    #[allow(dead_code)]
    pub(crate) drop_thread:        DropThread,
}

use super::node_exec::SharedNodeExec;

impl SharedNodeConf {
    pub(crate) fn new() -> (Self, SharedNodeExec) {
        let rb_graph     = RingBuffer::new(MAX_ALLOCATED_NODES * 2);
        let rb_quick     = RingBuffer::new(MAX_ALLOCATED_NODES * 8);
        let rb_drop      = RingBuffer::new(MAX_ALLOCATED_NODES * 2);

        let (rb_graph_prod, rb_graph_con) = rb_graph.split();
        let (rb_quick_prod, rb_quick_con) = rb_quick.split();
        let (rb_drop_prod,  rb_drop_con)  = rb_drop.split();

        let drop_thread = DropThread::new(rb_drop_con);

        let (monitor_backend, monitor) = new_monitor_processor();

        let mut node_ctx_values = Vec::new();
        node_ctx_values.resize_with(
            2 * MAX_ALLOCATED_NODES,
            || Arc::new(AtomicFloat::new(0.0)));

        let mut exec_node_ctx_vals = Vec::new();
        for ctx_val in node_ctx_values.iter() {
            exec_node_ctx_vals.push(ctx_val.clone());
        }

        (Self {
            node_ctx_values,
            graph_update_prod: rb_graph_prod,
            quick_update_prod: rb_quick_prod,
            monitor,
            drop_thread,
        }, SharedNodeExec {
            node_ctx_values:   exec_node_ctx_vals,
            graph_update_con:  rb_graph_con,
            quick_update_con:  rb_quick_con,
            graph_drop_prod:   rb_drop_prod,
            monitor_backend,
        })
    }
}

impl NodeConfigurator {
    pub(crate) fn new() -> (Self, SharedNodeExec) {
        let mut nodes = Vec::new();
        nodes.resize_with(MAX_ALLOCATED_NODES, || NodeInfo::Nop);

        let (shared, shared_exec) = SharedNodeConf::new();

        (NodeConfigurator {
            nodes,
            shared,
            node2idx:          HashMap::new(),
            trackers:          vec![Tracker::new(); MAX_AVAIL_TRACKERS],
        }, shared_exec)
    }
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

    pub fn unique_index_for(&self, ni: &NodeId) -> Option<usize> {
        self.node2idx.get(&ni).copied()
    }

    pub fn set_atom(&mut self, at_idx: usize, value: SAtom) {
        let _ =
            self.shared.quick_update_prod.push(
                QuickMessage::AtomUpdate { at_idx, value });
    }

    pub fn set_param(&mut self, input_idx: usize, value: f32) {
        let _ =
            self.shared.quick_update_prod.push(
                QuickMessage::ParamUpdate { input_idx, value });
    }

    pub fn phase_value_for(&self, ni: &NodeId) -> f32 {
        if let Some(idx) = self.unique_index_for(ni) {
            self.shared.node_ctx_values[(idx * 2) + 1].get()
        } else {
            0.0
        }
    }

    pub fn led_value_for(&self, ni: &NodeId) -> f32 {
        if let Some(idx) = self.unique_index_for(ni) {
            self.shared.node_ctx_values[idx * 2].get()
        } else {
            0.0
        }
    }

    pub fn monitor(&mut self, in_bufs: &[usize]) {
        let mut bufs = [0; MON_SIG_CNT];
        bufs.copy_from_slice(&in_bufs);
        let _ = self.shared.quick_update_prod.push(QuickMessage::SetMonitor { bufs });
    }

    pub fn get_pattern_data(&self, tracker_id: usize)
        -> Option<Rc<RefCell<PatternData>>>
    {
        if tracker_id >= self.trackers.len() {
            return None;
        }

        Some(self.trackers[tracker_id].data())
    }

    pub fn check_pattern_data(&mut self, tracker_id: usize) {
        if tracker_id >= self.trackers.len() {
            return;
        }

        self.trackers[tracker_id].send_one_update();
    }

    pub fn delete_nodes(&mut self) {
        self.node2idx.clear();
        self.nodes.fill_with(|| NodeInfo::Nop);

        let _ =
            self.shared.graph_update_prod.push(
                GraphMessage::Clear { prog: NodeProg::empty() });
    }

    pub fn create_node(&mut self, ni: NodeId) -> Option<(&NodeInfo, u8)> {
        println!("create_node: {}", ni);

        if let Some((mut node, info)) = node_factory(ni) {
            let mut index : Option<usize> = None;

            if let Node::TSeq { node } = &mut node {
                let tracker_idx = ni.instance();
                if let Some(trk) = self.trackers.get_mut(tracker_idx) {
                    node.set_backend(trk.get_backend());
                }
            }

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
                    self.shared.graph_update_prod.push(
                        GraphMessage::NewNode {
                           index: index as u8,
                           node,
                        });

                Some((&self.nodes[index], index as u8))

            } else {
                let index = self.nodes.len();
                self.node2idx.insert(ni, index);

                self.nodes.resize_with((self.nodes.len() + 1) * 2, || NodeInfo::Nop);
                self.nodes[index] = info;

                let _ =
                    self.shared.graph_update_prod.push(
                        GraphMessage::NewNode {
                            index: index as u8,
                            node,
                        });

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
            self.shared.graph_update_prod.push(
                GraphMessage::NewProg { prog, copy_old_out });
    }

    pub fn get_minmax_monitor_samples(&mut self, idx: usize) -> &MinMaxMonitorSamples {
        self.shared.monitor.get_minmax_monitor_samples(idx)
    }
}

