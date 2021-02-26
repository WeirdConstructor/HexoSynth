mod node_amp;
mod node_sin;
mod node_out;

use crate::nodes::NodeAudioContext;

use node_amp::Amp;
use node_sin::Sin;
use node_out::Out;

pub const MIDI_MAX_FREQ : f32 = 13289.75;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub enum UIType {
    Generic,
    LfoA,
    EnvA,
    OscA,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub enum UICategory {
    None,
    Oscillators,
    Time,
    NtoM,
    XtoY,
    IOUtil,
}

macro_rules! node_list {
    ($inmacro: ident) => {
        $inmacro!{
            nop => Nop,
            amp => Amp UIType::Generic UICategory::XtoY
               (0 sig  0.0, 1.0, 0.0)
               (1 gain 0.0, 2.0, 1.0)
               [0 sig],
            sin => Sin UIType::Generic UICategory::Oscillators
               (0 freq 0.0, crate::dsp::MIDI_MAX_FREQ, 440.0)
               [0 sig],
            out => Out UIType::Generic UICategory::IOUtil
               (0 in1  0.0, 1.0, 0.0)
               (1 in2  0.0, 1.0, 0.0),
        }
    }
}

impl UICategory {
    fn get_node_ids(&self, idx: usize, out: &mut Vec<NodeId>) {
        macro_rules! make_cat_lister {
            ($s1: ident => $v1: ident,
                $($str: ident => $variant: ident
                    UIType:: $gui_type: ident
                    UICategory:: $ui_cat: ident
                    $(($in_idx: literal $para: ident $min: expr, $max: expr, $def: expr))*
                    $([$out_idx: literal $out: ident])*
                    ,)+
            ) => {
                $(if UICategory::$ui_cat == *self {
                    out.push(NodeId::$variant(0));
                })+
            }
        }

        node_list!{make_cat_lister};
    }
}

macro_rules! make_node_info_enum {
    ($s1: ident => $v1: ident,
        $($str: ident => $variant: ident
            UIType:: $gui_type: ident
            UICategory:: $ui_cat: ident
            $(($in_idx: literal $para: ident $min: expr, $max: expr, $def: expr))*
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

        pub struct NodeInfoHolder {
            $s1: NodeInfo,
            $($str: NodeInfo),+
        }

        impl NodeInfoHolder {
            pub fn new() -> Self {
                Self {
                    $s1: NodeInfo::$v1,
                    $($str: NodeInfo::$variant(crate::dsp::ni::$variant::new())),+
                }
            }

            pub fn from_node_id(&self, nid: NodeId) -> &NodeInfo {
                match nid {
                    NodeId::$v1           => &self.$s1,
                    $(NodeId::$variant(_) => &self.$str),+
                }
            }
        }

        #[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Eq, Ord, Hash)]
        pub enum NodeId {
            $v1,
            $($variant(u8)),+
        }

        impl NodeId {
            pub fn set_instance(&self, instance: usize) -> NodeId {
                match self {
                    NodeId::$v1           => NodeId::$v1,
                    $(NodeId::$variant(_) => NodeId::$variant(instance as u8)),+
                }
            }

            pub fn from_node_info(ni: &NodeInfo) -> NodeId {
                match ni {
                    NodeInfo::$v1           => NodeId::$v1,
                    $(NodeInfo::$variant(_) => NodeId::$v1),+
                }
            }

            pub fn from_str(name: &str) -> Self {
                match name {
                    stringify!($s1)    => NodeId::$v1,
                    $(stringify!($str) => NodeId::$variant(0)),+,
                    _                  => NodeId::Nop,
                }
            }

            pub fn ui_type(&self) -> UIType {
                match self {
                    NodeId::$v1           => UIType::Generic,
                    $(NodeId::$variant(_) => UIType::$gui_type),+
                }
            }

            pub fn ui_category(&self) -> UICategory {
                match self {
                    NodeId::$v1           => UICategory::None,
                    $(NodeId::$variant(_) => UICategory::$ui_cat),+
                }
            }

            pub fn inp(&self, name: &str) -> Option<u8> {
                match self {
                    NodeId::$v1           => None,
                    $(NodeId::$variant(_) => {
                        match name {
                            $(stringify!($para) => Some($in_idx),)*
                            _ => None,
                        }
                    }),+
                }
            }

            pub fn out(&self, name: &str) -> Option<u8> {
                match self {
                    NodeId::$v1           => None,
                    $(NodeId::$variant(_) => {
                        match name {
                            $(stringify!($out) => Some($out_idx),)*
                            _ => None,
                        }
                    }),+
                }
            }

            pub fn instance(&self) -> usize {
                match self {
                    NodeId::$v1           => 0,
                    $(NodeId::$variant(i) => *i as usize),+
                }
            }

            pub fn set_default_inputs(&self, inputs: &mut [f32]) {
                match self {
                    NodeId::$v1           => {},
                    $(NodeId::$variant(_) => {
                        $(inputs[$in_idx] =
                            crate::dsp::norm_def::$variant::$para());+
                    }),+
                }
            }
        }

        #[allow(non_snake_case)]
        pub mod denorm_v {
            $(pub mod $variant {
                $(#[inline] pub fn $para(x: f32) -> f32 {
                    $min * (1.0 - x) + $max * x
                })*
            })+
        }

        #[allow(non_snake_case)]
        pub mod norm_def {
            $(pub mod $variant {
                $(#[inline] pub fn $para() -> f32 {
                      (($def - $min) / ($max - $min))
                })*
            })+
        }

        #[allow(non_snake_case)]
        pub mod norm {
            $(pub mod $variant {
                $(#[inline] pub fn $para(v: f32) -> f32 {
                      ((v - $min) / ($max - $min)).abs()
                })*
            })+
        }

        #[allow(non_snake_case)]
        pub mod denorm {
            $(pub mod $variant {
                $(#[inline] pub fn $para(inputs: &[f32]) -> f32 {
                    let x = inputs[$in_idx];
                    $min * (1.0 - x) + $max * x
                })*
            })+
        }

        #[allow(non_snake_case)]
        pub mod inp {
            $(pub mod $variant {
                $(#[inline] pub fn $para(inputs: &[f32]) -> f32 {
                    inputs[$in_idx]
                })*
            })+
        }

        #[allow(non_snake_case)]
        pub mod out {
            $(pub mod $variant {
                $(#[inline] pub fn $out(outputs: &mut [f32], v: f32) {
                    outputs[$out_idx] = v;
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
                            $($in_idx => crate::dsp::denorm_v::$variant::$para(x),)+
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

            pub fn to_id(&self, instance: usize) -> NodeId {
                match self {
                    NodeInfo::$v1           => NodeId::$v1,
                    $(NodeInfo::$variant(_) => NodeId::$variant(instance as u8)),+
                }
            }

            pub fn in_count(&self) -> usize {
                match self {
                    NodeInfo::$v1           => 0,
                    $(NodeInfo::$variant(n) => n.in_count()),+
                }
            }

            pub fn out_count(&self) -> usize {
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
            UIType:: $gui_type: ident
            UICategory:: $ui_cat: ident
            $(($in_idx: literal $para: ident $min: expr, $max: expr, $def: expr))*
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

        impl Node {
            pub fn to_id(&self, index: usize) -> NodeId {
                match self {
                    Node::$v1               => NodeId::$v1,
                    $(Node::$variant { .. } => NodeId::$variant(index as u8)),+
                }
            }

            pub fn reset(&mut self) {
                match self {
                    Node::$v1           => {},
                    $(Node::$variant { node } => {
                        node.reset();
                    }),+
                }
            }

            pub fn set_sample_rate(&mut self, sample_rate: f32) {
                match self {
                    Node::$v1           => {},
                    $(Node::$variant { node } => {
                        node.set_sample_rate(sample_rate);
                    }),+
                }
            }

        }
    }
}

node_list!{make_node_info_enum}
node_list!{make_node_enum}

pub fn node_factory(node_id: NodeId) -> Option<(Node, NodeInfo)> {
    println!("Factory: {:?}", node_id);

    macro_rules! make_node_factory_match {
        ($s1: expr => $v1: ident,
            $($str: ident => $variant: ident
                UIType:: $gui_type: ident
                UICategory:: $ui_cat: ident
                $(($in_idx: literal $para: ident $min: expr, $max: expr, $def: expr))*
                $([$out_idx: literal $out: ident])*
            ,)+
        ) => {
            match node_id {
                $(NodeId::$variant(_) => Some((
                    Node::$variant { node: $variant::new() },
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
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T, inputs: &[f32], outputs: &mut [f32]) {
        macro_rules! make_node_process {
            ($s1: ident => $v1: ident,
                $($str: ident => $variant: ident
                    UIType:: $gui_type: ident
                    UICategory:: $ui_cat: ident
                    $(($in_idx: literal $para: ident $min: expr, $max: expr, $def: expr))*
                    $([$out_idx: literal $out: ident])*
                ,)+
            ) => {
                match self {
                    Node::$v1 => {},
                    $(Node::$variant { node } => node.process(ctx, inputs, outputs),)+
                }
            }
        }

        node_list!{make_node_process}
    }
}
