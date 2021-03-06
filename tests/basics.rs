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

enum FFT {
    F16,
    F32,
    F64,
    F128,
    F512,
    F1024,
}

fn fft_thres_at_ms(buf: &mut [f32], size: FFT, amp_thres: u32, ms_idx: f32) -> Vec<(u16, u32)> {
    let ms_sample_offs = ms_idx * (SAMPLE_RATE / 1000.0);
    let fft_nbins = match size {
        FFT::F16      => 16,
        FFT::F32      => 32,
        FFT::F64      => 64,
        FFT::F128     => 128,
        FFT::F512     => 512,
        FFT::F1024    => 1024,
    };
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

    let spec =
        match size {
            FFT::F16 =>
                microfft::real::rfft_16(&mut buf[idx..(idx + len)]),
            FFT::F32 =>
                microfft::real::rfft_32(&mut buf[idx..(idx + len)]),
            FFT::F64 =>
                microfft::real::rfft_64(&mut buf[idx..(idx + len)]),
            FFT::F128 =>
                microfft::real::rfft_128(&mut buf[idx..(idx + len)]),
            FFT::F512 =>
                microfft::real::rfft_512(&mut buf[idx..(idx + len)]),
            FFT::F1024 =>
                microfft::real::rfft_1024(&mut buf[idx..(idx + len)]),
        };
    let amplitudes: Vec<_> = spec.iter().map(|c| c.norm() as u32).collect();

    for (i, amp) in amplitudes.iter().enumerate() {
        if *amp >= amp_thres {
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

    let sin = NodeId::Sin(2);
    let out = NodeId::Out(0);
    matrix.place(0, 0, Cell::empty(sin)
                       .out(None, sin.out("sig"), None));
    matrix.place(1, 0, Cell::empty(out)
                       .input(None, out.inp("in1"), None));
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

    let fft_res = fft_thres_at_ms(&mut out_l[..], FFT::F1024, 100, 0.0);
    assert_eq!(fft_res[0], (431, 248));
    assert_eq!(fft_res[1], (474, 169));

    let fft_res = fft_thres_at_ms(&mut out_l[..], FFT::F1024, 100, 1000.0);
    assert_eq!(fft_res[0], (431, 248));
    assert_eq!(fft_res[1], (474, 169));

    let fft_res = fft_thres_at_ms(&mut out_l[..], FFT::F1024, 100, 1500.0);
    assert_eq!(fft_res[0], (431, 248));
    assert_eq!(fft_res[1], (474, 169));
}


#[test]
fn check_sine_pitch_change() {
    let (mut node_conf, mut node_exec) = new_node_engine();
    let mut matrix = Matrix::new(node_conf, 3, 3);

    let sin = NodeId::Sin(2);
    let out = NodeId::Out(0);
    matrix.place(0, 0, Cell::empty(sin)
                       .out(None, sin.out("sig"), None));
    matrix.place(1, 0, Cell::empty(out)
                       .input(None, out.inp("in1"), None));
    matrix.sync();

    let (mut out_l, mut out_r) = run_no_input(&mut node_exec, 0.2);

    let fft_res = fft_thres_at_ms(&mut out_l[..], FFT::F1024, 200, 0.0);
    assert_eq!(fft_res[0], (431, 248));

    let fft_res = fft_thres_at_ms(&mut out_l[..], FFT::F64, 20, 100.0);
    assert_eq!(fft_res[0], (0, 22));

    let freq_param = sin.inp_param("freq").unwrap();

    matrix.set_param(
        freq_param,
        freq_param.norm(4400.0));

    let (mut out_l, mut out_r) = run_no_input(&mut node_exec, 1.0);

    // Test at the start of the slope (~ 690 Hz):
    let fft_res = fft_thres_at_ms(&mut out_l[..], FFT::F64, 15, 0.0);
    assert_eq!(fft_res[0], (0, 18));
    assert_eq!(fft_res[1], (689, 15));

    // In the middle (~ 2067 Hz):
    let fft_res = fft_thres_at_ms(&mut out_l[..], FFT::F64, 10, 5.0);
    assert_eq!(fft_res[0], (1378, 14));
    assert_eq!(fft_res[1], (2067, 12));

    // Goal (~ 4134 Hz)
    let fft_res = fft_thres_at_ms(&mut out_l[..], FFT::F64, 14, 10.0);
    assert_eq!(fft_res[0], (4134, 14));

    // Test the freq after the slope in high res (closer to 4400 Hz):
    let fft_res = fft_thres_at_ms(&mut out_l[..], FFT::F1024, 200, 400.0);
    assert_eq!(fft_res[0], (4393, 251));
}

#[test]
fn check_matrix_out_config_bug1() {
    let (mut node_conf, mut node_exec) = new_node_engine();
    let mut matrix = Matrix::new(node_conf, 7, 7);

    matrix.place(0, 0, Cell::empty(NodeId::Sin(0))
                       .out(None, Some(0), None));
    matrix.place(1, 0, Cell::empty(NodeId::Out(0))
                       .input(None, Some(0), None)
                       .out(None, None, Some(0)));

    matrix.place(0, 1, Cell::empty(NodeId::Sin(1))
                       .out(None, Some(0), None));
    matrix.place(1, 2, Cell::empty(NodeId::Sin(0))
                       .input(None, Some(0), None)
                       .out(None, None, Some(0)));
    matrix.place(1, 1, Cell::empty(NodeId::Out(0))
                       .input(Some(1), Some(0), None)
                       .out(None, None, Some(0)));

    matrix.sync();

    let (mut out_l, mut out_r) = run_no_input(&mut node_exec, 0.2);
}

#[test]
fn check_matrix_out_config_bug1_reduced() {
    let (mut node_conf, mut node_exec) = new_node_engine();
    let mut matrix = Matrix::new(node_conf, 7, 7);

    matrix.place(1, 0, Cell::empty(NodeId::Out(0))
                       .input(Some(0), None, None)
                       .out(None, None, Some(0)));
    matrix.place(1, 2, Cell::empty(NodeId::Out(0))
                       .input(Some(0), None, None)
                       .out(None, None, None));

    matrix.sync();

    let (mut out_l, mut out_r) = run_no_input(&mut node_exec, 0.2);
}

#[test]
fn check_matrix_out_config_bug1b_reduced() {
    let (mut node_conf, mut node_exec) = new_node_engine();
    let mut matrix = Matrix::new(node_conf, 7, 7);

    matrix.place(1, 0, Cell::empty(NodeId::Out(0))
                       .out(None, None, Some(0)));
    matrix.place(1, 1, Cell::empty(NodeId::Out(0))
                       .input(Some(0), None, None));

    matrix.sync();

    let (mut out_l, mut out_r) = run_no_input(&mut node_exec, 0.2);
}

#[test]
fn check_matrix_out_config_bug1c_reduced() {
    let (mut node_conf, mut node_exec) = new_node_engine();
    let mut matrix = Matrix::new(node_conf, 7, 7);

    matrix.place(1, 0, Cell::empty(NodeId::Sin(0))
                       .out(None, None, Some(0)));
    matrix.place(1, 1, Cell::empty(NodeId::Out(0))
                       .input(Some(9), None, None));

    matrix.sync();

    let (mut out_l, mut out_r) = run_no_input(&mut node_exec, 0.2);
}

macro_rules! simple_sine_output_test {
    ($matrix: ident, $block: tt) => {
        let (mut node_conf, mut node_exec) = new_node_engine();
        let mut $matrix = Matrix::new(node_conf, 7, 7);

        $block;

        $matrix.sync();

        let (mut out_l, mut out_r) = run_no_input(&mut node_exec, 0.2);

        let rms_mimax = calc_rms_mimax_each_ms(&out_l[..], 50.0);
        assert_float_eq!(rms_mimax[0].0, 0.5);
        assert_float_eq!(rms_mimax[0].1, -0.9999999);
        assert_float_eq!(rms_mimax[0].2, 0.9999999);
    }
}

#[test]
fn check_matrix_connect_even_top_left() {
    simple_sine_output_test!(matrix, {
        matrix.place(1, 0, Cell::empty(NodeId::Sin(0))
                           .out(None, Some(0), None));
        matrix.place(2, 1, Cell::empty(NodeId::Out(0))
                           .input(None, Some(0), None));
    });
}


#[test]
fn check_matrix_connect_even_bottom_left() {
    simple_sine_output_test!(matrix, {
        matrix.place(1, 1, Cell::empty(NodeId::Sin(0))
                           .out(Some(0), None, None));
        matrix.place(2, 1, Cell::empty(NodeId::Out(0))
                           .input(None, None, Some(0)));
    });
}

#[test]
fn check_matrix_connect_even_top() {
    simple_sine_output_test!(matrix, {
        matrix.place(0, 0, Cell::empty(NodeId::Sin(0))
                           .out(None, None, Some(0)));
        matrix.place(0, 1, Cell::empty(NodeId::Out(0))
                           .input(Some(0), None, None));
    });
}

#[test]
fn check_matrix_connect_odd_top_left() {
    simple_sine_output_test!(matrix, {
        matrix.place(0, 0, Cell::empty(NodeId::Sin(0))
                           .out(None, Some(0), None));
        matrix.place(1, 0, Cell::empty(NodeId::Out(0))
                           .input(None, Some(0), None));
    });
}

#[test]
fn check_matrix_connect_odd_bottom_left() {
    simple_sine_output_test!(matrix, {
        matrix.place(0, 1, Cell::empty(NodeId::Sin(0))
                           .out(Some(0), None, None));
        matrix.place(1, 0, Cell::empty(NodeId::Out(0))
                           .input(None, None, Some(0)));
    });
}

#[test]
fn check_matrix_connect_odd_top() {
    simple_sine_output_test!(matrix, {
        matrix.place(1, 0, Cell::empty(NodeId::Sin(0))
                           .out(None, None, Some(0)));
        matrix.place(1, 1, Cell::empty(NodeId::Out(0))
                           .input(Some(0), None, None));
    });
}


#[test]
fn check_matrix_adj_odd() {
    let (mut node_conf, mut node_exec) = new_node_engine();
    let mut matrix = Matrix::new(node_conf, 7, 7);

    /*
            _____
        I2 / I1  \ O1
          /       \
          \       /
        I3 \_____/  O2
             O3

          0     1    2      3
         ___         ___
       0/   \  ___ 0/   \  ___
        \___/0/S2 \ \___/0/   \
         ___  \___/       \___/
       1/S1 \        ___
        \___/  ___ 1/S3 \  ___
         ___ 1/S0 \ \___/1/   \
       2/S6 \ \___/       \___/
        \___/        ___
               ___ 2/S4 \  ___
             2/S5 \ \___/2/   \
              \___/       \___/
    */

    matrix.place(1, 1, Cell::empty(NodeId::Sin(0))
                       .out(Some(0), Some(0), Some(0))
                       .input(Some(0), Some(0), Some(0)));

    matrix.place(0, 1, Cell::empty(NodeId::Sin(1))
                       .out(None, Some(0), None));
    matrix.place(1, 0, Cell::empty(NodeId::Sin(2))
                       .out(None, None, Some(0)));
    matrix.place(2, 1, Cell::empty(NodeId::Sin(3))
                       .input(None, None, Some(0)));
    matrix.place(2, 2, Cell::empty(NodeId::Sin(4))
                       .input(None, Some(0), None));
    matrix.place(1, 2, Cell::empty(NodeId::Sin(5))
                       .input(Some(0), None, None));
    matrix.place(0, 2, Cell::empty(NodeId::Sin(6))
                       .out(Some(0), None, None));
    matrix.sync();

    assert_eq!(
        matrix.get_adjacent(1, 1, CellDir::B).unwrap().node_id(),
        NodeId::Sin(5));
    assert_eq!(
        matrix.get_adjacent(1, 1, CellDir::BR).unwrap().node_id(),
        NodeId::Sin(4));
    assert_eq!(
        matrix.get_adjacent(1, 1, CellDir::TR).unwrap().node_id(),
        NodeId::Sin(3));

    assert_eq!(
        matrix.get_adjacent(1, 1, CellDir::T).unwrap().node_id(),
        NodeId::Sin(2));
    assert_eq!(
        matrix.get_adjacent(1, 1, CellDir::TL).unwrap().node_id(),
        NodeId::Sin(1));
    assert_eq!(
        matrix.get_adjacent(1, 1, CellDir::BL).unwrap().node_id(),
        NodeId::Sin(6));
}


#[test]
fn check_matrix_adj_even() {
    let (mut node_conf, mut node_exec) = new_node_engine();
    let mut matrix = Matrix::new(node_conf, 7, 7);

    /*
            _____
        I2 / I1  \ O1
          /       \
          \       /
        I3 \_____/  O2
             O3

          0     1    2      3
         ___         ___
       0/   \  ___ 0/S2 \  ___
        \___/0/S1 \ \___/0/S3 \
         ___  \___/       \___/
       1/   \        ___
        \___/  ___ 1/S0 \  ___
         ___ 1/S6 \ \___/1/S4 \
       2/   \ \___/       \___/
        \___/        ___
               ___ 2/S5 \  ___
             2/   \ \___/2/   \
              \___/       \___/
    */

    matrix.place(2, 1, Cell::empty(NodeId::Sin(0))
                       .out(Some(0), Some(0), Some(0))
                       .input(Some(0), Some(0), Some(0)));

    matrix.place(1, 0, Cell::empty(NodeId::Sin(1))
                       .out(None, Some(0), None));
    matrix.place(2, 0, Cell::empty(NodeId::Sin(2))
                       .out(None, None, Some(0)));
    matrix.place(3, 0, Cell::empty(NodeId::Sin(3))
                       .input(None, None, Some(0)));
    matrix.place(3, 1, Cell::empty(NodeId::Sin(4))
                       .input(None, Some(0), None));
    matrix.place(2, 2, Cell::empty(NodeId::Sin(5))
                       .input(Some(0), None, None));
    matrix.place(1, 1, Cell::empty(NodeId::Sin(6))
                       .out(Some(0), None, None));
    matrix.sync();

    assert_eq!(
        matrix.get_adjacent(2, 1, CellDir::B).unwrap().node_id(),
        NodeId::Sin(5));
    assert_eq!(
        matrix.get_adjacent(2, 1, CellDir::BR).unwrap().node_id(),
        NodeId::Sin(4));
    assert_eq!(
        matrix.get_adjacent(2, 1, CellDir::TR).unwrap().node_id(),
        NodeId::Sin(3));

    assert_eq!(
        matrix.get_adjacent(2, 1, CellDir::T).unwrap().node_id(),
        NodeId::Sin(2));
    assert_eq!(
        matrix.get_adjacent(2, 1, CellDir::TL).unwrap().node_id(),
        NodeId::Sin(1));
    assert_eq!(
        matrix.get_adjacent(2, 1, CellDir::BL).unwrap().node_id(),
        NodeId::Sin(6));
}
