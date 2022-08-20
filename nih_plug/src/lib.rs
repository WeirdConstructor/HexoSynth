//use atomic_float::AtomicF32;
use nih_plug::prelude::*;

use hexodsp::matrix_repr::MatrixRepr;
use hexosynth::nodes::{EventWindowing, HxMidiEvent, HxTimedEvent};
use hexosynth::*;
use std::any::Any;
//use hexodsp::*;

use std::sync::{Arc, Mutex};

use nih_plug::param::internals::PersistentField;

pub struct HexoSynthState {
    matrix: Arc<Mutex<Matrix>>,
}

impl<'a> PersistentField<'a, String> for HexoSynthState {
    fn set(&self, new_value: String) {
        let mut m = self.matrix.lock().expect("Matrix is ok");
        if let Ok(repr) = MatrixRepr::deserialize(&new_value) {
            let _ = m.from_repr(&repr);
        }
    }

    fn map<F, R>(&self, f: F) -> R
    where
        F: Fn(&String) -> R,
    {
        let mut m = self.matrix.lock().expect("Matrix is ok");
        let mut repr = m.to_repr();
        let s = repr.serialize();
        f(&s)
    }
}

pub struct HexoSynthPlug {
    params: Arc<HexoSynthPlugParams>,
    matrix: Arc<Mutex<Matrix>>,
    node_exec: Box<NodeExecutor>,
    proc_log: bool,
}

#[derive(Params)]
struct HexoSynthPlugParams {
    #[id = "a1"]
    pub a1: FloatParam,
    #[id = "a2"]
    pub a2: FloatParam,
    #[id = "a3"]
    pub a3: FloatParam,
    #[id = "a4"]
    pub a4: FloatParam,
    #[id = "a5"]
    pub a5: FloatParam,
    #[id = "a6"]
    pub a6: FloatParam,
    #[persist = "HexSta"]
    pub matrix: HexoSynthState,
}

impl hexodsp::nodes::ExternalParams for HexoSynthPlugParams {
    fn a1(&self) -> f32 { self.a1.value }
    fn a2(&self) -> f32 { self.a2.value }
    fn a3(&self) -> f32 { self.a3.value }
}

impl Default for HexoSynthPlug {
    fn default() -> Self {
        let (matrix, mut node_exec) = init_hexosynth();

        hexodsp::log::init_thread_logger("init");

        std::thread::spawn(|| loop {
            hexodsp::log::retrieve_log_messages(|name, s| {
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open("/tmp/hexosynth.log");
                if let Ok(mut file) = file {
                    let _ = writeln!(file, "{}/{}", name, s);
                }
            });

            std::thread::sleep(std::time::Duration::from_millis(100));
        });
        use hexodsp::log::log;
        use std::io::Write;

        log(|w| write!(w, "INIT").unwrap());

        let matrix = Arc::new(Mutex::new(matrix));

        let params = Arc::new(HexoSynthPlugParams::new(matrix.clone()));

        node_exec.set_external_params(params.clone());

        Self {
            matrix,
            node_exec: Box::new(node_exec),
            params,
            proc_log: false,
            //            editor_state: editor::default_state(),

            //            peak_meter_decay_weight: 1.0,
            //            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
        }
    }
}

impl HexoSynthPlugParams {
    fn new(matrix: Arc<Mutex<Matrix>>) -> Self {
        Self {
            a1: FloatParam::new("A1", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 })
               .with_smoother(SmoothingStyle::None)
               .with_step_size(0.01),
            a2: FloatParam::new("A2", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 })
               .with_smoother(SmoothingStyle::None)
               .with_step_size(0.01),
            a3: FloatParam::new("A3", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 })
               .with_smoother(SmoothingStyle::None)
               .with_step_size(0.01),
            a4: FloatParam::new("A4", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
               .with_smoother(SmoothingStyle::None)
               .with_step_size(0.01),
            a5: FloatParam::new("A5", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
               .with_smoother(SmoothingStyle::None)
               .with_step_size(0.01),
            a6: FloatParam::new("A6", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
               .with_smoother(SmoothingStyle::None)
               .with_step_size(0.01),
            matrix: HexoSynthState { matrix },
        }
    }
}

fn blip(s: &str) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("/tmp/hexosynth_1.log");
    if let Ok(mut file) = file {
        let _ = writeln!(file, "- {}", s);
    }
}

fn note_event2hxevent(event: NoteEvent) -> Option<HxTimedEvent> {
    match event {
        NoteEvent::NoteOn { timing, channel, note, velocity, .. } => {
            Some(HxTimedEvent::note_on(timing as usize, channel, note, velocity))
        }
        NoteEvent::NoteOff { timing, channel, note, velocity, .. } => {
            Some(HxTimedEvent::note_off(timing as usize, channel, note))
        }
        NoteEvent::MidiCC { timing, channel, cc, value, .. } => {
            Some(HxTimedEvent::cc(timing as usize, channel, cc, value))
        }
        NoteEvent::Choke { timing, voice_id, channel, note, .. } => {
            Some(HxTimedEvent::note_off(timing as usize, channel, note))
        }
        _ => None,
    }
}

impl Plugin for HexoSynthPlug {
    const NAME: &'static str = "HexoSynth";
    const VENDOR: &'static str = "WeirdConstructor";
    const URL: &'static str = "https://github.com/WeirdConstructor/HexoSynth";
    const EMAIL: &'static str = "weirdconstructor@gmail.com";

    const VERSION: &'static str = "0.0.1";

    const DEFAULT_INPUT_CHANNELS: u32 = 2;
    const DEFAULT_OUTPUT_CHANNELS: u32 = 2;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&self) -> Option<Box<dyn Editor>> {
        hexodsp::log::init_thread_logger("editor");
        use hexodsp::log::log;
        use std::io::Write;

        Some(Box::new(HexoSynthEditor {
            scale_factor: Arc::new(Mutex::new(1.0_f32)),
            matrix: self.matrix.clone(),
        }))
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
        use hexodsp::log::log;
        use std::io::Write;
        hexodsp::log::init_thread_logger("proc_init");
        log(|w| write!(w, "PROC INIT").unwrap());
        self.node_exec.set_sample_rate(buffer_config.sample_rate);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext,
    ) -> ProcessStatus {
        use hexodsp::log::log;
        use std::io::Write;

        let mut ev_win = EventWindowing::new();

        //        if !self.proc_log {
        ////            hexodsp::log::init_thread_logger("proc");
        //            self.proc_log = true;
        //        }
        //        return ProcessStatus::Normal;
        //        log(|w| write!(w, "P").unwrap());

        self.node_exec.process_graph_updates();

        let mut offs = 0;

        let channel_buffers = buffer.as_slice();
        let mut frames_left = if channel_buffers.len() > 0 { channel_buffers[0].len() } else { 0 };

        let mut input_bufs = [[0.0; hexodsp::dsp::MAX_BLOCK_SIZE]; 2];

        let mut cnt = 0;
        while frames_left > 0 {
            let cur_nframes = if frames_left >= hexodsp::dsp::MAX_BLOCK_SIZE {
                hexodsp::dsp::MAX_BLOCK_SIZE
            } else {
                frames_left
            };

            self.node_exec.feed_midi_events_from(|| {
                if ev_win.feed_me() {
                    let mut new_event = None;
                    while new_event.is_none() {
                        if let Some(event) = context.next_event() {
                            new_event = note_event2hxevent(event);
                            println!("NEW EV: {:?}", new_event);
                        } else {
                            return None;
                        }
                    }

                    if let Some(event) = new_event {
                        ev_win.feed(event);
                    } else {
                        return None;
                    }
                }

                ev_win.next_event_in_range(offs, cur_nframes)
            });

            //            // First we fetch all the events for the current buffer/block,
            //            // which is limited to MAX_BLOCK_SIZE. So we need to hold back events
            //            // that have not been playing yet.
            //            {
            //
            //                let mut note_buffer = self.node_exec.get_note_buffer();
            //                note_buffer.reset();
            //
            //                while let Some(event) = ev_win.next_event_in_range(context, cur_end_frame) {
            //                    note_buffer.step_to(event.timing() as usize);
            //
            //                    match event {
            //                        NoteEvent::NoteOn { channel, note, velocity, .. } => {
            //                            note_buffer.note_on(channel, note);
            //                            note_buffer.set_velocity(channel, velocity);
            //                        }
            //                        NoteEvent::NoteOff { channel, note, velocity, .. } => {
            //                            note_buffer.note_off(channel, note);
            //                            note_buffer.set_velocity(channel, velocity);
            //                        }
            //                        NoteEvent::Choke { voice_id, channel, note, .. } => {
            //                            note_buffer.note_off(channel, note);
            //                        }
            //                        _ => {}
            //                    }
            //                }
            //
            //                note_buffer.step_to(cur_nframes - 1);
            //            }

            //            log(|w| write!(w, "FRAM LEFT: {}", frames_left).unwrap());

            input_bufs[0][0..cur_nframes]
                .copy_from_slice(&channel_buffers[0][offs..(offs + cur_nframes)]);
            input_bufs[1][0..cur_nframes]
                .copy_from_slice(&channel_buffers[1][offs..(offs + cur_nframes)]);

            let input = &[&input_bufs[0][0..cur_nframes], &input_bufs[1][0..cur_nframes]];

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

            let mut context = Context { nframes: cur_nframes, output: &mut output[..], input };

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
    scale_factor: Arc<Mutex<f32>>,
    matrix: Arc<Mutex<Matrix>>,
}

struct UnsafeWindowHandle {
    hdl: HexoSynthGUIHandle,
}

impl Drop for UnsafeWindowHandle {
    fn drop(&mut self) {
        self.hdl.close();
    }
}

unsafe impl Send for UnsafeWindowHandle {}
unsafe impl Sync for UnsafeWindowHandle {}

impl Editor for HexoSynthEditor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        _context: Arc<dyn GuiContext>,
    ) -> Box<dyn Any + Send + Sync> {
        Box::new(UnsafeWindowHandle {
            hdl: open_hexosynth(Some(parent.handle), self.matrix.clone()),
        })
    }

    fn size(&self) -> (u32, u32) {
        (1400, 800)
    }

    fn set_scale_factor(&self, factor: f32) -> bool {
        let mut sf = self.scale_factor.lock().expect("Lock this for scale factor");
        *sf = factor;
        true
    }

    fn param_values_changed(&self) {}
}

impl ClapPlugin for HexoSynthPlug {
    const CLAP_ID: &'static str = "de.m8geil.hexosynth";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("A modular synthesizer plugin with hexagonal nodes");
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = Some(Self::URL);
}

impl Vst3Plugin for HexoSynthPlug {
    const VST3_CLASS_ID: [u8; 16] = *b"HxSyGuiHxTKAaAAa";
    const VST3_CATEGORIES: &'static str = "Fx|Instrument";
}

nih_export_clap!(HexoSynthPlug);
nih_export_vst3!(HexoSynthPlug);
