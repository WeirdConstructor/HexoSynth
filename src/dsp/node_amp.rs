// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use crate::nodes::NodeAudioContext;
use crate::dsp::{SAtom, ProcBuf, DspNode, LedPhaseVals};

/// A simple amplifier
#[derive(Debug, Clone)]
pub struct Amp {
}

impl Amp {
    pub fn new() -> Self {
        Self {
        }
    }
    pub const inp : &'static str =
        "Amp inp\nSignal input\nRange: (-1..1)\n";
    pub const att : &'static str =
        "Amp att\nAttenuate input. Does only attenuate the signal, not amplify it.\n\
         Use this for envelope input.\nRange: (0..1)\n";
    pub const gain : &'static str =
        "Amp gain\nGain input. This control can actually amplify the signal.\nRange: (0..1)\n";
    pub const neg_att : &'static str =
        "Amp neg\nIf this is set to 'Clip', only positive inputs to 'att' are used.\nRange: (0..1)\n";
    pub const sig : &'static str =
        "Amp sig\nAmplified signal output\nRange: (-1..1)\n";
}

impl DspNode for Amp {
    fn outputs() -> usize { 1 }

    fn set_sample_rate(&mut self, _srate: f32) { }
    fn reset(&mut self) { }

    #[inline]
    fn process<T: NodeAudioContext>(
        &mut self, ctx: &mut T, atoms: &[SAtom], _params: &[ProcBuf],
        inputs: &[ProcBuf], outputs: &mut [ProcBuf], ctx_vals: LedPhaseVals)
    {
        use crate::dsp::{out, inp, denorm, denorm_v, inp_dir, at};

        let gain = inp::Amp::gain(inputs);
        let att  = inp::Amp::att(inputs);
        let inp  = inp::Amp::inp(inputs);
        let out  = out::Amp::sig(outputs);
        let neg  = at::Amp::neg_att(atoms);

        let last_frame   = ctx.nframes() - 1;

        let last_val =
            if neg.i() > 0 {
                for frame in 0..ctx.nframes() {
                    out.write(frame,
                        inp.read(frame)
                        * denorm_v::Amp::att(
                            inp_dir::Amp::att(att, frame)
                            .max(0.0))
                        * denorm::Amp::gain(gain, frame));
                }

                inp.read(last_frame)
                * denorm_v::Amp::att(
                    inp_dir::Amp::att(att, last_frame)
                    .max(0.0))
                * denorm::Amp::gain(gain, last_frame)

            } else {
                for frame in 0..ctx.nframes() {
                    out.write(frame,
                        inp.read(frame)
                        * denorm_v::Amp::att(
                            inp_dir::Amp::att(att, frame).abs())
                        * denorm::Amp::gain(gain, frame));
                }

                inp.read(last_frame)
                * denorm_v::Amp::att(
                    inp_dir::Amp::att(att, last_frame).abs())
                * denorm::Amp::gain(gain, last_frame)
            };

        ctx_vals[0].set(last_val);
    }
}
