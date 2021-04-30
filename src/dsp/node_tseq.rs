// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use crate::nodes::NodeAudioContext;
use crate::dsp::{SAtom, ProcBuf, DspNode, LedPhaseVals};
use crate::dsp::tracker::TrackerBackend;

/// A tracker based sequencer
#[derive(Debug)]
pub struct TSeq {
    backend:    Option<Box<TrackerBackend>>,
}

impl Clone for TSeq {
    fn clone(&self) -> Self {
        Self {
            backend: None
        }
    }
}

impl TSeq {
    pub fn new() -> Self {
        Self {
            backend: None
        }
    }

    pub fn set_backend(&mut self, backend: TrackerBackend) {
        self.backend = Some(Box::new(backend));
    }

    pub const clock : &'static str =
        "TSeq clock\nClock input\nRange: (0..1)\n";
    pub const clock_mode : &'static str =
        "TSeq clock_mode\nDefines the interepreation of the signal on the 'clock' input.\n\
         \n";
    pub const trk1 : &'static str =
        "TSeq trk1\nTrack 1 signal output\nRange: (-1..1)\n";
    pub const trk2 : &'static str =
        "TSeq trk2\nTrack 2 signal output\nRange: (-1..1)\n";
    pub const trk3 : &'static str =
        "TSeq trk3\nTrack 3 signal output\nRange: (-1..1)\n";
    pub const trk4 : &'static str =
        "TSeq trk4\nTrack 4 signal output\nRange: (-1..1)\n";
    pub const trk5 : &'static str =
        "TSeq trk5\nTrack 5 signal output\nRange: (-1..1)\n";
    pub const trk6 : &'static str =
        "TSeq trk6\nTrack 6 signal output\nRange: (-1..1)\n";
}

impl DspNode for TSeq {
    fn outputs() -> usize { 1 }

    fn set_sample_rate(&mut self, _srate: f32) { }
    fn reset(&mut self) { }

    #[inline]
    fn process<T: NodeAudioContext>(
        &mut self, ctx: &mut T, atoms: &[SAtom], _params: &[ProcBuf],
        inputs: &[ProcBuf], outputs: &mut [ProcBuf], ctx_vals: LedPhaseVals)
    {
//        use crate::dsp::{out, inp, denorm, denorm_v, inp_dir, at};

//        let gain = inp::TSeq::gain(inputs);
//        let att  = inp::TSeq::att(inputs);
//        let inp  = inp::TSeq::inp(inputs);
//        let out  = out::TSeq::sig(outputs);
//        let neg  = at::TSeq::neg_att(atoms);
//
//        let last_frame   = ctx.nframes() - 1;
//
//        let last_val =
//            if neg.i() > 0 {
//                for frame in 0..ctx.nframes() {
//                    out.write(frame,
//                        inp.read(frame)
//                        * denorm_v::Amp::att(
//                            inp_dir::Amp::att(att, frame)
//                            .max(0.0))
//                        * denorm::Amp::gain(gain, frame));
//                }
//
//                inp.read(last_frame)
//                * denorm_v::Amp::att(
//                    inp_dir::Amp::att(att, last_frame)
//                    .max(0.0))
//                * denorm::Amp::gain(gain, last_frame)
//
//            } else {
//                for frame in 0..ctx.nframes() {
//                    out.write(frame,
//                        inp.read(frame)
//                        * denorm_v::Amp::att(
//                            inp_dir::Amp::att(att, frame).abs())
//                        * denorm::Amp::gain(gain, frame));
//                }
//
//                inp.read(last_frame)
//                * denorm_v::Amp::att(
//                    inp_dir::Amp::att(att, last_frame).abs())
//                * denorm::Amp::gain(gain, last_frame)
//            };

        let last_val = 0.0;
        ctx_vals[0].set(last_val);
    }
}
