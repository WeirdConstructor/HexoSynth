// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use hexosynth::*;

use std::sync::Arc;
use std::sync::Mutex;

use cpal;
use anyhow;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    let (matrix, node_exec) = init_hexosynth();
    let matrix = Arc::new(Mutex::new(matrix));

    start_backend(node_exec, move || {
        open_hexosynth(None, matrix.clone());
    });
}

pub fn run<T, F: FnMut()>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut node_exec: NodeExecutor,
    mut frontend_loop: F,
) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels    = config.channels as usize;

    node_exec.set_sample_rate(sample_rate);

    let input_bufs = [[0.0; hexodsp::dsp::MAX_BLOCK_SIZE]; 2];
    let mut outputbufs = [[0.0; hexodsp::dsp::MAX_BLOCK_SIZE]; 2];

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut frames_left = data.len() / channels;

            let mut out_iter = data.chunks_mut(channels);

            node_exec.process_graph_updates();

            while frames_left > 0 {
                let cur_nframes =
                    if frames_left >= hexodsp::dsp::MAX_BLOCK_SIZE {
                        hexodsp::dsp::MAX_BLOCK_SIZE
                    } else {
                        frames_left
                    };

                let input = &[
                    &input_bufs[0][0..cur_nframes],
                    &input_bufs[1][0..cur_nframes],
                ];

                let split = outputbufs.split_at_mut(1);

                let mut output = [
                    &mut ((split.0[0])[0..cur_nframes]),
                    &mut ((split.1[0])[0..cur_nframes]),
                ];

                let mut context =
                    Context {
                        nframes: cur_nframes,
                        output: &mut output[..],
                        input,
                    };

                context.output[0].fill(0.0);
                context.output[1].fill(0.0);

                node_exec.process(&mut context);

                // This copy loop is a bit inefficient, it's likely you can
                // pass the right array slices directly into node_exec.process()
                // via the Context structure. But I was too lazy at this point
                // to figure this out. Check also the Jack example for a more
                // efficient solution.
                for i in 0..cur_nframes {
                    if let Some(frame) = out_iter.next() {
                        let mut ctx_chan = 0;
                        for sample in frame.iter_mut() {
                            let value: T =
                                cpal::Sample::from::<f32>(&context.output[ctx_chan][i]);
                            *sample = value;

                            ctx_chan += 1;
                            if ctx_chan > context.output.len() {
                                ctx_chan = context.output.len() - 1;
                            }
                        }
                    }
                }

                frames_left -= cur_nframes;
            }
        },
        err_fn,
    )?;
    stream.play()?;

    frontend_loop();

    Ok(())
}

// This function starts the CPAL backend and
// runs the audio loop with the NodeExecutor.
fn start_backend<F: FnMut()>(node_exec: NodeExecutor, frontend_loop: F) {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Finding useable audio device");
    let config = device.default_output_config().expect("A workable output config");

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32, F>(&device, &config.into(), node_exec, frontend_loop),
        cpal::SampleFormat::I16 => run::<i16, F>(&device, &config.into(), node_exec, frontend_loop),
        cpal::SampleFormat::U16 => run::<u16, F>(&device, &config.into(), node_exec, frontend_loop),
    }.expect("cpal works fine");
}

