#[allow(non_upper_case_globals)]
mod node_amp;
#[allow(non_upper_case_globals)]
mod node_sin;
#[allow(non_upper_case_globals)]
mod node_out;
mod satom;
pub mod helpers;

use crate::nodes::NodeAudioContext;

pub use satom::*;

use node_amp::Amp;
use node_sin::Sin;
use node_out::Out;

pub const MIDI_MAX_FREQ : f32 = 13289.75;

pub const MAX_BLOCK_SIZE : usize = 64;

/// A processing buffer with the exact right maximum size.
#[derive(Clone, Copy)]
pub struct ProcBuf(*mut [f32; MAX_BLOCK_SIZE]);

impl ProcBuf {
    pub fn new() -> Self {
        ProcBuf(Box::into_raw(Box::new([0.0; MAX_BLOCK_SIZE])))
    }

    pub fn null() -> Self {
        ProcBuf(std::ptr::null_mut())
    }
}

unsafe impl Send for ProcBuf {}
//unsafe impl Sync for HexoSynthShared {}

impl ProcBuf {
    #[inline]
    pub fn write(&mut self, idx: usize, v: f32) {
        unsafe {
            (*self.0)[idx] = v;
        }
    }

    #[inline]
    pub fn read(&self, idx: usize) -> f32 { unsafe { (*self.0)[idx] } }

    #[inline]
    pub fn fill(&mut self, v: f32) {
        unsafe {
            (*self.0).fill(v);
        }
    }

    pub fn free(&self) {
        if !self.0.is_null() {
            drop(unsafe { Box::from_raw(self.0) });
        }
    }
}

impl std::fmt::Debug for ProcBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            write!(f, "ProcBuf(0: {})", (*self.0)[0])
        }
    }
}

impl std::fmt::Display for ProcBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            write!(f, "ProcBuf(0: {})", (*self.0)[0])
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum UIParamDesc {
    Knob    { width: usize, prec: usize, unit: &'static str },
    Setting { labels: &'static [&'static str], unit: &'static str },
}


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
    Osc,
    Mod,
    NtoM,
    Signal,
    CV,
    IOUtil,
}

// A note about the input-indicies:
//
// Atoms and Input parameters share the same global ID space
// because thats how the client of the Matrix API needs to refer to
// them. Beyond the Matrix API the atom data is actually split apart
// from the parameters, because they are not smoothed.
//
// The index there only matters for addressing the atoms in the global atom vector.
//
// But the actually second index here is for referring to the atom index
// relative to the absolute count of atom data a Node has.
// It is used by the [Matrix] to get the global ParamId for the atom data
// when iterating through the atoms of a Node and initializes the default data
// for new nodes.
macro_rules! node_list {
    ($inmacro: ident) => {
        $inmacro!{
            nop => Nop,
            amp => Amp UIType::Generic UICategory::Signal
             // node_param_idx
             //   name       denorm_fun
             //        norm_fun    min  max  default
               (0 inp  n_id  d_id  0.0, 1.0, 0.0)
               (1 gain n_exp d_exp 0.0, 2.0, 1.0)
               [0 sig],
            sin => Sin UIType::Generic UICategory::Osc
               (0 freq n_pit d_pit 0.001, 1.0, 440.0)
               [0 sig],
            out => Out UIType::Generic UICategory::IOUtil
               (0  ch1  n_id  d_id  0.0, 1.0, 0.0)
               (1  ch2  n_id  d_id  0.0, 1.0, 0.0)
             // node_param_idx
             //   atom_idx
             //     name            constructor min max
             //          SAtom_Type         default value
               {2 0 mono Setting => setting(0)  0   1},
        }
    }
}

macro_rules! n_id { ($x: expr, $min: expr, $max: expr) => { $x } }
macro_rules! d_id { ($x: expr, $min: expr, $max: expr) => { $x } }

macro_rules! n_lin { ($x: expr, $min: expr, $max: expr) => {
    (($x - $min) / ($max - $min) as f32).abs()
} }
macro_rules! d_lin { ($x: expr, $min: expr, $max: expr) => {
    $min * (1.0 - $x) + $max * $x
} }

macro_rules! n_exp { ($x: expr, $min: expr, $max: expr) => {
    (($x - $min) / ($max - $min) as f32).abs().sqrt()
} }
macro_rules! d_exp { ($x: expr, $min: expr, $max: expr) => {
    { let x : f32 = $x * $x; $min * (1.0 - x) + $max * x }
} }

macro_rules! n_exp4 { ($x: expr, $min: expr, $max: expr) => {
    (($x - $min) / ($max - $min)).abs().sqrt().sqrt()
} }
macro_rules! d_exp4 { ($x: expr, $min: expr, $max: expr) => {
    { let x : f32 = $x * $x * $x * $x; $min * (1.0 - x) + $max * x }
} }

macro_rules! n_pit { ($x: expr, $min: expr, $max: expr) => {
    ((($x as f32).max(0.01) / 440.0).log2() / 10.0)
//    ((($x as f32).max(0.01) / 440.0).log2() / 5.0)
} }

macro_rules! d_pit { ($x: expr, $min: expr, $max: expr) => {
    {
        let note : f32 = ($x as f32) * 10.0;
        440.0 * (2.0_f32).powf(note)
    }
} }

impl UICategory {
    #[allow(unused_assignments)]
    pub fn get_node_ids<F: FnMut(NodeId)>(&self, mut skip: usize, mut fun: F) {
        macro_rules! make_cat_lister {
            ($s1: ident => $v1: ident,
                $($str: ident => $variant: ident
                    UIType:: $gui_type: ident
                    UICategory:: $ui_cat: ident
                    $(($in_idx: literal $para: ident
                       $n_fun: ident $d_fun: ident
                       $min: expr, $max: expr, $def: expr))*
                    $({$in_at_idx: literal $at_idx: literal $atom: ident
                       $at_type: ident => $at_fun: ident ($at_init: expr)
                       $amin: literal $amax: literal})*
                    $([$out_idx: literal $out: ident])*
                    ,)+
            ) => {
                $(if UICategory::$ui_cat == *self {
                    if skip == 0 {
                        fun(NodeId::$variant(0));
                    } else {
                        skip -= 1
                    }
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
            $(($in_idx: literal $para: ident
               $n_fun: ident $d_fun: ident
               $min: expr, $max: expr, $def: expr))*
            $({$in_at_idx: literal $at_idx: literal $atom: ident
               $at_type: ident => $at_fun: ident ($at_init: expr)
               $amin: literal $amax: literal})*
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
            $($variant((NodeId, crate::dsp::ni::$variant))),+
        }

        impl NodeInfo {
            pub fn from_node_id(nid: NodeId) -> NodeInfo {
                match nid {
                    NodeId::$v1           => NodeInfo::$v1,
                    $(NodeId::$variant(_) => NodeInfo::$variant((nid, crate::dsp::ni::$variant::new()))),+
                }
            }
        }

        #[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Eq, Ord, Hash)]
        pub struct ParamId {
            name: &'static str,
            node: NodeId,
            idx:  u8,
        }

        impl ParamId {
            pub fn node_id(&self) -> NodeId       { self.node }
            pub fn inp(&self)     -> u8           { self.idx }
            pub fn name(&self)    -> &'static str { self.name }

            pub fn is_atom(&self) -> bool {
                match self.node {
                    NodeId::$v1           => false,
                    $(NodeId::$variant(_) => {
                        match self.idx {
                            $($in_idx    => false,)*
                            $($in_at_idx => true,)*
                            _            => false,
                        }
                    }),+
                }
            }

            pub fn setting_min_max(&self) -> Option<(i64, i64)> {
                match self.node {
                    NodeId::$v1           => None,
                    $(NodeId::$variant(_) => {
                        match self.idx {
                            $($in_at_idx => Some(($amin, $amax)),)*
                            _            => None,
                        }
                    }),+
                }
            }

            pub fn as_atom_def(&self) -> SAtom {
                match self.node {
                    NodeId::$v1           => SAtom::param(0.0),
                    $(NodeId::$variant(_) => {
                        match self.idx {
                            $($in_idx    => SAtom::param(crate::dsp::norm_def::$variant::$para()),)*
                            $($in_at_idx => SAtom::$at_fun($at_init),)*
                            _            => SAtom::param(0.0),
                        }
                    }),+
                }
            }

            pub fn norm_def(&self) -> f32 {
                match self.node {
                    NodeId::$v1           => 0.0,
                    $(NodeId::$variant(_) => {
                        match self.idx {
                            $($in_idx => crate::dsp::norm_def::$variant::$para(),)*
                            _ => 0.0,
                        }
                    }),+
                }
            }

            pub fn norm(&self, v: f32) -> f32 {
                match self.node {
                    NodeId::$v1           => 0.0,
                    $(NodeId::$variant(_) => {
                        match self.idx {
                            $($in_idx => crate::dsp::norm_v::$variant::$para(v),)*
                            _ => 0.0,
                        }
                    }),+
                }
            }

            pub fn denorm(&self, v: f32) -> f32 {
                match self.node {
                    NodeId::$v1           => 0.0,
                    $(NodeId::$variant(_) => {
                        match self.idx {
                            $($in_idx => crate::dsp::denorm_v::$variant::$para(v),)*
                            _ => 0.0,
                        }
                    }),+
                }
            }
        }

        #[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Eq, Ord, Hash)]
        pub enum NodeId {
            $v1,
            $($variant(u8)),+
        }

        impl std::fmt::Display for NodeId {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    NodeId::$v1           => write!(f, "{}", stringify!($s1)),
                    $(NodeId::$variant(i) => write!(f, "{} {}", stringify!($str), i)),+
                }
            }
        }

        impl NodeId {
            pub fn to_instance(&self, instance: usize) -> NodeId {
                match self {
                    NodeId::$v1           => NodeId::$v1,
                    $(NodeId::$variant(_) => NodeId::$variant(instance as u8)),+
                }
            }

            pub fn eq_variant(&self, other: &NodeId) -> bool {
                match self {
                    NodeId::$v1           => *other == NodeId::$v1,
                    $(NodeId::$variant(_) =>
                        if let NodeId::$variant(_) = other { true }
                        else { false }),+
                }
            }

            pub fn from_node_info(ni: &NodeInfo) -> NodeId {
                match ni {
                    NodeInfo::$v1           => NodeId::$v1,
                    $(NodeInfo::$variant(_) => NodeId::$variant(0)),+
                }
            }

            pub fn name(&self) -> &'static str {
                match self {
                    NodeId::$v1           => stringify!($s1),
                    $(NodeId::$variant(_) => stringify!($str)),+
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

            /// This maps the atom index of the node to the absolute
            /// ParamId in the GUI (and in the [Matrix]).
            /// The Atom/Param duality is a bit weird because they share
            /// the same ID namespace for the UI. But in the actual
            /// backend, they are split. So the actual splitting happens
            /// in the [Matrix].
            pub fn atom_param_by_idx(&self, idx: usize) -> Option<ParamId> {
                match self {
                    NodeId::$v1           => None,
                    $(NodeId::$variant(_) => {
                        match idx {
                            $($at_idx => Some(ParamId {
                                node: *self,
                                name: stringify!($atom),
                                idx:  $in_at_idx,
                            }),)*
                            _ => None,
                        }
                    }),+
                }
            }

            pub fn inp_param_by_idx(&self, idx: usize) -> Option<ParamId> {
                match self {
                    NodeId::$v1           => None,
                    $(NodeId::$variant(_) => {
                        match idx {
                            $($in_idx => Some(ParamId {
                                node: *self,
                                name: stringify!($para),
                                idx:  $in_idx,
                            }),)*
                            _ => None,
                        }
                    }),+
                }
            }

            pub fn inp_param(&self, name: &str) -> Option<ParamId> {
                match self {
                    NodeId::$v1           => None,
                    $(NodeId::$variant(_) => {
                        match name {
                            $(stringify!($para) => Some(ParamId {
                                node: *self,
                                name: stringify!($para),
                                idx:  $in_idx,
                            }),)*
                            $(stringify!($atom) => Some(ParamId {
                                node: *self,
                                name: stringify!($atom),
                                idx:  $in_at_idx,
                            }),)*
                            _ => None,
                        }
                    }),+
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
        }

        #[allow(non_snake_case)]
        pub mod denorm_v {
            $(pub mod $variant {
                $(#[inline] pub fn $para(x: f32) -> f32 { $d_fun!(x, $min, $max) })*
            })+
        }

        #[allow(non_snake_case)]
        pub mod norm_def {
            $(pub mod $variant {
                $(#[inline] pub fn $para() -> f32 { $n_fun!($def, $min, $max) })*
            })+
        }

        #[allow(non_snake_case)]
        pub mod norm_v {
            $(pub mod $variant {
                $(#[inline] pub fn $para(v: f32) -> f32 { $n_fun!(v, $min, $max) })*
            })+
        }

        #[allow(non_snake_case)]
        pub mod denorm {
            $(pub mod $variant {
                $(#[inline] pub fn $para(buf: &crate::dsp::ProcBuf, frame: usize) -> f32 {
                    $d_fun!(buf.read(frame), $min, $max)
                })*
            })+
        }

        #[allow(non_snake_case)]
        pub mod inp_dir {
            $(pub mod $variant {
                $(#[inline] pub fn $para(inputs: &[crate::dsp::ProcBuf], frame: usize) -> f32 {
                    inputs[$in_idx].read(frame)
                })*
            })+
        }

        #[allow(non_snake_case)]
        pub mod inp {
            $(pub mod $variant {
                $(#[inline] pub fn $para(inputs: &[crate::dsp::ProcBuf]) -> &crate::dsp::ProcBuf {
                    &inputs[$in_idx]
                })*
            })+
        }

        #[allow(non_snake_case)]
        pub mod at {
            $(pub mod $variant {
                $(#[inline] pub fn $atom(atoms: &[crate::dsp::SAtom]) -> &crate::dsp::SAtom {
                    &atoms[$at_idx]
                })*
            })+
        }

        #[allow(non_snake_case)]
        pub mod out_dir {
            $(pub mod $variant {
                $(#[inline] pub fn $out(outputs: &mut [crate::dsp::ProcBuf], frame: usize, v: f32) {
                    outputs[$out_idx].write(frame, v);
                })*
            })+
        }

        #[allow(non_snake_case)]
        pub mod out {
            $(pub mod $variant {
                $(#[inline] pub fn $out(outputs: &mut [crate::dsp::ProcBuf]) -> &mut crate::dsp::ProcBuf {
                    &mut outputs[$out_idx]
                })*
            })+
        }

        mod ni {
            $(
                #[derive(Debug, Clone)]
                pub struct $variant {
                    inputs:         Vec<&'static str>,
                    atoms:          Vec<&'static str>,
                    outputs:        Vec<&'static str>,
                    input_help:     Vec<&'static str>,
                    atom_help:      Vec<&'static str>,
                    output_help:    Vec<&'static str>,
                }

                impl $variant {
                    pub fn new() -> Self {
                        Self {
                            inputs:  vec![$(stringify!($para),)*],
                            atoms:   vec![$(stringify!($atom),)*],
                            outputs: vec![$(stringify!($out),)*],

                            input_help:  vec![$(crate::dsp::$variant::$para,)*],
                            atom_help:   vec![$(crate::dsp::$variant::$atom,)*],
                            output_help: vec![$(crate::dsp::$variant::$out,)*],
                        }
                    }

                    pub fn in_name(&self, in_idx: usize) -> Option<&'static str> {
                        if let Some(s) = self.inputs.get(in_idx) {
                            Some(*s)
                        } else {
                            Some(*(self.atoms.get(in_idx)?))
                        }
                    }

                    pub fn at_name(&self, in_idx: usize) -> Option<&'static str> {
                        Some(*(self.atoms.get(in_idx)?))
                    }

                    pub fn out_name(&self, out_idx: usize) -> Option<&'static str> {
                        Some(*(self.outputs.get(out_idx)?))
                    }

                    pub fn in_help(&self, in_idx: usize) -> Option<&'static str> {
                        if let Some(s) = self.inputs.get(in_idx) {
                            Some(*s)
                        } else {
                            Some(*(self.atom_help.get(in_idx)?))
                        }
                    }

                    pub fn out_help(&self, out_idx: usize) -> Option<&'static str> {
                        Some(*(self.output_help.get(out_idx)?))
                    }

                    pub fn norm(&self, in_idx: usize, x: f32) -> f32 {
                        match in_idx {
                            $($in_idx => crate::dsp::norm_v::$variant::$para(x),)+
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
                    pub fn at_count(&self)  -> usize { self.atoms.len() }
                }
            )+
        }

        impl NodeInfo {
            pub fn from(s: &str) -> Self {
                match s {
                    stringify!($s1)    => NodeInfo::$v1,
                    $(stringify!($str) =>
                        NodeInfo::$variant(
                            (NodeId::$variant(0),
                             crate::dsp::ni::$variant::new()))),+,
                    _                  => NodeInfo::Nop,
                }
            }

            pub fn in_name(&self, idx: usize) -> Option<&'static str> {
                match self {
                    NodeInfo::$v1                 => None,
                    $(NodeInfo::$variant((_, ni)) => ni.in_name(idx)),+
                }
            }

            pub fn out_name(&self, idx: usize) -> Option<&'static str> {
                match self {
                    NodeInfo::$v1                 => None,
                    $(NodeInfo::$variant((_, ni)) => ni.out_name(idx)),+
                }
            }

            pub fn in_help(&self, idx: usize) -> Option<&'static str> {
                match self {
                    NodeInfo::$v1                 => None,
                    $(NodeInfo::$variant((_, ni)) => ni.in_help(idx)),+
                }
            }

            pub fn out_help(&self, idx: usize) -> Option<&'static str> {
                match self {
                    NodeInfo::$v1                 => None,
                    $(NodeInfo::$variant((_, ni)) => ni.out_help(idx)),+
                }
            }

            pub fn to_id(&self) -> NodeId {
                match self {
                    NodeInfo::$v1                 => NodeId::$v1,
                    $(NodeInfo::$variant((id, _)) => *id),+
                }
            }

            pub fn at_count(&self) -> usize {
                match self {
                    NodeInfo::$v1           => 0,
                    $(NodeInfo::$variant(n) => n.1.at_count()),+
                }
            }

            pub fn in_count(&self) -> usize {
                match self {
                    NodeInfo::$v1           => 0,
                    $(NodeInfo::$variant(n) => n.1.in_count()),+
                }
            }

            pub fn out_count(&self) -> usize {
                match self {
                    NodeInfo::$v1           => 0,
                    $(NodeInfo::$variant(n) => n.1.out_count()),+
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
            $(($in_idx: literal $para: ident
               $n_fun: ident $d_fun: ident
               $min: expr, $max: expr, $def: expr))*
            $({$in_at_idx: literal $at_idx: literal $atom: ident
               $at_type: ident => $at_fun: ident ($at_init: expr)
               $amin: literal $amax: literal})*
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
            pub fn to_id(&self, instance: usize) -> NodeId {
                match self {
                    Node::$v1               => NodeId::$v1,
                    $(Node::$variant { .. } => NodeId::$variant(instance as u8)),+
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
                $(($in_idx: literal $para: ident
                   $n_fun: ident $d_fun: ident
                   $min: expr, $max: expr, $def: expr))*
                $({$in_at_idx: literal $at_idx: literal $atom: ident
                   $at_type: ident => $at_fun: ident ($at_init: expr)
                   $amin: literal $amax: literal})*
                $([$out_idx: literal $out: ident])*
            ,)+
        ) => {
            match node_id {
                $(NodeId::$variant(_) => Some((
                    Node::$variant { node: $variant::new() },
                    NodeInfo::from_node_id(node_id),
                )),)+
                _ => None,
            }
        }
    }

    node_list!{make_node_factory_match}
}

impl Node {
    #[inline]
    pub fn process<T: NodeAudioContext>(
        &mut self, ctx: &mut T, atoms: &[SAtom], params: &[ProcBuf], inputs: &[ProcBuf], outputs: &mut [ProcBuf])
    {
        macro_rules! make_node_process {
            ($s1: ident => $v1: ident,
                $($str: ident => $variant: ident
                    UIType:: $gui_type: ident
                    UICategory:: $ui_cat: ident
                    $(($in_idx: literal $para: ident
                       $n_fun: ident $d_fun: ident
                       $min: expr, $max: expr, $def: expr))*
                    $({$in_at_idx: literal $at_idx: literal $atom: ident
                       $at_type: ident => $at_fun: ident ($at_init: expr)
                       $amin: literal $amax: literal})*
                    $([$out_idx: literal $out: ident])*
                ,)+
            ) => {
                match self {
                    Node::$v1 => {},
                    $(Node::$variant { node } => node.process(ctx, atoms, params, inputs, outputs),)+
                }
            }
        }

        node_list!{make_node_process}
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_node_size_staying_small() {
        assert_eq!(std::mem::size_of::<Node>(),     12);
        assert_eq!(std::mem::size_of::<NodeId>(),   2);
        assert_eq!(std::mem::size_of::<ParamId>(),  24);
    }

    #[test]
    fn check_pitch() {
        assert_eq!(d_pit!(-0.2, 0.001, 1.0).round() as i32, 110_i32);
        assert_eq!((n_pit!(110.0, 0.001, 1.0) * 100.0).round() as i32, -20_i32);
        assert_eq!(d_pit!(0.0, 0.001, 1.0).round() as i32, 440_i32);
        assert_eq!((n_pit!(440.0, 0.001, 1.0) * 100.0).round() as i32, 0_i32);
        assert_eq!(d_pit!(0.3, 0.001, 1.0).round() as i32, 3520_i32);
        assert_eq!((n_pit!(3520.0, 0.001, 1.0) * 100.0).round() as i32, 30_i32);

        for i in 1..999 {
            let x = (((i as f32) / 1000.0) - 0.5) * 2.0;
            let r = d_pit!(x, 0.001, 1.0);
            println!("x={:8.5} => {:8.5}", x, r);
            assert_eq!(
                (n_pit!(r, 0.001, 1.0) * 10000.0).round() as i32,
                (x * 10000.0).round() as i32);
        }
    }
}
