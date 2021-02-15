mod amp;
mod sin;
mod out;

use crate::nodes::NodeAudioContext;

use amp::Amp;
use sin::Sin;
use out::Out;

pub const MIDI_MAX_FREQ : f32 = 13289.75;

macro_rules! node_list {
    ($inmacro: ident) => {
        $inmacro!{
            "nop" => Nop,
            "amp" => Amp,
            "sin" => Sin,
            "out" => Out,
        }
    }
}

macro_rules! make_node_info_enum {
    ($($str: expr => $variant: ident,)+) => {
        /// Holds information about the node type that was allocated.
        /// Also holds the current parameter values for the UI of the corresponding
        /// Node. See also `NodeConfigurator` which holds the information for all
        /// the nodes.
        #[derive(Debug, Clone, Copy)]
        pub enum NodeInfo {
            $($variant),+
        }
    }
}

macro_rules! make_node_enum {
    ($s1: expr => $v1: ident, $($str: expr => $variant: ident,)+) => {
        /// Holds the complete node program.
        #[derive(Debug, Clone)]
        pub enum Node {
            /// An empty node that does nothing. It's a placeholder
            /// for non allocated nodes.
            $v1,
            $($variant { node: $variant },)+
        }
    }
}

node_list!{make_node_info_enum}
node_list!{make_node_enum}

pub fn node_factory(name: &str, sample_rate: f32) -> Option<(Node, NodeInfo)> {
    println!("Factory: {}", name);

    macro_rules! make_node_factory_match {
        ($s1: expr => $v1: ident, $($str: expr => $variant: ident,)+) => {
            match name {
                $($str => Some((
                    Node::$variant { node: $variant::new(sample_rate) },
                    NodeInfo::$variant
                )),)+
                _ => None,
            }
        }
    }

    node_list!{make_node_factory_match}
}

impl Node {
    pub fn get(&self, idx: u8) -> f32 {
        macro_rules! make_node_set {
            ($s1: expr => $v1: ident, $($str: expr => $variant: ident,)+) => {
                match self {
                    Node::$v1 => 0.0,
                    $(Node::$variant { node } => node.get(idx),)+
                }
            }
        }

        node_list!{make_node_set}
    }

    pub fn set(&mut self, idx: u8, v: f32) {
        macro_rules! make_node_set {
            ($s1: expr => $v1: ident, $($str: expr => $variant: ident,)+) => {
                match self {
                    Node::$v1 => {},
                    $(Node::$variant { node } => node.set(idx, v),)+
                }
            }
        }

        node_list!{make_node_set}
    }

    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T) {
        macro_rules! make_node_process {
            ($s1: expr => $v1: ident, $($str: expr => $variant: ident,)+) => {
                match self {
                    Node::$v1 => {},
                    $(Node::$variant { node } => node.process(ctx),)+
                }
            }
        }

        node_list!{make_node_process}
    }
}
