// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use crate::nodes::NodeAudioContext;
use crate::dsp::{SAtom, ProcBuf, GraphFun, GraphAtomData, DspNode, LedValue};

/// A simple amplifier
#[derive(Debug, Clone)]
pub struct Test {
}

impl Test {
    pub fn new() -> Self {
        Self {
        }
    }
    pub const f : &'static str = "F Test";
    pub const s : &'static str = "S Test";
//  pub const gain : &'static str =
//      "Amp gain\nGain input\nRange: (0..1)\n";
//  pub const sig : &'static str =
//      "Amp sig\nAmplified signal output\nRange: (-1..1)\n";
}

impl DspNode for Test {
    fn outputs() -> usize { 1 }

    fn set_sample_rate(&mut self, _srate: f32) { }
    fn reset(&mut self) { }

    #[inline]
    fn process<T: NodeAudioContext>(
        &mut self, _ctx: &mut T, _atoms: &[SAtom], _params: &[ProcBuf],
        _inputs: &[ProcBuf], _outputs: &mut [ProcBuf], _led: &LedValue)
    {
//        use crate::dsp::out;
//        use crate::dsp::inp;
//        use crate::dsp::denorm;
//
//        let gain = inp::Test::gain(inputs);
//        let inp  = inp::Test::inp(inputs);
//        let out  = out::Test::sig(outputs);
//        for frame in 0..ctx.nframes() {
//            out.write(frame, inp.read(frame) * denorm::Test::gain(gain, frame));
//        }
    }

    fn graph_fun() -> Option<GraphFun> {
        Some(Box::new(|_gd: &dyn GraphAtomData, _init: bool, x: f32| -> f32 {
            x
        }))
    }
}
