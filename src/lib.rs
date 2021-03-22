// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

use core::arch::x86_64::{
    _MM_FLUSH_ZERO_ON,
    _MM_SET_FLUSH_ZERO_MODE,
    _MM_GET_FLUSH_ZERO_MODE
};

pub mod nodes;
#[allow(unused_macros)]
pub mod dsp;
pub mod matrix;
pub mod cell_dir;
pub mod monitor;
pub mod matrix_repr;

pub mod ui;
mod util;

use nodes::*;
use matrix::*;

pub use cell_dir::CellDir;

use dsp::NodeId;
use serde::{Serialize, Deserialize};
use raw_window_handle::HasRawWindowHandle;

use std::rc::Rc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;


pub use baseplug::{
    ProcessContext,
    PluginContext,
    WindowOpenResult,
    PluginUI,
    Plugin,
};


baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    pub struct HexoSynthModel {
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "A1")]
        mod_a1: f32,
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "A2")]
        mod_a2: f32,
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "A3")]
        mod_a3: f32,
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "A4")]
        mod_a4: f32,
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "A5")]
        mod_a5: f32,
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "A6")]
        mod_a6: f32,

        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "B1")]
        mod_b1: f32,
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "B2")]
        mod_b2: f32,
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "B3")]
        mod_b3: f32,
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "B4")]
        mod_b4: f32,
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "B5")]
        mod_b5: f32,
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "B6")]
        mod_b6: f32,
    }
}

impl Default for HexoSynthModel {
    fn default() -> Self {
        Self {
            mod_a1: 0.0,
            mod_a2: 0.0,
            mod_a3: 0.0,
            mod_a4: 0.0,
            mod_a5: 0.0,
            mod_a6: 0.0,

            mod_b1: 0.0,
            mod_b2: 0.0,
            mod_b3: 0.0,
            mod_b4: 0.0,
            mod_b5: 0.0,
            mod_b6: 0.0,
        }
    }
}

pub struct HexoSynthShared {
    pub matrix:    Arc<Mutex<Matrix>>,
    pub node_exec: Rc<RefCell<Option<NodeExecutor>>>,
}

unsafe impl Send for HexoSynthShared {}
unsafe impl Sync for HexoSynthShared {}

impl PluginContext<HexoSynth> for HexoSynthShared {
    fn new() -> Self {

        let (node_conf, node_exec) = nodes::new_node_engine();
        let (w, h) = (8, 7);
//        let (w, h) = (15, 15);
        let mut matrix = Matrix::new(node_conf, w, h);

//        let mut i = 2;
//        for x in 0..w {
//            for y in 0..h {
//                matrix.place(x, y,
//                    Cell::empty(NodeId::Sin(i))
//                    .out(Some(0), Some(0), Some(0))
//                    .input(Some(0), Some(0), Some(0)));
//                i += 1;
//            }
//        }

        matrix.place(0, 1, Cell::empty(NodeId::Sin(0))
                           .out(Some(0), None, None));
        matrix.place(1, 0, Cell::empty(NodeId::Amp(0))
                           .out(Some(0), None, None)
                           .input(None, None, Some(0)));
        matrix.place(2, 0, Cell::empty(NodeId::Out(0))
                           .input(None, None, Some(0)));
        matrix.place(4, 4, Cell::empty(NodeId::Test(0))
                           .input(None, None, None));
        matrix.sync();


        Self {
            matrix:    Arc::new(Mutex::new(matrix)),
            node_exec: Rc::new(RefCell::new(Some(node_exec))),
        }
    }
}

pub struct HexoSynth {
}

pub struct Context<'a, 'b, 'c, 'd> {
    pub nframes:    usize,
    pub output:     &'a mut [&'b mut [f32]],
    pub input:      &'c [&'d [f32]],
}

impl<'a, 'b, 'c, 'd> nodes::NodeAudioContext for Context<'a, 'b, 'c, 'd> {
    #[inline]
    fn nframes(&self) -> usize { self.nframes }

    #[inline]
    fn output(&mut self, channel: usize, frame: usize, v: f32) {
        self.output[channel][frame] = v;
    }

    #[inline]
    fn input(&mut self, channel: usize, frame: usize) -> f32 {
        self.input[channel][frame]
    }
}

impl Plugin for HexoSynth {
    const NAME:    &'static str = "HexoSynth Modular";
    const PRODUCT: &'static str = "Hexagonal Modular Synthesizer";
    const VENDOR:  &'static str = "Weird Constructor";

    const INPUT_CHANNELS: usize = 2;
    const OUTPUT_CHANNELS: usize = 2;

    type Model = HexoSynthModel;
    type PluginContext = HexoSynthShared;

    #[inline]
    fn new(sample_rate: f32, _model: &HexoSynthModel, shared: &HexoSynthShared) -> Self {
        let mut node_exec = shared.node_exec.borrow_mut();
        let node_exec     = node_exec.as_mut().unwrap();
        node_exec.set_sample_rate(sample_rate);

        Self { }
    }

    #[inline]
    fn process(&mut self, _model: &HexoSynthModelProcess,
               ctx: &mut ProcessContext<Self>, shared: &HexoSynthShared) {

        let prev_ftz = unsafe { _MM_GET_FLUSH_ZERO_MODE() };
        unsafe {
            _MM_SET_FLUSH_ZERO_MODE(_MM_FLUSH_ZERO_ON);
        }

        let mut node_exec = shared.node_exec.borrow_mut();
        let node_exec     = node_exec.as_mut().unwrap();

        node_exec.process_graph_updates();

        let mut frames_left = ctx.nframes;
        let mut offs        = 0;

        while frames_left > 0 {
            let cur_nframes =
                if frames_left >= crate::dsp::MAX_BLOCK_SIZE {
                    crate::dsp::MAX_BLOCK_SIZE
                } else {
                    frames_left
                };

            frames_left -= cur_nframes;

            let input  = &[
                &ctx.inputs[0].buffers[0][offs..(offs + cur_nframes)],
                &ctx.inputs[0].buffers[1][offs..(offs + cur_nframes)],
            ];

            let split = ctx.outputs[0].buffers.split_at_mut(1);

            let mut output = [
                &mut ((*split.0[0])[offs..(offs + cur_nframes)]),
                &mut ((*split.1[0])[offs..(offs + cur_nframes)]),
            ];

//            let output = &mut [&mut out_a_p[offs..(offs + cur_nframes)],
//                               &mut out_b_p[offs..(offs + cur_nframes)]];
//            let input =
//                &[&in_a_p[offs..(offs + cur_nframes)],
//                  &in_b_p[offs..(offs + cur_nframes)]];

            let mut context =
                Context {
                    nframes: cur_nframes,
                    output: &mut output[..],
                    input,
                };

            for i in 0..context.nframes {
                context.output[0][i] = 0.0;
                context.output[1][i] = 0.0;
            }

            node_exec.process(&mut context);

//            if oversample_simulation {
//                node_exec.process(&mut context);
//                node_exec.process(&mut context);
//                node_exec.process(&mut context);
//            }

            offs += cur_nframes;
        }

        unsafe {
            _MM_SET_FLUSH_ZERO_MODE(prev_ftz);
        }
    }
}

use hexotk::*;
use dsp::ParamId;

pub struct HexoSynthUIParams {
    params:     HashMap<AtomId, (ParamId, Atom)>,
    matrix:     Arc<Mutex<Matrix>>,
    /// Generation counter, to check for matrix updates.
    matrix_gen: RefCell<usize>,
}

impl HexoSynthUIParams {
    pub fn new(matrix: Arc<Mutex<Matrix>>) -> Self {
        let params = HashMap::new();

        let matrix_gen = matrix.lock().unwrap().get_generation();

        let mut hsup =
            HexoSynthUIParams {
                params,
                matrix_gen: RefCell::new(matrix_gen),
                matrix
            };

        hsup.sync_from_matrix();

        hsup
    }

    pub fn sync_from_matrix(&mut self) {
        let m = self.matrix.lock().unwrap();

        // TODO: this could all lead to speed problems in the UI:
        //       the allocation might cause a (too long?) pause.
        //       if this is too slow, then matrix.sync() is probably also
        //       too slow and we need to do that on an extra thread.
        //       and all communication in HexoSynthUIParams needs to happen
        //       through an Arc<Mutex<HashMap<AtomId, ...>>>.
        let mut new_hm = HashMap::new();

        m.for_each_atom(|unique_idx, param_id, satom| {
            println!("NODEID: {} => idx={}", param_id.node_id(), unique_idx);

            new_hm.insert(
                AtomId::new(unique_idx as u32, param_id.inp() as u32),
                (param_id, satom.clone().into()));
        });

        *self.matrix_gen.borrow_mut() = m.get_generation();

        self.params = new_hm;
    }

    pub fn get_param(&self, id: AtomId) -> Option<&(ParamId, Atom)> {
        self.params.get(&id)
    }

    pub fn set_param(&mut self, id: AtomId, atom: Atom) {
        let pid =
            if let Some((pid, _)) = self.params.get(&id) {
                *pid
            } else {
                return;
            };

        self.params.insert(id, (pid, atom.clone()));
        self.matrix.lock().unwrap().set_param(pid, atom.into());
    }
}

// TODO: Connect the AtomDataModel with the Matrix:
//      - Filter out NODE_MATRIX_ID requests.
//      - Map AtomId to ParamId
//      - Upon creation, read out all paramters from the Matrix
//        - Make sure the matrix is properly initialized/synced on startup.
//          So that the paramter defaults exists.
//        - retain the paramters in HexoSynthUIParams for the UI
//      - Make sure the NodeId defaults are properly loaded from dsp/mod.rs
//      - writing paramters is translated to SAtom
impl AtomDataModel for HexoSynthUIParams {
    fn len(&self) -> usize {
        self.params.len()
    }

    fn check_sync(&mut self) {
        let cur_gen = self.matrix.lock().unwrap().get_generation();
        if *self.matrix_gen.borrow() < cur_gen {
            self.sync_from_matrix();
        }
    }

    fn get(&self, id: AtomId) -> Option<&Atom> {
        Some(&self.get_param(id)?.1)
    }

    fn get_denorm(&self, id: AtomId) -> Option<f32> {
        let (pid, atom) = self.get_param(id)?;
        Some(pid.denorm(atom.f()))
    }

    fn set(&mut self, id: AtomId, v: Atom) {
        self.set_param(id, v);
    }

    fn set_default(&mut self, id: AtomId) {
        if let Some((pid, _)) = self.get_param(id) {
            let at = pid.as_atom_def().into();
            self.set(id, at);
        }
    }

    fn change_start(&mut self, id: AtomId) {
        println!("CHANGE START: {}", id);
    }

    fn change(&mut self, id: AtomId, v: f32, single: bool) {
        println!("CHANGE: {},{} ({})", id, v, single);
        if let Some((pid, _)) = self.get_param(id) {
            if let Some((min, max)) = pid.param_min_max() {
        println!("CHANGE: {},{} ({}), min={}, max={}", id, v, single, min, max);
                self.set(id, Atom::param(v.clamp(min, max)));
            }
        }
    }

    fn change_end(&mut self, id: AtomId, v: f32) {
        println!("CHANGE END: {},{}", id, v);
        if let Some((pid, _)) = self.get_param(id) {
            if let Some((min, max)) = pid.param_min_max() {
                self.set(id, Atom::param(v.clamp(min, max)));
            }
        }
    }

    fn step_next(&mut self, id: AtomId) {
        println!("STEP NEXT!!!: {}", id);

        if let Some((pid, atom)) = self.get_param(id) {
            if let Atom::Setting(i) = atom {
                if let Some((min, max)) = pid.setting_min_max() {
                    let new = i + 1;
                    let new =
                        if new > max { min }
                        else { new };

                    self.set(id, Atom::setting(new));
                }
            }
        }
    }

    fn step_prev(&mut self, id: AtomId) {
        if let Some((pid, atom)) = self.get_param(id) {
            if let Atom::Setting(i) = atom {
                if let Some((min, max)) = pid.setting_min_max() {
                    let new = i - 1;
                    let new =
                        if new < min { max }
                        else { new };

                    self.set(id, Atom::setting(new));
                }
            }
        }
    }

    fn fmt<'a>(&self, id: AtomId, buf: &'a mut [u8]) -> usize {
        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);

        if let Some((pid, atom)) = self.get_param(id) {
            if let Atom::Setting(i) = atom {
                if let Some(lbl) = pid.setting_lbl(i.abs() as usize) {
                    match write!(bw, "{}", lbl) {
                        Ok(_)  => bw.buffer().len(),
                        Err(_) => 0,
                    }
                } else {
                    match write!(bw, "{}", i) {
                        Ok(_)  => bw.buffer().len(),
                        Err(_) => 0,
                    }
                }

            } else {
                if let Some(denorm_v) = self.get_denorm(id) {
                    match write!(bw, "{:6.3}", denorm_v) {
                        Ok(_)  => bw.buffer().len(),
                        Err(_) => 0,
                    }
                } else {
                    0
                }
            }

        } else {
            0
        }
    }
}

pub const NODE_MATRIX_ID : u32 = 9999;
pub const NODE_PANEL_ID  : u32 = 9998;

impl PluginUI for HexoSynth {
    type Handle = u32;

    fn ui_size() -> (i16, i16) {
        (1400, 700)
    }

    fn ui_open(parent: &impl HasRawWindowHandle, ctx: &HexoSynthShared) -> WindowOpenResult<Self::Handle> {
        use crate::ui::matrix::NodeMatrixData;

        let matrix = ctx.matrix.clone();

        open_window("HexoSynth", 1400, 700, Some(parent.raw_window_handle()), Box::new(|| {
            Box::new(UI::new(
                Box::new(NodeMatrixData::new(matrix.clone(), UIPos::center(12, 12), NODE_MATRIX_ID)),
                Box::new(HexoSynthUIParams::new(matrix)),
                (1400 as f64, 700 as f64),
            ))
        }));

        Ok(42)
    }

    fn ui_param_notify(
        _handle: &Self::Handle,
        _param: &'static baseplug::Param<Self, <Self::Model as baseplug::Model<Self>>::Smooth>,
        _val: f32,
    ) {
    }

    fn ui_close(mut _handle: Self::Handle) {
        // TODO: Close window!
    }
}

//#[cfg(not(test))]
//baseplug::vst2!(HexoSynth, b"HxsY");
