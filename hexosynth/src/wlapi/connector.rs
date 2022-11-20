// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::arg_chk;
use hexotk::ConnectorData;
use std::cell::RefCell;
use std::rc::Rc;
use wlambda::*;

#[derive(Clone)]
pub struct VValConnectorData(Rc<RefCell<ConnectorData>>);

impl VValConnectorData {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(ConnectorData::new())))
    }
}

impl VValUserData for VValConnectorData {
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
            "clear" => {
                arg_chk!(args, 0, "$<UI::ConnectorData>.clear[]");

                self.0.borrow_mut().clear();
                Ok(VVal::Bol(true))
            }
            "add_input" => {
                arg_chk!(args, 2, "$<UI::ConnectorData>.add_input[name, active]");

                self.0.borrow_mut().add_input(env.arg(0).s_raw(), env.arg(1).b());
                Ok(VVal::Bol(true))
            }
            "add_output" => {
                arg_chk!(args, 2, "$<UI::ConnectorData>.add_output[name, active]");

                self.0.borrow_mut().add_output(env.arg(0).s_raw(), env.arg(1).b());
                Ok(VVal::Bol(true))
            }
            "set_connection" => {
                arg_chk!(args, 1, "$<UI::ConnectorData>.set_connection[$p(in_idx, out_idx)]");

                let pair = env.arg(0);
                self.0.borrow_mut().set_connection(pair.v_i(0) as usize, pair.v_i(1) as usize);
                Ok(VVal::Bol(true))
            }
            "get_connection" => {
                arg_chk!(args, 0, "$<UI::ConnectorData>.get_connection[]");

                if let Some((in_idx, out_idx)) = self.0.borrow_mut().get_connection() {
                    Ok(VVal::pair(VVal::Int(in_idx as i64), VVal::Int(out_idx as i64)))
                } else {
                    Ok(VVal::None)
                }
            }
            "clear_connection" => {
                arg_chk!(args, 0, "$<UI::ConnectorData>.clear_connection[]");

                self.0.borrow_mut().clear_connection();
                Ok(VVal::Bol(true))
            }
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }
}

pub fn vv2connector_data(mut v: VVal) -> Option<Rc<RefCell<ConnectorData>>> {
    v.with_usr_ref(|model: &mut VValConnectorData| model.0.clone())
}
