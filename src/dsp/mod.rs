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
            "amp" => Amp (gain 0.0,2.0)             [sig],
            "sin" => Sin (freq 0.0,MIDI_MAX_FREQ)   [sig],
            "out" => Out (in1 0.0,1.0) (in2 0.0,1.0),
        }
    }
}

macro_rules! make_node_info_enum {
    ($s1: expr => $v1: ident,
        $($str: expr => $variant: ident
            $(($para: ident $min: expr, $max: expr))*
            $([$out: ident])*
            ,)+
    ) => {
        /// Holds information about the node type that was allocated.
        /// Also holds the current parameter values for the UI of the corresponding
        /// Node. See also `NodeConfigurator` which holds the information for all
        /// the nodes.
        #[derive(Debug, Clone, Copy)]
        pub enum NodeInfo {
            $v1,
            $($variant),+
        }

        impl NodeInfo {
            pub fn from(s: &str) -> Self {
                match s {
                    $s1    => NodeInfo::$v1,
                    $($str => NodeInfo::$variant),+,
                    _      => NodeInfo::Nop,
                }
            }

            pub fn outputs(&self) -> usize {
                match self {
                    NodeInfo::$v1 => 0,
                    $(NodeInfo::$variant => $variant::outputs()),+
                }
            }
        }
    }
}

macro_rules! make_node_enum {
    ($s1: expr => $v1: ident,
        $($str: expr => $variant: ident
            $(($para: ident $min: expr, $max: expr))*
            $([$out: ident])*
            ,)+
    ) => {
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
        ($s1: expr => $v1: ident,
            $($str: expr => $variant: ident
                $(($para: ident $min: expr, $max: expr))*
                $([$out: ident])*
                ,)+
        ) => {
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
    #[inline]
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T, inputs: &[(usize, usize)], outinfo: &(usize, usize), out: &mut [f32]) {
        macro_rules! make_node_process {
            ($s1: expr => $v1: ident,
                $($str: expr => $variant: ident
                    $(($para: ident $min: expr, $max: expr))*
                    $([$out: ident])*
                    ,)+
            ) => {
                match self {
                    Node::$v1 => {},
                    $(Node::$variant { node } => node.process(ctx, inputs, outinfo, out),)+
                }
            }
        }

        node_list!{make_node_process}
    }
}
