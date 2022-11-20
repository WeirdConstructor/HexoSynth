// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::arg_chk;
use hexotk::{ListData, ListModel};
use std::cell::RefCell;
use std::rc::Rc;
use wlambda::*;

#[derive(Clone)]
pub struct VValListData(Rc<RefCell<ListData>>);

impl VValListData {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(ListData::new())))
    }
}

impl VValUserData for VValListData {
    fn s(&self) -> String {
        format!("$<UI::ListData>")
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
            "clear" => {
                arg_chk!(args, 0, "$<UI::ListData>.clear[]");

                self.0.borrow_mut().clear();
                Ok(VVal::Bol(true))
            }
            "push" => {
                arg_chk!(args, 1, "$<UI::ListData>.push[string]");

                self.0.borrow_mut().push(env.arg(0).s_raw());
                Ok(VVal::Bol(true))
            }
            "get_selection" => {
                arg_chk!(args, 0, "$<UI::ListData>.get_selection[]");

                Ok(self
                    .0
                    .borrow_mut()
                    .selected_item()
                    .map(|v| VVal::Int(v as i64))
                    .unwrap_or(VVal::None))
            }
            "select" => {
                arg_chk!(args, 1, "$<UI::ListData>.select[index]");

                if env.arg(0).is_none() {
                    self.0.borrow_mut().deselect();
                } else {
                    self.0.borrow_mut().select(env.arg(0).i() as usize);
                }
                Ok(VVal::Bol(true))
            }
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }
}

pub fn vv2list_data(mut v: VVal) -> Option<Rc<RefCell<ListData>>> {
    if v.is_vec() {
        let mut ld = ListData::new();
        v.with_iter(|iter| {
            for (v, _) in iter {
                ld.push(v.s_raw());
            }
        });
        Some(Rc::new(RefCell::new(ld)))
    } else {
        v.with_usr_ref(|model: &mut VValListData| model.0.clone())
    }
}
