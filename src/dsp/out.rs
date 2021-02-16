use crate::nodes::NodeAudioContext;

/// The (stereo) output port of the plugin
#[derive(Debug, Clone)]
pub struct Out {
    /// - 0: signal channel 1
    /// - 1: signal channel 2
    input:  [f32; 2],

    /// Sample rate
    srate: f32,
}

impl Out {
    pub fn outputs() -> usize { 0 }

    pub fn new(srate: f32) -> Self {
        Self {
            srate,
            input:  [0.0; 2],
        }
    }

    #[inline]
    pub fn get(&self, _idx: u8) -> f32 {
        0.0
    }

    #[inline]
    pub fn set(&mut self, idx: u8, v: f32) {
        self.input[idx as usize] = v;
    }

    #[inline]
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T, inputs: &[(usize, usize)], outinfo: &(usize, usize), out: &mut [f32]) {
        for io in inputs.iter() { self.input[io.1] = out[io.0]; }

        ctx.output(0, self.input[0]);
        ctx.output(1, self.input[1]);
    }
}
