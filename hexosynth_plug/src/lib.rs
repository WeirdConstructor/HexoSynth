use nih_plug::prelude::*;

use hexodsp::matrix_repr::MatrixRepr;
use hexodsp::{DynNode1x1Context, DynamicNode1x1};
use hexosynth::nodes::{EventWindowing, HxMidiEvent, HxTimedEvent};
use hexosynth::*;
use std::any::Any;
//use hexodsp::*;

use raw_window_handle::HasRawWindowHandle;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use nih_plug::params::persist::PersistentField;

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
    #[id = "b1"]
    pub b1: FloatParam,
    #[id = "b2"]
    pub b2: FloatParam,
    #[id = "b3"]
    pub b3: FloatParam,
    #[id = "c1"]
    pub c1: FloatParam,
    #[id = "c2"]
    pub c2: FloatParam,
    #[id = "c3"]
    pub c3: FloatParam,
    #[id = "d1"]
    pub d1: FloatParam,
    #[id = "d2"]
    pub d2: FloatParam,
    #[id = "d3"]
    pub d3: FloatParam,
    #[id = "e1"]
    pub e1: FloatParam,
    #[id = "e2"]
    pub e2: FloatParam,
    #[id = "e3"]
    pub e3: FloatParam,
    #[id = "f1"]
    pub f1: FloatParam,
    #[id = "f2"]
    pub f2: FloatParam,
    #[id = "f3"]
    pub f3: FloatParam,
    #[persist = "HexSta"]
    pub matrix: HexoSynthState,
}

impl hexodsp::nodes::ExternalParams for HexoSynthPlugParams {
    fn a1(&self) -> f32 {
        self.a1.value()
    }
    fn a2(&self) -> f32 {
        self.a2.value()
    }
    fn a3(&self) -> f32 {
        self.a3.value()
    }
    fn b1(&self) -> f32 {
        self.b1.value()
    }
    fn b2(&self) -> f32 {
        self.b2.value()
    }
    fn b3(&self) -> f32 {
        self.b3.value()
    }
    fn c1(&self) -> f32 {
        self.c1.value()
    }
    fn c2(&self) -> f32 {
        self.c2.value()
    }
    fn c3(&self) -> f32 {
        self.c3.value()
    }
    fn d1(&self) -> f32 {
        self.d1.value()
    }
    fn d2(&self) -> f32 {
        self.d2.value()
    }
    fn d3(&self) -> f32 {
        self.d3.value()
    }
    fn e1(&self) -> f32 {
        self.e1.value()
    }
    fn e2(&self) -> f32 {
        self.e2.value()
    }
    fn e3(&self) -> f32 {
        self.e3.value()
    }
    fn f1(&self) -> f32 {
        self.f1.value()
    }
    fn f2(&self) -> f32 {
        self.f2.value()
    }
    fn f3(&self) -> f32 {
        self.f3.value()
    }
}

use synfx_dsp::{Comb, DCBlockFilter, DelayBuffer};

struct Proto1 {
    buf: DelayBuffer<f32>,
    comb: Comb,
    dc: DCBlockFilter<f32>,
}

impl Proto1 {
    pub fn new() -> Self {
        Self { buf: DelayBuffer::new(), comb: Comb::new(), dc: DCBlockFilter::new() }
    }
}

impl DynamicNode1x1 for Proto1 {
    fn set_sample_rate(&mut self, srate: f32) {
        self.buf.set_sample_rate(srate);
        self.comb.set_sample_rate(srate);
        self.dc.set_sample_rate(srate);
    }

    fn reset(&mut self) {}

    fn process(&mut self, input: &[f32], output: &mut [f32], ctx: &DynNode1x1Context) {
        let drywet = 0.5;

        for (i, (inp, out)) in input.iter().zip(output.iter_mut()).enumerate() {
            let del_time_ms = ctx.alpha_slice()[i].clamp(0.0, 1.0) * 2000.0;
            let fb = ctx.beta_slice()[i];
            let comb_ms = ctx.gamma_slice()[i].clamp(0.0, 1.0) * 10.0;
            let g = ctx.delta_slice()[i].clamp(-1.0, 1.0);

            let delayed = self.buf.tap_c(del_time_ms);
            let mix = delayed + inp;
            let comb_out = self.comb.next_feedforward(comb_ms, -1.0 * g, mix * fb);
            let comb_out = self.dc.next(comb_out);
            self.buf.feed(comb_out);
            *out = delayed * drywet + inp * (1.0 - drywet);
        }
    }
}

impl Default for HexoSynthPlug {
    fn default() -> Self {
        let (matrix, mut node_exec) = init_hexosynth();

        matrix.set_dynamic_node1x1(0, Box::new(Proto1::new()));

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

macro_rules! mkparam {
    ($field: ident, $name: literal) => {
        let $field = FloatParam::new($name, 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_smoother(SmoothingStyle::None)
            .with_step_size(0.01);
    };
}

impl HexoSynthPlugParams {
    fn new(matrix: Arc<Mutex<Matrix>>) -> Self {
        mkparam! {a1, "A1"}
        mkparam! {a2, "A2"}
        mkparam! {a3, "A3"}
        mkparam! {b1, "B1"}
        mkparam! {b2, "B2"}
        mkparam! {b3, "B3"}
        mkparam! {c1, "C1"}
        mkparam! {c2, "C2"}
        mkparam! {c3, "C3"}
        mkparam! {d1, "D1"}
        mkparam! {d2, "D2"}
        mkparam! {d3, "D3"}
        mkparam! {e1, "E1"}
        mkparam! {e2, "E2"}
        mkparam! {e3, "E3"}
        mkparam! {f1, "F1"}
        mkparam! {f2, "F2"}
        mkparam! {f3, "F3"}
        Self {
            a1,
            a2,
            a3,
            b1,
            b2,
            b3,
            c1,
            c2,
            c3,
            d1,
            d2,
            d3,
            e1,
            e2,
            e3,
            f1,
            f2,
            f3,
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

fn note_event2hxevent<S>(event: NoteEvent<S>) -> Option<HxTimedEvent> {
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
    type BackgroundTask = ();
    type SysExMessage = ();

    const NAME: &'static str = "HexoSynth";
    const VENDOR: &'static str = "WeirdConstructor";
    const URL: &'static str = "https://github.com/WeirdConstructor/HexoSynth";
    const EMAIL: &'static str = "weirdconstructor@gmail.com";

    const VERSION: &'static str = "0.0.2";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        hexodsp::log::init_thread_logger("editor");
        use hexodsp::log::log;
        use std::io::Write;

        Some(Box::new(HexoSynthEditor {
            scale_factor: Arc::new(Mutex::new(1.0_f32)),
            matrix: self.matrix.clone(),
            params: self.params.clone(),
            gen_counter: Arc::new(AtomicU64::new(0)),
        }))
    }

    fn initialize(
        &mut self,
        _bus_config: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
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
        context: &mut impl ProcessContext<Self>,
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

            offs += cur_nframes;
            frames_left -= cur_nframes;
        }

        ProcessStatus::Normal
    }
}

struct HexoSynthEditor {
    scale_factor: Arc<Mutex<f32>>,
    matrix: Arc<Mutex<Matrix>>,
    params: Arc<HexoSynthPlugParams>,
    gen_counter: Arc<AtomicU64>,
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

macro_rules! setup_param {
    ($self: ident, $config: ident, $context: ident, $index: expr, $ext: ident, $param: ident) => {
        $config.param_set.$ext[$index].set_counter($self.gen_counter.clone());
        $config.param_set.$ext[$index].set_getter({
            let params = $self.params.clone();
            Box::new(move || params.$param.value())
        });

        $config.param_set.$ext[$index].set_changers(
            {
                let ctx = $context.clone();
                let params = $self.params.clone();
                Box::new(move || ParamSetter::new(&*ctx).begin_set_parameter(&params.$param))
            },
            {
                let ctx = $context.clone();
                let params = $self.params.clone();
                Box::new(move |v| {
                    ParamSetter::new(&*ctx).set_parameter_normalized(&params.$param, v)
                })
            },
            {
                let ctx = $context.clone();
                let params = $self.params.clone();
                Box::new(move || ParamSetter::new(&*ctx).end_set_parameter(&params.$param))
            },
        );
    };
}

impl Editor for HexoSynthEditor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        context: Arc<dyn GuiContext>,
    ) -> Box<dyn Any + Send> {
        let mut config = OpenHexoSynthConfig::new();

        setup_param!(self, config, context, 0, a, a1);
        setup_param!(self, config, context, 1, a, a2);
        setup_param!(self, config, context, 2, a, a3);

        setup_param!(self, config, context, 0, b, b1);
        setup_param!(self, config, context, 1, b, b2);
        setup_param!(self, config, context, 2, b, b3);

        setup_param!(self, config, context, 0, c, c1);
        setup_param!(self, config, context, 1, c, c2);
        setup_param!(self, config, context, 2, c, c3);

        setup_param!(self, config, context, 0, d, d1);
        setup_param!(self, config, context, 1, d, d2);
        setup_param!(self, config, context, 2, d, d3);

        setup_param!(self, config, context, 0, e, e1);
        setup_param!(self, config, context, 1, e, e2);
        setup_param!(self, config, context, 2, e, e3);

        setup_param!(self, config, context, 0, f, f1);
        setup_param!(self, config, context, 1, f, f2);
        setup_param!(self, config, context, 2, f, f3);

        Box::new(UnsafeWindowHandle {
            hdl: open_hexosynth_with_config(
                Some(parent.raw_window_handle()),
                self.matrix.clone(),
                config,
            ),
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

    fn param_value_changed(&self, _: &str, _: f32) {}

    fn param_modulation_changed(&self, _: &str, _: f32) {}

    fn param_values_changed(&self) {
        let prev = self.gen_counter.load(Ordering::Relaxed);
        self.gen_counter.store(prev + 1, Ordering::Relaxed);
    }
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
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Instrument];
}

nih_export_clap!(HexoSynthPlug);
nih_export_vst3!(HexoSynthPlug);
