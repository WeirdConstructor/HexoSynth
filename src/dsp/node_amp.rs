use crate::nodes::NodeAudioContext;
use crate::dsp::{SAtom, ProcBuf, GraphFun};

/// A simple amplifier
#[derive(Debug, Clone)]
pub struct Amp {
}

impl Amp {
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
        use crate::dsp::out;
        use crate::dsp::inp;
        use crate::dsp::denorm;

        let gain = inp::Amp::gain(inputs);
        let inp  = inp::Amp::inp(inputs);
        let out  = out::Amp::sig(outputs);
        for frame in 0..ctx.nframes() {
            out.write(frame, inp.read(frame) * denorm::Amp::gain(gain, frame));
        }
    }

    pub fn graph_fun() -> Option<GraphFun> { None }

    pub const inp : &'static str =
        "Amp inp\nSignal input\nRange: (-1..1)\n";
    pub const gain : &'static str =
        "Amp gain\nGain input\nRange: (0..1)\n";
    pub const sig : &'static str =
        "Amp sig\nAmplified signal output\nRange: (-1..1)\n";
}
