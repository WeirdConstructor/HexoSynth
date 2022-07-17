// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::arg_chk;
use hexodsp::{NodeId, NodeInfo};
use wlambda::*;

use std::rc::Rc;

#[derive(Clone)]
pub struct VValNodeInfo {
    node_id: NodeId,
    info: Rc<NodeInfo>,
}

impl VValNodeInfo {
    pub fn new(node_id: NodeId) -> Self {
        Self { info: Rc::new(NodeInfo::from_node_id(node_id)), node_id }
    }
}

impl vval::VValUserData for VValNodeInfo {
    fn s(&self) -> String {
        format!(
            "$<HexoDSP::NodeInfo node={}, at_cnt={}, in_cnt={}, out_cnt={}>",
            self.node_id.name(),
            self.info.at_count(),
            self.info.in_count(),
            self.info.out_count()
        )
    }

    fn call_method(&self, key: &str, env: &mut Env) -> Result<VVal, StackAction> {
        let args = env.argv_ref();

        match key {
            "help" => {
                arg_chk!(args, 0, "$<HexoDSP::NodeInfo>.help[]");
                Ok(VVal::new_str(self.info.help()))
            }
            "desc" => {
                arg_chk!(args, 0, "$<HexoDSP::NodeInfo>.desc[]");
                Ok(VVal::new_str(self.info.desc()))
            }
            "in_help" => {
                arg_chk!(args, 1, "$<HexoDSP::NodeInfo>.in_help[index]");
                if let Some(txt) = self.info.in_help(args[0].i() as usize) {
                    Ok(VVal::new_str(txt))
                } else {
                    Ok(VVal::None)
                }
            }
            //            "add_cluster_at" => {
            //                arg_chk!(args, 2, "cluster.add_cluster_at[matrix, $i(x, y)]");
            //                Ok(VVal::None)
            //            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> {
        Box::new(self.clone())
    }
}
