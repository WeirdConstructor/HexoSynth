// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use wlambda::*;

use hexotk::*;

#[allow(dead_code)]
mod ui;
#[allow(dead_code)]
mod cluster;
mod matrix_param_model;
mod wlapi;

mod jack;
mod synth;

use ui::*;

use wlapi::{
    atom2vv, vv2atom,
    vv2hex_knob_model, vv2hex_grid_model,
    matrix2vv,
    VValCluster,
    VValCellDir,
    cell_dir2vv,
    new_test_grid_model,
    MatrixRecorder,
    VValBlockDSPCode,
    VValBlockCodeLanguage,
    vv2block_code_model,
};

use hexodsp::{Matrix, CellDir};

use std::rc::Rc;
use std::cell::RefCell;

use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct VValSampleBuf {
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

fn btn2vval(btn: MButton) -> VVal {
    match btn {
        MButton::Right    => VVal::new_sym("right"),
        MButton::Middle   => VVal::new_sym("middle"),
        MButton::Left     => VVal::new_sym("left"),
    }
}

#[macro_export]
macro_rules! set_modfun {
    ($st: expr, $ref: ident, $fun: tt, $min: expr, $max: expr, $env: ident, $argc: ident, $b: block) => {
        {
            let $ref = $ref.clone();
            $st.fun(
                &stringify!($fun),
                move |$env: &mut Env, $argc: usize| $b, $min, $max, false);
        }
    }
}

fn setup_hx_module(matrix: Arc<Mutex<Matrix>>) -> wlambda::SymbolTable {
    let mut st = wlambda::SymbolTable::new();

    st.set(
        "hexo_consts_rs",
        VVal::new_str(std::include_str!("ui/widgets/mod.rs")));

    st.fun(
        "get_main_matrix_handle", move |_env: &mut Env, _argc: usize| {
            Ok(matrix2vv(matrix.clone()))
        }, Some(0), Some(0), false);

    st.fun(
        "new_block_language", move |_env: &mut Env, _argc: usize| {
            Ok(VValBlockCodeLanguage::create())
        }, Some(0), Some(0), false);

    st.fun(
        "new_block_code", move |env: &mut Env, argc: usize| {
            Ok(VValBlockDSPCode::create(env.arg(0)))
        }, Some(1), Some(1), false);

    st.fun(
        "new_cluster", move |_env: &mut Env, _argc: usize| {
            Ok(VVal::new_usr(VValCluster::new()))
        }, Some(0), Some(0), false);

    st.fun(
        "new_sample_buf_from", move |env: &mut Env, _argc: usize| {
            let mut v = vec![];
            env.arg(0).with_iter(|it| {
                for (s, _) in it {
                    v.push(s.f() as f32);
                }
            });

            Ok(VVal::new_usr(VValSampleBuf::from_vec(v)))
        }, Some(1), Some(1), false);

    st.fun(
        "dir", move |env: &mut Env, _argc: usize| {
            Ok(VVal::new_usr(VValCellDir::from_vval(&env.arg(0))))
        }, Some(1), Some(1), false);

    st.fun(
        "dir_edge", move |env: &mut Env, _argc: usize| {
            Ok(VVal::new_usr(VValCellDir::from_vval_edge(&env.arg(0))))
        }, Some(1), Some(1), false);

    st.fun(
        "to_atom", move |env: &mut Env, _argc: usize| {
            Ok(atom2vv(vv2atom(env.arg(0))))
        }, Some(1), Some(1), false);

    st.fun(
        "dir_path_from_to", move |env: &mut Env, _argc: usize| {
            let from = env.arg(0);
            let to   = env.arg(1);

            let path =
                CellDir::path_from_to(
                    (from.v_i(0) as usize, from.v_i(1) as usize),
                    (to.v_i(0)   as usize, to.v_i(1)   as usize));

            let pth = VVal::vec();
            for p in path.iter() {
                pth.push(cell_dir2vv(*p));
            }

            Ok(pth)
        }, Some(2), Some(2), false);

    st.fun(
        "pos_are_adjacent", move |env: &mut Env, _argc: usize| {
            let from = env.arg(0);
            let to   = env.arg(1);

            if let Some(dir) =
                CellDir::are_adjacent(
                    (from.v_i(0) as usize, from.v_i(1) as usize),
                    (to.v_i(0)   as usize, to.v_i(1)   as usize))
            {
                Ok(cell_dir2vv(dir))
            }
            else
            {
                Ok(VVal::None)
            }
        }, Some(2), Some(2), false);

    st.fun(
        "create_test_hex_grid_model", |_env: &mut Env, _argc: usize| {
            Ok(new_test_grid_model())
        }, Some(0), Some(0), false);

    st
}

fn main() {
    synth::start(move |matrix| {
        let matrix_recorder = Arc::new(MatrixRecorder::new());
        if let Ok(mut matrix) = matrix.lock() {
            matrix.set_observer(matrix_recorder.clone());
        }

        let global_env = wlambda::GlobalEnv::new_default();
        global_env.borrow_mut().set_module("hx",        setup_hx_module(matrix));
        global_env.borrow_mut().set_module("node_id",   wlapi::setup_node_id_module());

        let wl_ctx      = wlambda::EvalContext::new(global_env.clone());
        let wl_ctx      = Rc::new(RefCell::new(wl_ctx));

        match wl_ctx.borrow_mut().eval_file("wllib/main.wl") {
            Ok(_) => { },
            Err(e) => { panic!("Error in main.wl:\n{}", e); }
        }

        let init_fun =
            wl_ctx.borrow_mut().get_global_var("init")
               .expect("global 'init' function in main.wl defined");

        match wl_ctx.borrow_mut().call(&init_fun, &[]) {
            Ok(_) => {},
            Err(e) => { panic!("Error in main.wl 'init':\n{}", e); }
        }

    });
}
