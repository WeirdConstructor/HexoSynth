use crate::nodes::NodeAudioContext;

/// A sine oscillator
#[derive(Debug, Clone)]
pub struct Sin {
    /// - 0: frequency
    input:  [f32; 1],
    /// - 0: signal
    output: [f32; 1],
    /// Sample rate
    srate: f32,
    /// Oscillator phase
    phase: f32,
}

impl Sin {
    pub fn new(srate: f32) -> Self {
        Self {
            srate,
            input:  [0.0; 1],
            output: [0.0; 1],
            phase: 0.0,
        }
    }

    pub fn get(&self, _idx: u8) -> f32 {
        self.output[0]
    }

    pub fn set(&mut self, _idx: u8, v: f32) {
        self.input[0] = v;
    }

    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T) {
        let freq = self.input[0] * super::MIDI_MAX_FREQ;
        let freq = 4034.0;

        self.output[0] = (self.phase * 2.0 * std::f32::consts::PI).sin();

        self.phase += freq / self.srate;
        self.phase = self.phase.fract();
    }
}
