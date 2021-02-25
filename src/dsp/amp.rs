use crate::nodes::NodeAudioContext;

/// A simple amplifier
#[derive(Debug, Clone)]
pub struct Amp {
    /// - 0: signal
    /// - 1: amplitude
    input:  [f32; 2],
}

impl Amp {
    pub fn outputs() -> usize { 1 }

    pub fn new() -> Self {
        Self {
            input:  [0.0; 2],
        }
    }

    pub fn set_sample_rate(&mut self, srate: f32) { }

    #[inline]
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T, inputs: &[(usize, usize)], outinfo: &(usize, usize), out: &mut [f32]) {
        for io in inputs.iter() { self.input[io.1] = out[io.0]; }
        let out = &mut out[outinfo.0..outinfo.1];

        out[0] = self.input[0] * self.input[1];
    }
}
