// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

//use crate::arg_chk;
use wlambda::*;
use hexodsp::{Matrix, NodeId, SAtom, dsp::GraphFun, dsp::GraphAtomData};
use hexotk::GraphMinMaxModel;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

struct MonitorMinMaxData {
    matrix:     Arc<Mutex<Matrix>>,
    index:      usize,
}

impl GraphMinMaxModel for MonitorMinMaxData {
    fn get_generation(&self) -> u64 {
        0
    }

    fn read(&mut self, dst: &mut [(f32, f32)]) {
    }

    fn fmt_val(&mut self, buf: &mut [u8]) -> usize {
        0
    }
}

#[derive(Clone)]
pub struct VGraphMinMaxModel(Rc<RefCell<dyn GraphMinMaxModel>>);

impl VGraphMinMaxModel {
    pub fn new_monitor_model(matrix: Arc<Mutex<Matrix>>, index: usize) -> Self {
        Self(Rc::new(RefCell::new(MonitorMinMaxData {
            matrix,
            index,
        })))
    }
}

impl VValUserData for VGraphMinMaxModel {
    fn s(&self) -> String { format!("$<UI::GraphMinMaxModel>") }
    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }

    fn call_method(&self, _key: &str, _env: &mut Env)
        -> Result<VVal, StackAction>
    {
        Ok(VVal::None)
    }
}

pub fn vv2graph_minmax_model(mut v: VVal) -> Option<Rc<RefCell<dyn GraphMinMaxModel>>> {
    v.with_usr_ref(|model: &mut VGraphMinMaxModel| model.0.clone())
}
