use crate::nodes::NodeAudioContext;
use crate::dsp::{SAtom, ProcBuf};

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
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T, atoms: &[SAtom], inputs: &[ProcBuf], _outputs: &mut [ProcBuf]) {
        use crate::dsp::inp;
        use crate::dsp::at;

        if at::Out::mono(atoms).i() > 0 {
            let in1 = inp::Out::ch1(inputs);
            for frame in 0..ctx.nframes() {
                ctx.output(0, frame, in1.read(frame));
                ctx.output(1, frame, in1.read(frame));
            }
        } else {
            let in1 = inp::Out::ch1(inputs);
            let in2 = inp::Out::ch2(inputs);
            for frame in 0..ctx.nframes() {
                ctx.output(0, frame, in1.read(frame));
                ctx.output(1, frame, in2.read(frame));
            }
        }
    }

    pub const mono : &'static str =
        "Out mono\nIf enabled, ch1 will be sent to both output channels\n(UI only)";

    pub const ch1 : &'static str =
        "Out ch1\nAudio channel 1 (left)\nRange: (-1..1)";
    pub const ch2 : &'static str =
        "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";

    pub const ch3 : &'static str = "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";
    pub const ch4 : &'static str = "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";
    pub const ch5 : &'static str = "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";
    pub const ch6 : &'static str = "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";
    pub const ch7 : &'static str = "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";
    pub const ch8 : &'static str = "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";
    pub const ch9 : &'static str = "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";
    pub const ch10: &'static str = "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";
    pub const ch11: &'static str = "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";
    pub const ch12: &'static str = "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";
    pub const ch13: &'static str = "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";
    pub const ch14: &'static str = "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";
    pub const ch15: &'static str = "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";
    pub const ch16: &'static str = "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";
    pub const ch17: &'static str = "Out ch2\nAudio channel 2 (right)\nRange: (-1..1)";
}
