mod amp;
mod sin;

use amp::Amp;
use sin::Sin;

pub const MIDI_MAX_FREQ : f32 = 13289.75;

/// Holds information about the node type that was allocated.
/// Also holds the current parameter values for the UI of the corresponding
/// Node. See also `NodeConfigurator` which holds the information for all
/// the nodes.
#[derive(Debug, Clone, Copy)]
pub enum NodeInfo {
    Nop,
    Amp,
    Sin,
}

/// Holds the complete node program.
#[derive(Debug, Clone)]
pub enum Node {
    /// An empty node that does nothing. It's a placeholder
    /// for non allocated nodes.
    Nop,
    Amp { node: Amp },
    Sin { node: Sin },
}

pub fn node_factory(name: &str, sample_rate: f32) -> Option<(Node, NodeInfo)> {
    println!("Factory: {}", name);
    match name {
        "amp" => Some((
            Node::Amp { node: Amp::new(sample_rate) },
            NodeInfo::Amp
        )),
        "sin" => Some((
            Node::Sin { node: Sin::new(sample_rate) },
            NodeInfo::Sin
        )),
        _     => None,
    }
}

impl Node {
    pub fn set(&mut self, idx: usize, v: f32) {
        match self {
            Node::Nop          => {},
            Node::Amp { node } => node.set(idx, v),
            Node::Sin { node } => node.set(idx, v),
        }
    }

    pub fn process(&mut self) {
        match self {
            Node::Nop          => {},
            Node::Amp { node } => node.process(),
            Node::Sin { node } => node.process(),
        }
    }
}
