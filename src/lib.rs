#![allow(incomplete_features)]
#![feature(generic_associated_types)]

mod nodes;
mod dsp;

use serde::{Serialize, Deserialize};

use baseplug::{
    ProcessContext,
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

struct HexoSynth {
    node_conf:  NodeConfigurator,
    node_exec:  NodeExecutor,
}

struct Context<'a, 'b, 'c, 'd> {
    frame_idx:  usize,
    output: &'a mut [&'b mut [f32]],
    input:  &'c [&'d [f32]],
}

impl<'a, 'b, 'c, 'd> nodes::NodeAudioContext for Context<'a, 'b, 'c, 'd> {
    fn output(&mut self, channel: usize, v: f32) {
        self.output[self.frame_idx][channel] = v;
    }

    fn input(&mut self, channel: usize) -> f32 {
        self.input[self.frame_idx][channel]
    }
}

impl Plugin for HexoSynth {
    const NAME:    &'static str = "HexoSynth Modular";
    const PRODUCT: &'static str = "Hexagonal Modular Synthesizer";
    const VENDOR:  &'static str = "Weird Constructor";

    const INPUT_CHANNELS: usize = 2;
    const OUTPUT_CHANNELS: usize = 2;

    type Model = HexoSynthModel;

    #[inline]
    fn new(sample_rate: f32, _model: &HexoSynthModel) -> Self {
        let (mut node_conf, node_exec) = nodes::new_node_engine(sample_rate);

        let amp_id = node_conf.create_node("amp").unwrap();
        let sin_id = node_conf.create_node("sin").unwrap();

        node_conf.upload_prog(vec![
            NodeOp { idx: amp_id, calc: true, out: [
                OutOp::Transfer { out_port_idx: 0, node_idx: sin_id, dst_param_idx: 0 },
                OutOp::Nop,
                OutOp::Nop
            ] },
            NodeOp { idx: sin_id, calc: true, out: [
                OutOp::Transfer { out_port_idx: 0, node_idx: 0, dst_param_idx: 0 },
                OutOp::Nop,
                OutOp::Nop
            ] },
        ]);

        Self {
            node_conf,
            node_exec,
        }
    }

    #[inline]
    fn process(&mut self, model: &HexoSynthModelProcess,
               ctx: &mut ProcessContext<Self>) {

        let input  = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        self.node_exec.process_graph_updates();

        for i in 0..ctx.nframes {
            self.node_exec.process(&mut Context {
                frame_idx: i,
                output,
                input,
            });

//            output[0][i] = input[0][i] * model.mod_a1[i];
//            output[1][i] = input[1][i] * model.mod_a1[i];
        }
    }
}

baseplug::vst2!(HexoSynth, b"HxsY");
