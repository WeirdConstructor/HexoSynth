use crate::nodes::NodeAudioContext;
use crate::dsp::{SAtom, ProcBuf, GraphFun, GraphAtomData};

/// A simple amplifier
#[derive(Debug, Clone)]
pub struct Test {
}

impl Test {
    pub fn outputs() -> usize { 1 }

    pub fn new() -> Self {
        Self {
        }
    }

    pub fn set_sample_rate(&mut self, _srate: f32) { }
    pub fn reset(&mut self) { }

    #[inline]
    pub fn process<T: NodeAudioContext>(
        &mut self, ctx: &mut T, _atoms: &[SAtom], _params: &[ProcBuf],
        inputs: &[ProcBuf], outputs: &mut [ProcBuf])
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

    pub fn graph_fun() -> Option<GraphFun> {
        Some(Box::new(|gd: &dyn GraphAtomData, init: bool, x: f32| -> f32 {
            x
        }))
    }

    pub const f : &'static str = "F Test";
    pub const s : &'static str = "S Test";

//    pub const gain : &'static str =
//        "Amp gain\nGain input\nRange: (0..1)\n";
//    pub const sig : &'static str =
//        "Amp sig\nAmplified signal output\nRange: (-1..1)\n";
}
