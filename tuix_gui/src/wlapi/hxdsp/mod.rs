// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

pub mod node_id;
pub mod node_info;
pub mod param;
pub mod atom;

pub use node_id::*;
pub use atom::*;
pub use param::*;
pub use node_info::*;

use wlambda::*;
use hexodsp::{NodeId};

pub fn vv2node_id(v: &VVal) -> NodeId {
    let node_id = v.v_(0).with_s_ref(|s| NodeId::from_str(s));
    node_id.to_instance(v.v_i(1) as usize)
}

pub fn node_id2vv(nid: NodeId) -> VVal {
    VVal::pair(VVal::new_str(nid.name()), VVal::Int(nid.instance() as i64))
}

