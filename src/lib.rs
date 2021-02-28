#![allow(incomplete_features)]
#![feature(generic_associated_types)]

pub mod nodes;
pub mod dsp;
pub mod matrix;

pub mod ui;
mod util;

use dsp::NodeId;
use serde::{Serialize, Deserialize};
use raw_window_handle::HasRawWindowHandle;
use std::rc::Rc;

use baseplug::{
    ProcessContext,
    PluginContext,
    WindowOpenResult,
    PluginUI,
    Plugin,
};


baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct HexoSynthModel {
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

/*

Requirements:

- Pre-allocated Nodes in the audio backend
  (mono voice for now)
- mod_a1 to mod_b6 are automateable from the Host
  => Sync from VST interface into backend, with smoothing
     is done by baseplug
- UI parameters for the Nodes in the audio backend
  have their fixed adresses.
  - Automated values are sent over a ring buffer to the backend
    => the backend then initializes or searches a ramp with
       that parameter id and initializes it. for the next 64 frames.
- State of Nodes in the backend is not reset until a specific reset
  button is pressed.

What I would love to have:

- No fixed amount of pre-allocated nodes
  PROBLEM 1 => This means, we can't bind UI parameters fixed anymore.
  PROBLEM 2 => State of Nodes that are in use between the Graph updates
               needs to persist.
  - Solution 1:
    - Make a globally synchronized list of nodes
        - Frontend: List of Node types in use.
            - Index in the List is the Node-ID
            - UI Parameters are stored in the Frontend-List
            - Updates for Parameters are sent automatically to the
              backend.
        - Backend:
            - Received parameters updates are converted into ramps.
    - Invariants:
        - Always send UI parameters updates AND connections
          AFTER updating the Node list in the backend.
          => Can only do this using a ring buffer with a command queue
            COMMANDS:
                - Create Node with <type> at <idx>
                  with default values <params> from <boxed node>
                - Update Parameter <p> Node <idx> to <v> in next iteration.
                - Remove Node <idx>
                  (This creates an empty dummy node)
                - Update Eval Program <boxed prog>
          => Requires a ring buffer for feedback:
            EVENTS:
                - Removed Node <boxed node>
                - Old Program <boxed prog>
                - Feedback Trigger <node-idx> <feedback-id>

*/

use nodes::*;
use matrix::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::cell::RefCell;

struct HexoSynthShared {
    matrix:    Arc<Mutex<Matrix>>,
    node_exec: Rc<RefCell<NodeExecutor>>,
}

unsafe impl Send for HexoSynthShared {}
unsafe impl Sync for HexoSynthShared {}

impl PluginContext<HexoSynth> for HexoSynthShared {
    fn new() -> Self {
        let (mut node_conf, node_exec) = nodes::new_node_engine();
        let mut matrix = Matrix::new(node_conf, 7, 7);

        matrix.place(0, 0, Cell::empty(NodeId::Sin(0))
                           .out(None, Some(0), None));
        matrix.place(1, 0, Cell::empty(NodeId::Out(0))
                           .input(None, Some(0), None)
                           .out(None, None, Some(0)));

        matrix.place(0, 1, Cell::empty(NodeId::Sin(1))
                           .out(None, Some(0), None));
        matrix.place(1, 1, Cell::empty(NodeId::Sin(0))
                           .input(None, Some(0), None)
                           .out(None, None, Some(0)));

        let mut i = 1;
        for x in 2..5 {
            for y in 2..5 {
                matrix.place(x, y, Cell::empty(NodeId::Sin(i)));
                i += 1;
            }
        }
        matrix.sync();


        Self {
            matrix:    Arc::new(Mutex::new(matrix)),
            node_exec: Rc::new(RefCell::new(node_exec)),
        }
    }
}

struct HexoSynth {
//    matrix:     Matrix,
//    node_conf:  NodeConfigurator,
//    node_exec:  NodeExecutor,
}

pub struct Context<'a, 'b, 'c, 'd> {
    pub frame_idx:  usize,
    pub output:     &'a mut [&'b mut [f32]],
    pub input:      &'c [&'d [f32]],
}

impl<'a, 'b, 'c, 'd> nodes::NodeAudioContext for Context<'a, 'b, 'c, 'd> {
    fn output(&mut self, channel: usize, v: f32) {
        self.output[channel][self.frame_idx] = v;
    }

    fn input(&mut self, channel: usize) -> f32 {
        self.input[channel][self.frame_idx]
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
//        let (mut node_conf, node_exec) = nodes::new_node_engine(sample_rate);

        let mut node_exec = shared.node_exec.borrow_mut();
        node_exec.set_sample_rate(sample_rate);

        Self { }
    }

    #[inline]
    fn process(&mut self, model: &HexoSynthModelProcess,
               ctx: &mut ProcessContext<Self>, shared: &HexoSynthShared) {

        let input  = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        let mut node_exec = shared.node_exec.borrow_mut();

        node_exec.process_graph_updates();

        let mut context = Context {
            frame_idx: 0,
            output,
            input,
        };

        for i in 0..ctx.nframes {
            context.frame_idx    = i;
            context.output[0][i] = 0.0;
            context.output[1][i] = 0.0;

            node_exec.process(&mut context);
        }
    }
}

use hexotk::*;
use hexotk::widgets::*;

struct HexoSynthUIParams {
    params: [f32; 100],
}

impl Parameters for HexoSynthUIParams {
    fn len(&self) -> usize { self.params.len() }
    fn get(&self, id: ParamID) -> f32 { self.params[id.param_id() as usize] }
    fn get_denorm(&self, id: ParamID) -> f32 { self.params[id.param_id() as usize] }
    fn set(&mut self, id: ParamID, v: f32) { self.params[id.param_id() as usize] = v; }
    fn set_default(&mut self, id: ParamID) {
        self.set(id, 0.0);
    }

    fn change_start(&mut self, id: ParamID) {
//        println!("CHANGE START: {}", id);
    }

    fn change(&mut self, id: ParamID, v: f32, single: bool) {
//        println!("CHANGE: {},{} ({})", id, v, single);
        self.set(id, v);
    }

    fn change_end(&mut self, id: ParamID, v: f32) {
//        println!("CHANGE END: {},{}", id, v);
        self.set(id, v);
    }

    fn step_next(&mut self, id: ParamID) {
        self.set(id, (self.get(id) + 0.2).fract());
    }

    fn step_prev(&mut self, id: ParamID) {
        self.set(id, ((self.get(id) - 0.2) + 1.0).fract());
    }

    fn fmt<'a>(&self, id: ParamID, buf: &'a mut [u8]) -> usize {
        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);
        match write!(bw, "{:6.3}", self.get_denorm(id)) {
            Ok(_)  => bw.buffer().len(),
            Err(_) => 0,
        }
    }
}


impl PluginUI for HexoSynth {
    type Handle = u32;

    fn ui_size() -> (i16, i16) {
        (800, 800)
    }

    fn ui_open(parent: &impl HasRawWindowHandle, ctx: &HexoSynthShared) -> WindowOpenResult<Self::Handle> {
//        use hexotk::components::matrix::NodeMatrixData;
        use crate::ui::matrix::NodeMatrixData;

        let matrix = ctx.matrix.clone();

        open_window("HexoSynth", 800, 800, Some(parent.raw_window_handle()), Box::new(|| {
            let wt_btn      = Rc::new(Button::new(80.0, 10.0));
//            let wt_hexgrid  = Rc::new(HexGrid::new(14.0, 10.0));
//            let wt_knob     = Rc::new(Knob::new(30.0, 10.0, 10.0));
//            let wt_cont     = Rc::new(Container::new());

//            let mut node_ctrls = ContainerData::new();
//            node_ctrls.new_row()
//               .add(wt_btn,          1.into(), UIPos::right( 6, 6), ButtonData::new_toggle("Test Btn"))
//               .add(wt_knob.clone(), 2.into(), UIPos::center(3, 6), KnobData::new())
//               .add(wt_knob.clone(), 2.into(), UIPos::center(3, 6), KnobData::new())
//               .new_row()
//               .add(wt_knob.clone(), 4.into(), UIPos::center(3, 6), KnobData::new())
//               .add(wt_knob.clone(), 5.into(), UIPos::center(3, 6), KnobData::new())
//               .add(wt_knob.clone(), 6.into(), UIPos::center(3, 6), KnobData::new())
//               .add(wt_knob.clone(), 7.into(), UIPos::center(3, 6), KnobData::new());

//            let mut con = ContainerData::new();
//            con.new_row()
//               .add_direct(NodeMatrixData::new(UIPos::center(7, 12), 11))
//               .add(wt_cont.clone(), 0.into(), UIPos::center(5, 12), node_ctrls);

            let mut ui = Box::new(UI::new(
                Box::new(NodeMatrixData::new(matrix, UIPos::center(12, 12), 11)),
//                WidgetData::new_box(
//                    wt_btn, 0.into(), UIPos::center(12, 12), ButtonData::new_toggle("Test Btn")),
//                WidgetData::new_box(
//                    wt_cont, 0.into(), UIPos::center(12, 12), con),
                Box::new(HexoSynthUIParams { params: [0.0; 100] }),
                (800 as f64, 800 as f64),
            ));

            ui
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

#[cfg(any(not(test), crate_type="cdylib"))]
baseplug::vst2!(HexoSynth, b"HxsY");
