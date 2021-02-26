use hexosynth::matrix::*;
use hexosynth::nodes::new_node_engine;
use hexosynth::dsp::*;

use hound;
use num_complex::Complex;
use microfft;

macro_rules! assert_float_eq {
    ($a:expr, $b:expr) => {
        if ($a - $b).abs() > 0.0001 {
            panic!(r#"assertion failed: `(left == right)`
  left: `{:?}`,
 right: `{:?}`"#, $a, $b)
        }
    }
}

const SAMPLE_RATE : f32 = 44100.0;

fn save_wav(name: &str, buf: &[f32]) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(name, spec).unwrap();
    for s in buf.iter() {
        let amp = i16::MAX as f32;
        writer.write_sample((amp * s) as i16).unwrap();
    }
}

fn run_no_input(node_exec: &mut hexosynth::nodes::NodeExecutor, seconds: f32) -> (Vec<f32>, Vec<f32>) {
    node_exec.set_sample_rate(SAMPLE_RATE);
    node_exec.process_graph_updates();

    let nframes = (seconds * SAMPLE_RATE) as usize;

    let mut input    = vec![0.0; nframes];
    let mut output_l = vec![0.0; nframes];
    let mut output_r = vec![0.0; nframes];

    let mut context = hexosynth::Context {
        frame_idx: 0,
        output: &mut [output_l.as_mut(), output_r.as_mut()],
        input: &[input.as_ref()],
    };

    for i in 0..nframes {
        context.frame_idx = i;
        context.output[0][i] = 0.0;
        context.output[1][i] = 0.0;

        node_exec.process(&mut context);
    }

    (output_l, output_r)
}

fn calc_rms_mimax_each_ms(buf: &[f32], ms: f32) -> Vec<(f32, f32, f32)> {
    let ms_samples = ms * SAMPLE_RATE / 1000.0;
    let len_ms     = ms_samples as usize;

    let mut idx    = 0;
    let mut res    = vec![];
    loop {
        if (idx + len_ms) > buf.len() {
            break;
        }

        let mut max = -1000.0;
        let mut min = 1000.0;
        for s in buf[idx..(idx + len_ms)].iter() {
            max = s.max(max);
            min = s.min(min);
        }

        let rms : f32 =
            buf[idx..(idx + len_ms)]
                .iter()
                .map(|s: &f32| s * s).sum::<f32>()
            / ms_samples;

        res.push((rms, min, max));

        idx += len_ms;
    }

    res
}

fn fft_thres_at_ms(buf: &mut [f32], amp_thres: u32, ms_idx: f32) -> Vec<(u16, u32)> {
    let ms_sample_offs = ms_idx * (SAMPLE_RATE / 1000.0);
    let fft_nbins = 1024;
    let len = fft_nbins;

    let mut idx    = ms_sample_offs as usize;
    let mut res    = vec![];

    if (idx + len) > buf.len() {
        return res;
    }

    // Hann window:
    for (i, s) in buf[idx..(idx + len)].iter_mut().enumerate() {
        let w =
            0.5
            * (1.0 
               - ((2.0 * std::f32::consts::PI * i as f32)
                  / (fft_nbins as f32 - 1.0))
                 .cos());
        *s *= w;
    }

    let spec = microfft::real::rfft_1024(&mut buf[idx..(idx + len)]);
    let amplitudes: Vec<_> = spec.iter().map(|c| c.norm() as u32).collect();

    for (i, amp) in amplitudes.iter().enumerate() {
        if *amp > amp_thres {
            let freq = (i as f32 * SAMPLE_RATE) / fft_nbins as f32;
            res.push((freq.round() as u16, *amp));
        }
    }

    res
}

#[test]
fn check_matrix_sine() {
    let (mut node_conf, mut node_exec) = new_node_engine();
    let mut matrix = Matrix::new(node_conf, 3, 3);

    matrix.place(0, 0, Cell::empty(NodeId::Sin(2))
                       .out(None, Some(0), None));
    matrix.place(1, 0, Cell::empty(NodeId::Out(0))
                       .input(None, Some(0), None)
                       .out(None, None, Some(0)));
    matrix.sync();

    let (mut out_l, mut out_r) = run_no_input(&mut node_exec, 4.0);

    let sum_l : f32 = out_l.iter().map(|v| v.abs()).sum();
    let sum_r : f32 = out_r.iter().map(|v| v.abs()).sum();
    assert_float_eq!(sum_l, 112303.086);
    assert_float_eq!(sum_r, 0.0);

    save_wav("check_matrix_sine.wav", &out_l);

    let rms_mimax = calc_rms_mimax_each_ms(&out_l[..], 1000.0);
    for i in 0..4 {
        assert_float_eq!(rms_mimax[i].0, 0.5);
        assert_float_eq!(rms_mimax[i].1, -0.9999999);
        assert_float_eq!(rms_mimax[i].2, 0.9999999);
    }

    let fft_res = fft_thres_at_ms(&mut out_l[..], 100, 0.0);
    assert_eq!(fft_res[0], (431, 248));
    assert_eq!(fft_res[1], (474, 169));

    let fft_res = fft_thres_at_ms(&mut out_l[..], 100, 1000.0);
    assert_eq!(fft_res[0], (431, 248));
    assert_eq!(fft_res[1], (474, 169));

    let fft_res = fft_thres_at_ms(&mut out_l[..], 100, 1500.0);
    assert_eq!(fft_res[0], (431, 248));
    assert_eq!(fft_res[1], (474, 169));
}
