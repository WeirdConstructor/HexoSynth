use hexotk::*;
use hexotk::widgets::*;
use hexosynth::ui::matrix::NodeMatrixData;
use hexosynth::*;

use std::rc::Rc;
use std::sync::Arc;


use cpal::{Data, Sample, SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};


fn start_backend(shared: Arc<HexoSynthShared>) {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");

    let mut supported_configs_range =
        device.supported_output_configs()
            .expect("error while querying configs");

    let mut config = None;
    while let Some(f) = supported_configs_range.next() {
        if f.max_sample_rate().0 > 44100
           && f.sample_format() == SampleFormat::F32
           && f.channels() == 2
        {
            println!("Config found: {:?}", f);
            config = Some(f.with_sample_rate(cpal::SampleRate(44100)));
        }
    }

    let config = config.expect("Finding some F32, 44100Hz 2 Channel config!");

    let mut node_exec = shared.node_exec.borrow_mut().take().unwrap();
    node_exec.set_sample_rate(config.sample_rate().0 as f32);

    let mut l = vec![0.0_f32; 1024];
    let mut r = vec![0.0_f32; 1024];
    let li = vec![0.0_f32; 1024];
    let ri = vec![0.0_f32; 1024];

    let mut fun = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        println!("XX");
        let mut obuf = [&mut l[..], &mut r[..]];
        let mut ibuf = [&li[..], &ri[..]];

        node_exec.process_graph_updates();

        let mut context = Context {
            frame_idx: 0,
            output: &mut obuf[..],
            input: &ibuf[..],
        };

        let nframes = data.len() / 2;

        for i in 0..nframes {
            context.frame_idx    = i;
            context.output[0][i] = 0.0;
            context.output[1][i] = 0.0;

            node_exec.process(&mut context);

            data[0] = context.output[0][i];
            data[1] = context.output[1][i];
        }
    };

    let config : cpal::StreamConfig = config.into();
    let stream = device.build_output_stream(
        &config,
        fun,
        move |err| {
            eprintln!("an error occurred on the output audio stream: {}", err);
            // react to errors here.
        },
    ).expect("Stream to be built nicely");
    stream.play().expect("The stream to play...");
    println!("Started audio!");
}

fn main() {
    let shared = Arc::new(HexoSynthShared::new());

    start_backend(shared.clone());

    let matrix = shared.matrix.clone();

    open_window("HexoTK Standalone", 1400, 700, None, Box::new(|| {

        let mut ui = Box::new(UI::new(
            Box::new(NodeMatrixData::new(matrix, UIPos::center(12, 12), 11)),
            Box::new(HexoSynthUIParams::new()),
            (1400 as f64, 700 as f64),
        ));

        ui
    }));

}
