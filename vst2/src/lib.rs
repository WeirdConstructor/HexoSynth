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

use hexosynth::*;

use serde::{Serialize, Deserialize};
use raw_window_handle::HasRawWindowHandle;

use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::io::Write;

use keyboard_types::KeyboardEvent;

pub use hexodsp::*;

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

pub enum HostEvent {
    KeyboardEvent(KeyboardEvent),
}

pub struct HexoSynthShared {
    pub matrix:         Arc<Mutex<Matrix>>,
    pub driver_queue:   Arc<Mutex<Vec<DriverRequest>>>,
    pub node_exec:      Rc<RefCell<Option<NodeExecutor>>>,
}

unsafe impl Send for HexoSynthShared {}
unsafe impl Sync for HexoSynthShared {}

impl PluginContext<HexoSynth> for HexoSynthShared {
    fn new() -> Self {
        let (matrix, node_exec) = init_hexosynth();

        Self {
            matrix:       Arc::new(Mutex::new(matrix)),
            node_exec:    Rc::new(RefCell::new(Some(node_exec))),
            driver_queue: Arc::new(Mutex::new(vec![])),
        }
    }
}

pub struct HexoSynth;

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

        hexodsp::log::init_thread_logger("init");

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

impl PluginUI for HexoSynth {
    type Handle = u32;

    fn ui_size() -> (i16, i16) {
        (1400, 787)
    }

    fn ui_open(parent: &impl HasRawWindowHandle, ctx: &HexoSynthShared) -> WindowOpenResult<Self::Handle> {
        if hexodsp::log::init_thread_logger("ui") {
            hexodsp::log(|w| {
                let _ = write!(w, "DAW UI thread logger initialized"); });
        }

        let (mut drv, _drv_frontend) = Driver::new();
        drv.set_sync_queue(ctx.driver_queue.clone());

        open_hexosynth(
            Some(parent.raw_window_handle()),
            Some(drv),
            ctx.matrix.clone());

        Ok(42)
    }

    fn ui_param_notify(
        _handle: &Self::Handle,
        _param: &'static baseplug::Param<Self, <Self::Model as baseplug::Model<Self>>::Smooth>,
        _val: f32,
    ) {
    }

    fn ui_close(mut _handle: Self::Handle, _ctx: &HexoSynthShared) {
        // TODO: Close window!
    }

    fn ui_key_down(ctx: &HexoSynthShared, ev: KeyboardEvent) -> bool {
        hexodsp::log(|w| {
            let _ = write!(w, "VST KeyDown: {:?}", ev);
        });
        println!("VSTEVDW: {:?}", ev);
        if let Ok(mut queue) = ctx.driver_queue.lock() {
            queue.push(DriverRequest::KeyDown { key: ev.key });
        }
        true
    }

    fn ui_key_up(ctx: &HexoSynthShared, ev: KeyboardEvent) -> bool {
        hexodsp::log(|w| {
            let _ = write!(w, "VST KeyUp: {:?}", ev);
        });
        println!("VSTEVUP: {:?}", ev);
        if let Ok(mut queue) = ctx.driver_queue.lock() {
            queue.push(DriverRequest::KeyUp { key: ev.key });
        }
        true
    }
}

#[cfg(not(test))]
baseplug::vst2!(HexoSynth, b"HxsY");
