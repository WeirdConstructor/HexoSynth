use crate::nodes::NodeAudioContext;

/// A simple amplifier
#[derive(Debug, Clone)]
pub struct Amp {
    /// - 0: signal
    /// - 1: amplitude
    input:  [f32; 2],

    /// Sample rate
    srate: f32,
}

impl Amp {
    pub fn outputs() -> usize { 1 }

    pub fn new(srate: f32) -> Self {
        Self {
            srate,
            input:  [0.0; 2],
        }
    }

    #[inline]
    pub fn set(&mut self, idx: u8, v: f32) {
        self.input[idx as usize] = v;
    }

    #[inline]
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T, out: &mut [f32]) {
        out[0] = self.input[0] * self.input[1];
    }
}
