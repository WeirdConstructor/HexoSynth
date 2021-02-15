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
    pub fn new(srate: f32) -> Self {
        Self {
            srate,
            input:  [0.0; 2],
        }
    }

    pub fn set(&mut self, idx: usize, v: f32) {
        self.input[idx] = v;
    }

    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T) {
        ctx.output(0, self.input[0]);
        ctx.output(1, self.input[1]);
    }
}
