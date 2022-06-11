// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::arg_chk;
use std::sync::{Arc, Mutex};
use hexodsp::Matrix;
use wlambda::*;

#[derive(Clone)]
pub struct VValSampleBuf {
    buf: Arc<Mutex<Vec<f32>>>,
}

impl VValSampleBuf {
    pub fn from_vec(v: Vec<f32>) -> Self {
        Self {
            buf: Arc::new(Mutex::new(v)),
        }
    }
}

impl vval::VValUserData for VValSampleBuf {
    fn s(&self) -> String {
        let size = self.buf.lock().map_or(0, |guard| guard.len());
        format!("$<SampleBuf[{}]>", size)
    }

    fn set_key(&self, key: &VVal, val: VVal) -> Result<(), StackAction> {
        let idx = key.i() as usize;

        if let Ok(mut guard) = self.buf.lock() {
            if idx < guard.len() {
                guard[idx] = val.f() as f32;
            }
        }

        Ok(())
    }

    fn get_key(&self, key: &str) -> Option<VVal> {
        let idx = key.parse::<usize>().unwrap_or(0);
        let val =
            self.buf.lock().map_or(
                None,
                |guard| guard.get(idx).copied())?;

        Some(VVal::Flt(val as f64))
    }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
            "len" => {
                arg_chk!(args, 0, "sample_buf.len[]");

                let size = self.buf.lock().map_or(0, |guard| guard.len());
                Ok(VVal::Int(size as i64))
            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

fn vv2sample_buf(mut v: VVal) -> Option<Arc<Mutex<Vec<f32>>>> {
    v.with_usr_ref(|model: &mut VValSampleBuf| model.buf.clone())
}

fn sample_buf2vv(r: Arc<Mutex<Vec<f32>>>) -> VVal {
    VVal::new_usr(VValSampleBuf { buf: r })
}

