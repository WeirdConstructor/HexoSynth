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
            nop => Nop,
            amp => Amp
               (0 gain 0.0, 2.0)
               [0 sig],
            sin => Sin
               (0 freq 0.0, crate::dsp::MIDI_MAX_FREQ)
               [0 sig],
            out => Out
                (0 in1  0.0, 1.0)
                (1 in2  0.0, 1.0),
        }
    }
}

macro_rules! make_node_info_enum {
    ($s1: ident => $v1: ident,
        $($str: ident => $variant: ident
            $(($in_idx: literal $para: ident $min: expr, $max: expr))*
            $([$out_idx: literal $out: ident])*
            ,)+
    ) => {
        /// Holds information about the node type that was allocated.
        /// Also holds the current parameter values for the UI of the corresponding
        /// Node. See also `NodeConfigurator` which holds the information for all
        /// the nodes.
        #[derive(Debug, Clone)]
        pub enum NodeInfo {
            $v1,
            $($variant(crate::dsp::ni::$variant)),+
        }

        pub mod denorm {
            $(pub mod $variant {
                $(#[inline] pub fn $para(x: f32) -> f32 {
                    $min * (1.0 - x) + $max * x
                })*
            })+
        }

        pub mod norm {
            $(pub mod $variant {
                $(#[inline] pub fn $para(v: f32) -> f32 {
                      ((v - $min) / ($max - $min)).abs()
                })*
            })+
        }


        mod ni {
            $(
                #[derive(Debug, Clone)]
                pub struct $variant {
                    inputs: Vec<&'static str>,
                    outputs: Vec<&'static str>,
                }

                impl $variant {
                    pub fn new() -> Self {
                        Self {
                            inputs:  vec![$(stringify!($para),)*],
                            outputs: vec![$(stringify!($out),)*],
                        }
                    }

                    pub fn norm(&self, in_idx: usize, x: f32) -> f32 {
                        match in_idx {
                            $($in_idx => crate::dsp::norm::$variant::$para(x),)+
                            _         => 0.0,
                        }
                    }

                    pub fn denorm(&self, in_idx: usize, x: f32) -> f32 {
                        match in_idx {
                            $($in_idx => crate::dsp::denorm::$variant::$para(x),)+
                            _         => 0.0,
                        }
                    }

                    pub fn out_count(&self) -> usize { self.outputs.len() }
                    pub fn in_count(&self)  -> usize { self.inputs.len() }
                }
            )+
        }

        impl NodeInfo {
            pub fn from(s: &str) -> Self {
                match s {
                    stringify!($s1)    => NodeInfo::$v1,
                    $(stringify!($str) => NodeInfo::$variant(crate::dsp::ni::$variant::new())),+,
                    _                  => NodeInfo::Nop,
                }
            }

            pub fn outputs(&self) -> usize {
                match self {
                    NodeInfo::$v1           => 0,
                    $(NodeInfo::$variant(n) => n.out_count()),+
                }
            }
        }
    }
}

macro_rules! make_node_enum {
    ($s1: ident => $v1: ident,
        $($str: ident => $variant: ident
            $(($in_idx: literal $para: ident $min: expr, $max: expr))*
            $([$out_idx: literal $out: ident])*
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
            $($str: ident => $variant: ident
                $(($in_idx: literal $para: ident $min: expr, $max: expr))*
                $([$out_idx: literal $out: ident])*
            ,)+
        ) => {
            match name {
                $(stringify!($str) => Some((
                    Node::$variant { node: $variant::new(sample_rate) },
                    NodeInfo::$variant(crate::dsp::ni::$variant::new()),
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
            ($s1: ident => $v1: ident,
                $($str: ident => $variant: ident
                    $(($in_idx: literal $para: ident $min: expr, $max: expr))*
                    $([$out_idx: literal $out: ident])*
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
