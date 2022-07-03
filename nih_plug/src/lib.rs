//use atomic_float::AtomicF32;
use nih_plug::prelude::*;

use hexosynth::*;
use std::any::Any;
//use hexodsp::*;

use std::sync::{Arc, Mutex};

struct Gain {
    params:     Arc<GainParams>,
    matrix:     Arc<Mutex<Matrix>>,
    node_exec:  Box<NodeExecutor>,
    proc_log:   bool,
}

#[derive(Params)]
struct GainParams {
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for Gain {
    fn default() -> Self {
        let (matrix, mut node_exec) = init_hexosynth();
        node_exec.no_logging();

        hexodsp::log::init_thread_logger("init");

        std::thread::spawn(|| {
            loop {
                hexodsp::log::retrieve_log_messages(|name, s| {
                    use std::io::Write;
                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open("/tmp/hexosynth.log").unwrap();
                    let _ = writeln!(file, "{}/{}", name, s);
                });

                std::thread::sleep(
                    std::time::Duration::from_millis(100));
            };
        });
        use std::io::Write;
        use hexodsp::log::log;

        log(|w| write!(w, "INIT").unwrap());

        Self {
            matrix:    Arc::new(Mutex::new(matrix)),
            node_exec: Box::new(node_exec),

            params: Arc::new(GainParams::default()),
            proc_log: false,
//            editor_state: editor::default_state(),

//            peak_meter_decay_weight: 1.0,
//            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
        }
    }
}

impl Default for GainParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                0.0,
                FloatRange::Linear {
                    min: -30.0,
                    max: 30.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_step_size(0.01)
            .with_unit(" dB"),
        }
    }
}

fn blip(s: &str) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open("/tmp/hexosynth.log").unwrap();
    let _ = writeln!(file, "- {}", s);
}

impl Plugin for Gain {
    const NAME: &'static str = "HexoSynth";
    const VENDOR: &'static str = "WeirdConstructor";
    const URL: &'static str = "https://github.com/WeirdConstructor/HexoSynth";
    const EMAIL: &'static str = "weirdconstructor@gmail.com";

    const VERSION: &'static str = "0.0.1";

    const DEFAULT_NUM_INPUTS: u32 = 2;
    const DEFAULT_NUM_OUTPUTS: u32 = 2;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&self) -> Option<Box<dyn Editor>> {
        hexodsp::log::init_thread_logger("editor");
        use std::io::Write;
        use hexodsp::log::log;

        Some(Box::new(HexoSynthEditor {
            matrix: self.matrix.clone()
        }))
//        editor::create(
//            self.params.clone(),
//            self.peak_meter.clone(),
//            self.editor_state.clone(),
//        )
    }

    fn accepts_bus_config(&self, config: &BusConfig) -> bool {
        config.num_output_channels >= 2
    }

    fn initialize(
        &mut self,
        _bus_config: &BusConfig,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext,
    ) -> bool {
        use std::io::Write;
        use hexodsp::log::log;
        hexodsp::log::init_thread_logger("proc_init");
        log(|w| write!(w, "PROC INIT").unwrap());
        self.node_exec.set_sample_rate(buffer_config.sample_rate);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext,
    ) -> ProcessStatus {
        use std::io::Write;
        use hexodsp::log::log;

        if !self.proc_log {
//            hexodsp::log::init_thread_logger("proc");
            self.proc_log = true;
        }
//        return ProcessStatus::Normal;
//        log(|w| write!(w, "P").unwrap());

        self.node_exec.process_graph_updates();

        let mut frames_left = buffer.len();
        let mut offs        = 0;

        let channel_buffers = buffer.as_slice();

        let mut input_bufs = [[0.0; hexodsp::dsp::MAX_BLOCK_SIZE]; 2];

        let mut cnt = 0;
        while frames_left > 0 {
//            log(|w| write!(w, "FRAM LEFT: {}", frames_left).unwrap());

            let cur_nframes =
                if frames_left >= hexodsp::dsp::MAX_BLOCK_SIZE {
                    hexodsp::dsp::MAX_BLOCK_SIZE
                } else {
                    frames_left
                };

            input_bufs[0].copy_from_slice(
                &channel_buffers[0][offs..(offs + cur_nframes)]);
            input_bufs[1].copy_from_slice(
                &channel_buffers[1][offs..(offs + cur_nframes)]);

            let input = &[
                &input_bufs[0][0..cur_nframes],
                &input_bufs[1][0..cur_nframes],
            ];

            let split = channel_buffers.split_at_mut(1);

            let mut output = [
                &mut ((*split.0[0])[offs..(offs + cur_nframes)]),
                &mut ((*split.1[0])[offs..(offs + cur_nframes)]),
            ];

//            let output = &mut [&mut out_a_p[offs..(offs + cur_nframes)],
//                               &mut out_b_p[offs..(offs + cur_nframes)]];
//            let input =
//                &[&in_a_p[offs..(offs + cur_nframes)],
//                  &in_b_p[offs..(offs + cur_nframes)]];

            let mut context =
                Context {
                    nframes: cur_nframes,
                    output: &mut output[..],
                    input,
                };

            context.output[0].fill(0.0);
            context.output[1].fill(0.0);

            self.node_exec.process(&mut context);

//            if oversample_simulation {
//                node_exec.process(&mut context);
//                node_exec.process(&mut context);
//                node_exec.process(&mut context);
//            }

            offs += cur_nframes;
            frames_left -= cur_nframes;

//            if cnt >= 1 {
//                return ProcessStatus::Normal;
//            }

//            cnt += 1;
        }

        ProcessStatus::Normal
    }
}

struct HexoSynthEditor {
    matrix: Arc<Mutex<Matrix>>,
}

impl Editor for HexoSynthEditor {
    fn spawn(&self, parent: ParentWindowHandle, _context: Arc<dyn GuiContext>)
        -> Box<dyn Any + Send + Sync>
    {
        open_hexosynth(Some(parent.handle), self.matrix.clone());
        Box::new(0)
    }

    fn size(&self) -> (u32, u32) {
        (1000, 800)
    }

    fn set_scale_factor(&self, factor: f32) -> bool {
        true
    }

    fn param_values_changed(&self) {
    }
}

impl ClapPlugin for Gain {
    const CLAP_ID: &'static str = "de.m8geil.hexosynth";
    const CLAP_DESCRIPTION: &'static str = "A modular synthesizer plugin with hexagonal nodes";
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
    const CLAP_MANUAL_URL: &'static str = Self::URL;
    const CLAP_SUPPORT_URL: &'static str = Self::URL;
}

impl Vst3Plugin for Gain {
    const VST3_CLASS_ID: [u8; 16] = *b"GainGuiIcedAaAAa";
    const VST3_CATEGORIES: &'static str = "Fx|Dynamics";
}

nih_export_clap!(Gain);
nih_export_vst3!(Gain);
