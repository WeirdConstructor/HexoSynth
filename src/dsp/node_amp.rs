use crate::nodes::NodeAudioContext;
use crate::dsp::SAtom;

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

    pub fn set_sample_rate(&mut self, srate: f32) { }
    pub fn reset(&mut self) { }

    #[inline]
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T, atoms: &[SAtom], inputs: &[f32], outputs: &mut [f32]) {
        use crate::dsp::out;
        use crate::dsp::inp;
        use crate::dsp::denorm;
        out::Amp::sig(
            outputs,
            inp::Amp::inp(inputs) * denorm::Amp::gain(inputs));
    }

    pub const inp : &'static str =
        "Amp inp\nSignal input\nRange: (-1..1)\n";
    pub const gain : &'static str =
        "Amp gain\nGain input\nRange: (0..1)\n";
    pub const sig : &'static str =
        "Amp sig\nAmplified signal output\nRange: (-1..1)\n";
}
