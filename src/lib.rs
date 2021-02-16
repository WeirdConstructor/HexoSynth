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
    output:     &'a mut [&'b mut [f32]],
    input:      &'c [&'d [f32]],
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

    #[inline]
    fn new(sample_rate: f32, _model: &HexoSynthModel) -> Self {
        let (mut node_conf, node_exec) = nodes::new_node_engine(sample_rate);

        let amp_id = node_conf.create_node("amp").unwrap();
        let sin_id = node_conf.create_node("sin").unwrap();
        let out_id = node_conf.create_node("out").unwrap();

        let mut outlen = 0;
        let mut amp_outidxlen = (outlen, 0);
        outlen += dsp::NodeInfo::from("amp").outputs();
        amp_outidxlen.1 = outlen;

        let mut sin_outidxlen = (outlen, 0);
        outlen += dsp::NodeInfo::from("sin").outputs();
        sin_outidxlen.1 = outlen;

        let mut out_outidxlen = (outlen, 0);
        outlen += dsp::NodeInfo::from("out").outputs();
        out_outidxlen.1 = outlen;

        let mut outvec = Vec::new();
        outvec.resize(outlen, 0.0);

        println!("OUTVEC: amp={},{} sin={},{} out={},{} {:?}",
            amp_outidxlen.0,
            amp_outidxlen.1,
            sin_outidxlen.0,
            sin_outidxlen.1,
            out_outidxlen.0,
            out_outidxlen.1,
            outvec);

        let mut prog_vec = Vec::new();
        for i in 0..50 {
            prog_vec.push(NodeOp { idx: amp_id, inputs: vec![], out_idxlen: amp_outidxlen, out: vec![
                // TODO FIXME: The compiler needs to keep track which output
                //             to actually forward!
                // TODO FIXME: The compiler also needs to keep track which inputs
                //             to actually write to!
            ] });
        }

        for i in 0..50 {
            prog_vec.push(NodeOp { idx: sin_id, out_idxlen: sin_outidxlen, out: vec![
                OutOp { out_port_idx: 0, node_idx: out_id, dst_param_idx: 0 },
            ], inputs: vec![
                (amp_outidxlen.0, 0),
                (amp_outidxlen.0, 0),
                (amp_outidxlen.0, 0),
                (amp_outidxlen.0, 0),
                (amp_outidxlen.0, 0),
                (amp_outidxlen.0, 0),
                (amp_outidxlen.0, 0),
                (amp_outidxlen.0, 0),
                (amp_outidxlen.0, 0),
                (amp_outidxlen.0, 0),
                (amp_outidxlen.0, 0),
                (amp_outidxlen.0, 0),
                (amp_outidxlen.0, 0),
            ]});
        }
        prog_vec.push(NodeOp { idx: out_id, out_idxlen: out_outidxlen, out: vec![ ],
            inputs: vec![
                (sin_outidxlen.0, 0),
                (sin_outidxlen.0, 0),
                (sin_outidxlen.0, 0),
                (sin_outidxlen.0, 0),
                (sin_outidxlen.0, 0),
                (sin_outidxlen.0, 0),
                (sin_outidxlen.0, 0),
                (sin_outidxlen.0, 0),
                (sin_outidxlen.0, 0),
                (sin_outidxlen.0, 0),
                (sin_outidxlen.0, 0),
                (sin_outidxlen.0, 0),
            ]
        });

        node_conf.upload_prog(nodes::NodeProg {
            out: outvec,
            prog: prog_vec,
        });

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

        let mut context = Context {
            frame_idx: 0,
            output,
            input,
        };

        for i in 0..ctx.nframes {
            context.frame_idx    = i;
            context.output[0][i] = 0.0;
            context.output[1][i] = 0.0;

            self.node_exec.process(&mut context);
        }
    }
}

baseplug::vst2!(HexoSynth, b"HxsY");
