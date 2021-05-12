// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.
use crate::dsp::NodeId;
use crate::nodes::MAX_ALLOCATED_NODES;
use std::collections::HashMap;
use std::collections::HashSet;

pub const MAX_NODE_EDGES    : usize = 64;
pub const UNUSED_NODE_EDGE  : usize = 999999;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Node {
    node_id:    NodeId,
    edges:      [usize; MAX_NODE_EDGES],
    unused_idx: usize,
}

impl Node {
    pub fn new() -> Self {
        Self {
            node_id:    NodeId::Nop,
            edges:      [UNUSED_NODE_EDGE; MAX_NODE_EDGES],
            unused_idx: 0,
        }
    }

    pub fn clear(&mut self) {
        self.node_id    = NodeId::Nop;
        self.edges      = [UNUSED_NODE_EDGE; MAX_NODE_EDGES];
        self.unused_idx = 0;
    }

    pub fn add_edge(&mut self, node_index: usize) {
        for ni in self.edges.iter().take(self.unused_idx) {
            if *ni == node_index {
                return;
            }
        }

        self.edges[self.unused_idx] = node_index;
        self.unused_idx += 1;
    }
}

#[derive(Debug, Clone)]
pub struct NodeGraph {
    node2idx:   HashMap<NodeId, usize>,
    node_count: usize,
    nodes:      [Node; MAX_ALLOCATED_NODES],
}

impl NodeGraph {
    pub fn new() -> Self {
        Self {
            node2idx:   HashMap::new(),
            node_count: 0,
            nodes:      [Node::new(); MAX_ALLOCATED_NODES],
        }
    }

    pub fn clear(&mut self) {
        self.node2idx.clear();
        self.node_count = 0;
    }

    pub fn add_node(&mut self, node_id: NodeId) -> usize {
        if let Some(idx) = self.node2idx.get(&node_id) {
            *idx

        } else {
            let idx = self.node_count;
            self.node_count += 1;

            self.nodes[idx].clear();
            self.nodes[idx].node_id = node_id;
            self.node2idx.insert(node_id, idx);

            idx
        }
    }

    fn get_node(&self, node_id: NodeId) -> Option<&Node> {
        let idx = *self.node2idx.get(&node_id)?;
        Some(&self.nodes[idx])
    }

    fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut Node> {
        let idx = *self.node2idx.get(&node_id)?;
        Some(&mut self.nodes[idx])
    }

    pub fn add_edge(&mut self, from_node_id: NodeId, to_node_id: NodeId) {
        let to_idx = self.add_node(to_node_id);

        if let Some(from_node) = self.get_node_mut(from_node_id) {
            from_node.add_edge(to_idx);
        }
    }

    pub fn has_path(&self, from_node_id: NodeId, to_node_id: NodeId) -> Option<bool> {
        let mut visited_set : HashSet<NodeId> =
            HashSet::with_capacity(MAX_ALLOCATED_NODES);

        let mut node_stack = Vec::with_capacity(MAX_ALLOCATED_NODES);
        node_stack.push(from_node_id);

        while let Some(node_id) = node_stack.pop() {
            if visited_set.contains(&node_id) {
                return None;
            } else {
                visited_set.insert(node_id);
            }

            if node_id == to_node_id {
                return Some(true);
            }

            if let Some(node) = self.get_node(node_id) {
                for node_idx in node.edges.iter().take(node.unused_idx) {
                    node_stack.push(self.nodes[*node_idx].node_id);
                }
            }
        }

        return Some(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_ngraph_dfs_1() {
        let mut ng = NodeGraph::new();
        ng.add_node(NodeId::Sin(2));
        ng.add_node(NodeId::Sin(1));
        ng.add_node(NodeId::Sin(0));
        ng.add_node(NodeId::Sin(0));

        ng.add_edge(NodeId::Sin(2), NodeId::Sin(0));
        ng.add_edge(NodeId::Sin(0), NodeId::Sin(1));

        assert!(ng.has_path(NodeId::Sin(2), NodeId::Sin(1)).unwrap());
        assert!(ng.has_path(NodeId::Sin(2), NodeId::Sin(0)).unwrap());
        assert!(ng.has_path(NodeId::Sin(0), NodeId::Sin(1)).unwrap());
        assert!(!ng.has_path(NodeId::Sin(1), NodeId::Sin(0)).unwrap());
        assert!(!ng.has_path(NodeId::Sin(0), NodeId::Sin(2)).unwrap());
        assert!(!ng.has_path(NodeId::Amp(0), NodeId::Out(2)).unwrap());
    }

    #[test]
    fn check_ngraph_dfs_cycle_2() {
        let mut ng = NodeGraph::new();
        ng.add_node(NodeId::Sin(2));
        ng.add_node(NodeId::Sin(1));
        ng.add_node(NodeId::Sin(0));

        ng.add_edge(NodeId::Sin(2), NodeId::Sin(0));
        ng.add_edge(NodeId::Sin(0), NodeId::Sin(1));
        ng.add_edge(NodeId::Sin(0), NodeId::Sin(2));

        assert!(
            ng.has_path(NodeId::Sin(2), NodeId::Sin(1))
            .is_none());
    }

    #[test]
    fn check_ngraph_clear() {
        let mut ng = NodeGraph::new();
        ng.add_node(NodeId::Sin(2));
        ng.add_node(NodeId::Sin(1));
        ng.add_node(NodeId::Sin(0));

        ng.add_edge(NodeId::Sin(2), NodeId::Sin(0));
        ng.add_edge(NodeId::Sin(0), NodeId::Sin(1));

        assert!(ng.has_path(NodeId::Sin(2), NodeId::Sin(1)).unwrap());

        ng.clear();

        assert!(!ng.has_path(NodeId::Sin(2), NodeId::Sin(1)).unwrap());
    }
}
