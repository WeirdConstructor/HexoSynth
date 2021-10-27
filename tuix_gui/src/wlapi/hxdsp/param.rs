// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::arg_chk;
use super::{vv2node_id, node_id2vv, atom2vv};
use wlambda::*;
use hexodsp::{ParamId};

#[derive(Clone)]
pub struct VValParamId {
    param: ParamId,
}

impl VValUserData for VValParamId {
    fn s(&self) -> String {
        format!(
            "$<HexoDSP::ParamId node_id={}, idx={}, name={}>",
            self.param.node_id(),
            self.param.inp(),
            self.param.name())
    }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
            "as_parts" => {
                arg_chk!(args, 0, "param_id.as_parts[]");

                Ok(VVal::pair(
                    node_id2vv(self.param.node_id()),
                    VVal::Int(self.param.inp() as i64)))
            },
            "name" => {
                arg_chk!(args, 0, "param_id.name[]");

                Ok(VVal::new_str(self.param.name()))
            },
            "default_value" => {
                arg_chk!(args, 0, "param_id.default_value[]");

                Ok(atom2vv(self.param.as_atom_def()))
            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

pub fn vv2param_id(mut v: VVal) -> Option<ParamId> {
    if let Some(pid) =
        v.with_usr_ref(|s: &mut VValParamId| s.param.clone())
    {
        return Some(pid);
    }

    let nid = vv2node_id(&v.v_(0));
    let p = v.v_(1);

    if p.is_int() {
        nid.param_by_idx(p.i() as usize)
    } else {
        p.with_s_ref(|s| nid.inp_param(s))
    }
}

pub fn param_id2vv(param: ParamId) -> VVal {
    VVal::new_usr(VValParamId { param: param })
}
