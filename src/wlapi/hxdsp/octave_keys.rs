// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::arg_chk;
use wlambda::*;
use hexodsp::{Matrix, NodeId, ParamId, SAtom};
use hexotk::{OctaveKeysModel, DummyOctaveKeysData};
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct OctaveKeysNodeData {
    matrix:     Arc<Mutex<Matrix>>,
    node_id:    NodeId,
    param_id:   ParamId,
}

impl OctaveKeysModel for OctaveKeysNodeData {
    fn key_mask(&self) -> i64 {
        let m = self.matrix.lock().expect("matrix lockable");
        m.get_param(&self.param_id)
         .map(|a| a.i())
         .unwrap_or(0x0)
    }

    fn phase_value(&self) -> f64 {
        let m = self.matrix.lock().expect("matrix lockable");
        m.phase_value_for(&self.node_id) as f64
    }

    fn get_generation(&self) -> u64 {
        let m = self.matrix.lock().expect("matrix lockable");
        m.get_generation() as u64
    }

    fn change(&mut self, new_mask: i64) {
        let mut m = self.matrix.lock().expect("matrix lockable");
        m.set_param(self.param_id, SAtom::setting(new_mask));
    }
}

#[derive(Clone)]
pub struct VOctaveKeysModel(Rc<RefCell<dyn OctaveKeysModel>>);

impl VOctaveKeysModel {
    pub fn new(matrix: Arc<Mutex<Matrix>>, param_id: ParamId) -> Self {
        Self(Rc::new(RefCell::new(OctaveKeysNodeData {
            matrix,
            node_id: param_id.node_id(),
            param_id,
        })))
    }
}

impl VValUserData for VOctaveKeysModel {
    fn s(&self) -> String { format!("$<UI::OctaveKeysAtomBind>") }
    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
//        let args = env.argv_ref();
        Ok(VVal::None)

//        match key {
//            "clear" => {
//                arg_chk!(args, 0, "$<UI::ConnectorData>.clear[]");
//
//                self.0.borrow_mut().clear();
//                Ok(VVal::Bol(true))
//            }
//            "add_input" => {
//                arg_chk!(args, 2, "$<UI::ConnectorData>.add_input[name, active]");
//
//                self.0.borrow_mut().add_input(env.arg(0).s_raw(), env.arg(1).b());
//                Ok(VVal::Bol(true))
//            }
//            "add_output" => {
//                arg_chk!(args, 2, "$<UI::ConnectorData>.add_output[name, active]");
//
//                self.0.borrow_mut().add_output(env.arg(0).s_raw(), env.arg(1).b());
//                Ok(VVal::Bol(true))
//            }
//            "set_connection" => {
//                arg_chk!(args, 1, "$<UI::ConnectorData>.set_connection[$p(in_idx, out_idx)]");
//
//                let pair = env.arg(0);
//                self.0.borrow_mut().set_connection(
//                    pair.v_i(0) as usize,
//                    pair.v_i(1) as usize);
//                Ok(VVal::Bol(true))
//            }
//            "get_connection" => {
//                arg_chk!(args, 0, "$<UI::ConnectorData>.get_connection[]");
//
//                if let Some((in_idx, out_idx)) =
//                    self.0.borrow_mut().get_connection()
//                {
//                    Ok(VVal::pair(
//                        VVal::Int(in_idx as i64),
//                        VVal::Int(out_idx as i64)))
//                } else {
//                    Ok(VVal::None)
//                }
//            }
//            "clear_connection" => {
//                arg_chk!(args, 0, "$<UI::ConnectorData>.clear_connection[]");
//
//                self.0.borrow_mut().clear_connection();
//                Ok(VVal::Bol(true))
//            }
//            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
//        }
    }
}

pub fn vv2octave_keys_model(mut v: VVal) -> Option<Rc<RefCell<dyn OctaveKeysModel>>> {
    v.with_usr_ref(|model: &mut VOctaveKeysModel| model.0.clone())
}
