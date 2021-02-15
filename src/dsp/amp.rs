use crate::nodes::NodeAudioContext;

/// A simple amplifier
#[derive(Debug, Clone)]
pub struct Amp {
    /// - 0: signal
    /// - 1: amplitude
    input:  [f32; 2],

    /// - 0: signal
    output: [f32; 1],

    /// Sample rate
    srate: f32,
}

impl Amp {
    pub fn new(srate: f32) -> Self {
        Self {
            srate,
            input:  [0.0; 2],
            output: [0.0; 1],
        }
    }

    #[inline]
    pub fn get(&self, _idx: u8) -> f32 {
        self.output[0]
    }

    #[inline]
    pub fn set(&mut self, idx: u8, v: f32) {
        self.input[idx as usize] = v;
    }

    #[inline]
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T) {
        self.output[0] = self.input[0] * self.input[1];
    }
}
