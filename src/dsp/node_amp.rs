use crate::nodes::NodeAudioContext;

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
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T, inputs: &[f32], outputs: &mut [f32]) {
        use crate::dsp::out;
        use crate::dsp::inp;
        use crate::dsp::denorm;
        out::Amp::sig(
            outputs,
            inp::Amp::sig(inputs) * denorm::Amp::gain(inputs));
    }
}
