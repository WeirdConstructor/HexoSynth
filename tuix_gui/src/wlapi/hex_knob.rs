// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use wlambda::*;
use crate::hexknob::{ParamModel};
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub struct VValHexKnobModel {
    pub model: Rc<RefCell<dyn ParamModel>>,
}

impl VValUserData for VValHexKnobModel {
    fn s(&self) -> String { format!("$<UI::HexKnobModel>") }
    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

pub fn vv2hex_knob_model(mut v: VVal) -> Option<Rc<RefCell<dyn ParamModel>>> {
    v.with_usr_ref(|model: &mut VValHexKnobModel| model.model.clone())
}

