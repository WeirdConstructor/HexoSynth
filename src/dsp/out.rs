use crate::nodes::NodeAudioContext;

/// The (stereo) output port of the plugin
#[derive(Debug, Clone)]
pub struct Out {
    /// - 0: signal channel 1
    /// - 1: signal channel 2
    input:  [f32; 2],
}

impl Out {
    pub fn outputs() -> usize { 0 }

    pub fn new() -> Self {
        Self {
            input:  [0.0; 2],
        }
    }

    pub fn set_sample_rate(&mut self, _srate: f32) { }

    #[inline]
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T, inputs: &[(usize, usize)], outinfo: &(usize, usize), out: &mut [f32]) {
        for io in inputs.iter() { self.input[io.1] = out[io.0]; }

        ctx.output(0, self.input[0]);
        ctx.output(1, self.input[1]);
    }
}
