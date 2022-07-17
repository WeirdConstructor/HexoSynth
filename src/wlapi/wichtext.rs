// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::arg_chk;
use hexotk::{WichTextData, WichTextSimpleDataStore};
use std::rc::Rc;
use wlambda::*;

#[derive(Clone)]
pub struct VValWichTextSimpleDataStore(WichTextSimpleDataStore);

impl VValWichTextSimpleDataStore {
    pub fn new() -> Self {
        Self(WichTextSimpleDataStore::new())
    }
}

impl VValUserData for VValWichTextSimpleDataStore {
    fn s(&self) -> String {
        format!("$<UI::ConnectorData>")
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> {
        Box::new(self.clone())
    }

    fn call_method(&self, key: &str, env: &mut Env) -> Result<VVal, StackAction> {
        let args = env.argv_ref();

        match key {
            "set_text" => {
                arg_chk!(args, 1, "$<UI::ConnectorData>.set_text[text]");

                self.0.set_text(env.arg(0).s_raw());
                Ok(VVal::Bol(true))
            }
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }
}

pub fn vv2wichtext_data(mut v: VVal) -> Option<Rc<dyn WichTextData>> {
    v.with_usr_ref(|data: &mut VValWichTextSimpleDataStore| {
        let ret: Rc<dyn WichTextData> = Rc::new(data.0.clone());
        ret
    })
}
