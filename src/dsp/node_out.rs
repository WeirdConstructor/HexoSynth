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
    pub fn reset(&mut self) { }

    #[inline]
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T, inputs: &[f32], _outputs: &mut [f32]) {
        use crate::dsp::inp;

        ctx.output(0, inp::Out::ch1(inputs));
        ctx.output(1, inp::Out::ch2(inputs));
    }

    pub const ch1 : &'static str = "Audio channel 1 (left)";
    pub const ch2 : &'static str = "Audio channel 2 (right)";
}
