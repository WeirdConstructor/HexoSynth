/// A simple amplifier
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

    pub fn set(&mut self, idx: usize, v: f32) {
        self.input[idx] = v;
    }

    pub fn process(&mut self) {
        let freq = self.input[0] * super::MIDI_MAX_FREQ;

        self.output[0] = (self.phase * 2.0 * std::f32::consts::PI).sin();

        self.phase += freq / self.srate;
    }
}
