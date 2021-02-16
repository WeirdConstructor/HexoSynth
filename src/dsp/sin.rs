use crate::nodes::NodeAudioContext;

/// A sine oscillator
#[derive(Debug, Clone)]
pub struct Sin {
    /// - 0: frequency
    input:  [f32; 1],
    /// Sample rate
    srate: f32,
    /// Oscillator phase
    phase: f32,
}

impl Sin {
    pub fn outputs() -> usize { 1 }

    pub fn new(srate: f32) -> Self {
        Self {
            srate,
            input:  [0.0; 1],
            phase: 0.0,
        }
    }

    #[inline]
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T, inputs: &[(usize, usize)], outinfo: &(usize, usize), out: &mut [f32]) {
        for io in inputs.iter() { self.input[io.1] = out[io.0]; }
        let out = &mut out[outinfo.0..outinfo.1];

        let freq = self.input[0] * super::MIDI_MAX_FREQ;
        let freq = 440.0;

        out[0] = 0.2 * (self.phase * 2.0 * std::f32::consts::PI).sin();

        self.phase += freq / self.srate;
        self.phase = self.phase.fract();
    }
}
