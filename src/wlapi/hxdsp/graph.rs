// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::arg_chk;
use wlambda::*;
use hexodsp::{Matrix, NodeId, ParamId, SAtom, dsp::GraphFun, dsp::GraphAtomData};
use hexotk::{GraphModel};
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

struct NodeGraphAtomData {
    matrix:     Arc<Mutex<Matrix>>,
    node_id:    NodeId,
}

impl GraphAtomData for NodeGraphAtomData {
    fn get(&self, param_idx: u32) -> Option<SAtom> { None }
    fn get_denorm(&self, param_idx: u32) -> f32 { 0.0 }
    fn get_norm(&self, param_idx: u32) -> f32 { 0.0 }
    fn get_phase(&self) -> f32 { 0.0 }
    fn get_led(&self) -> f32 { 0.0 }
}

struct NodeGraphModel {
    matrix:   Arc<Mutex<Matrix>>,
    nga_data: Box<dyn GraphAtomData>,
    fun:      Option<GraphFun>,
}

impl GraphModel for NodeGraphModel {
    fn get_generation(&self) -> u64 {
        let m = self.matrix.lock().expect("Matrix lockable");
        m.get_generation() as u64
    }
    fn f(&self, init: bool, x: f64, x_next: f64) -> f64 {
        0.0
    }
    fn vline1_pos(&self) -> Option<f64> { None }
    fn vline2_pos(&self) -> Option<f64> { None }
}

#[derive(Clone)]
pub struct VGraphModel(Rc<RefCell<dyn GraphModel>>);

impl VGraphModel {
    pub fn new(matrix: Arc<Mutex<Matrix>>, node_id: NodeId) -> Self {
        Self(Rc::new(RefCell::new(NodeGraphModel {
            nga_data: Box::new(
                NodeGraphAtomData {
                    matrix: matrix.clone(),
                    node_id: node_id.clone(),
                },
            ),
            fun: node_id.graph_fun(),
            matrix,
        })))
    }
}

impl VValUserData for VGraphModel {
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

pub fn vv2graph_model(mut v: VVal) -> Option<Rc<RefCell<dyn GraphModel>>> {
    v.with_usr_ref(|model: &mut VGraphModel| model.0.clone())
}
