use crate::nodes::NodeAudioContext;
use crate::dsp::{SAtom, ProcBuf, DspNode};

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

}