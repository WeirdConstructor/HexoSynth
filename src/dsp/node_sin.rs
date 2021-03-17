use crate::nodes::NodeAudioContext;
use crate::dsp::{SAtom, ProcBuf, DspNode, denorm, out, inp};
use crate::dsp::helpers::fast_sin;


/// A sine oscillator
#[derive(Debug, Clone)]
pub struct Sin {
    /// Sample rate
    srate: f32,
    /// Oscillator phase
    phase: f32,
}

const TWOPI : f32 = 2.0 * std::f32::consts::PI;

impl Sin {
    pub fn new() -> Self {
        Self {
            srate: 44100.0,
            phase: 0.0,
        }
    }
    pub const freq : &'static str =
        "Sin freq\nFrequency of the oscillator.\n\nRange: (-1..1)\n";
    pub const sig : &'static str =
        "Sin sig\nOscillator signal output.\n\nRange: (-1..1)\n";
}

impl DspNode for Sin {
    fn outputs() -> usize { 1 }

    fn set_sample_rate(&mut self, srate: f32) {
        self.srate = srate;
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }

    #[inline]
    fn process<T: NodeAudioContext>(
        &mut self, ctx: &mut T, _atoms: &[SAtom], _params: &[ProcBuf],
        inputs: &[ProcBuf], outputs: &mut [ProcBuf])
    {
        let o    = out::Sin::sig(outputs);
        let freq = inp::Sin::freq(inputs);
        let isr  = 1.0 / self.srate;

        for frame in 0..ctx.nframes() {
            let freq = denorm::Sin::freq(freq, frame);

            o.write(frame, fast_sin(self.phase * TWOPI));

            self.phase += freq * isr;
            self.phase = self.phase.fract();
        }
    }

    
}
