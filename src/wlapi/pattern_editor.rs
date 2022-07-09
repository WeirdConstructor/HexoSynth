// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

//use crate::arg_chk;
use wlambda::*;
pub use hexotk::{
    UIPatternModel, PatternData,
    PatternEditorFeedback, PatternEditorFeedbackDummy
};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct VVPatEditFb(Arc<Mutex<dyn PatternEditorFeedback>>);

impl VVPatEditFb {
    pub fn new_vv(fb: Arc<Mutex<dyn PatternEditorFeedback>>) -> VVal {
        VVal::new_usr(VVPatEditFb(fb))
    }

    pub fn new_dummy() -> Self {
        Self(Arc::new(Mutex::new(PatternEditorFeedbackDummy::new())))
    }
}

impl VValUserData for VVPatEditFb {
    fn s(&self) -> String { format!("$<UI::PatEditFb>") }
    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }

    fn call_method(&self, key: &str, _env: &mut Env)
        -> Result<VVal, StackAction>
    {
        match key {
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }
}

pub fn vv2pat_edit_feedback(mut v: VVal) -> Option<Arc<Mutex<dyn PatternEditorFeedback>>> {
    v.with_usr_ref(|data: &mut VVPatEditFb| { data.0.clone() })
}

#[derive(Clone)]
pub struct VVPatModel(Arc<Mutex<dyn UIPatternModel>>);

impl VVPatModel {
    pub fn new_vv(fb: Arc<Mutex<dyn UIPatternModel>>) -> VVal {
        VVal::new_usr(VVPatModel(fb))
    }

    pub fn new_unconnected(max_rows: usize) -> Self {
        Self(Arc::new(Mutex::new(PatternData::new(max_rows))))
    }
}

impl VValUserData for VVPatModel {
    fn s(&self) -> String { format!("$<UI::PatModel>") }
    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }

    fn call_method(&self, key: &str, _env: &mut Env)
        -> Result<VVal, StackAction>
    {
        match key {
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }
}

pub fn vv2pat_model(mut v: VVal) -> Option<Arc<Mutex<dyn UIPatternModel>>> {
    v.with_usr_ref(|data: &mut VVPatModel| { data.0.clone() })
}
