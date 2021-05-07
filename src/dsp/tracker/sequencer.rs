// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use super::PatternColType;
use super::MAX_PATTERN_LEN;
use super::MAX_COLS;
use crate::dsp::helpers::SplitMix64;

pub struct PatternSequencer {
    rows:       usize,
    col_types:  [PatternColType; MAX_COLS],
    data:       Vec<Vec<f32>>,
    rng:        SplitMix64,
    rand_vals:  [(usize, f64); MAX_COLS],
}

const FRACT_16THS : [f32; 16] = [
    1.0 / 16.0,
    2.0 / 16.0,
    3.0 / 16.0,
    4.0 / 16.0,
    5.0 / 16.0,
    6.0 / 16.0,
    7.0 / 16.0,
    8.0 / 16.0,
    9.0 / 16.0,
   10.0 / 16.0,
   11.0 / 16.0,
   12.0 / 16.0,
   13.0 / 16.0,
   14.0 / 16.0,
   15.0 / 16.0,
           1.0
];

impl PatternSequencer {
    pub fn new_default_seed(rows: usize) -> Self {
        Self {
            rows,
            col_types: [PatternColType::Value; MAX_COLS],
            data:      vec![vec![0.0; MAX_PATTERN_LEN]; MAX_COLS],
            rng:       SplitMix64::new(0x91234),
            rand_vals: [(99999, 0.0); MAX_COLS],
        }
    }

    pub fn new(rows: usize) -> Self {
        use std::time::SystemTime;
        let seed =
            match SystemTime::now()
                  .duration_since(SystemTime::UNIX_EPOCH)
            {
                Ok(n)  => n.as_nanos() as i64,
                Err(_) => 1_234_567_890,
            };
        Self {
            rows,
            col_types: [PatternColType::Value; MAX_COLS],
            data:      vec![vec![0.0; MAX_PATTERN_LEN]; MAX_COLS],
            rng:       SplitMix64::new_from_i64(seed),
            rand_vals: [(99999, 0.0); MAX_COLS],
        }
    }

    pub fn set_rows(&mut self, rows: usize) {
        self.rows = rows;
    }

    pub fn rows(&self) -> usize { self.rows }

    pub fn set_col(&mut self, col: usize, col_data: &[f32])
    {
        for (out_cell, in_cell) in self.data[col].iter_mut().zip(col_data.iter()) {
            if *in_cell > 0.0 {
                println!("YOOOO UPDATE SETCOL {}", *in_cell);
            }
            *out_cell = *in_cell;
        }
    }

    pub fn col_interpolate_at_phase(
        &self, col: usize, phase: &[f32], out: &mut [f32])
    {
        let col = &self.data[col][..];

        let last_row_idx : f32 = (self.rows as f32) - 0.000001;

        for (phase, out) in phase.iter().zip(out.iter_mut()) {
            let row_phase  = phase * last_row_idx;
            let phase_frac = row_phase.fract();
            let line       = row_phase.floor() as usize;
            let prev_line  = if line == 0 { self.rows - 1 } else { line - 1 };

            let prev = col[prev_line];
            let next = col[line];

            // println!("INTERP: {}={:9.7}, {}={:9.7} | {:9.7}",
            //          prev_line, prev,
            //          line, next,
            //          phase_frac);

            *out = prev * (1.0 - phase_frac) + next * phase_frac;
        }
    }

    pub fn col_get_at_phase(
        &self, col: usize, phase: &[f32], out: &mut [f32])
    {
        let col = &self.data[col][..];

        let last_row_idx : f32 = (self.rows as f32) - 0.000001;

        for (phase, out) in phase.iter().zip(out.iter_mut()) {
            let row_phase  = phase * last_row_idx;
            let line       = row_phase.floor() as usize;

            *out = col[line];
        }
    }

    pub fn col_gate_at_phase(
        &mut self, col_idx: usize, phase: &[f32], out: &mut [f32])
    {
        let col = &self.data[col_idx][..];

        let last_row_idx : f32 = (self.rows as f32) - 0.00001;

        for (phase, out) in phase.iter().zip(out.iter_mut()) {
            let row_phase  = phase.clamp(0.0, 1.0) * last_row_idx;
            let line       = row_phase.floor() as usize;
            let phase_frac = row_phase.fract();

            let gate : u32 = col[line].to_bits();

            // pulse_width:
            //      0xF  - Gate is on for full row
            //      0x0  - Gate is on for a very short burst
            let pulse_width : f32 = FRACT_16THS[(gate & 0x00F) as usize];
            // row_div:
            //      0xF  - Row has 1  Gate
            //      0x0  - Row is divided up into 16 Gates
            let row_div     : f32 = (16 - ((gate & 0x0F0) >> 4)) as f32;
            // probability:
            //      0xF  - Row is always triggered
            //      0x7  - Row fires only in 50% of the cases
            //      0x0  - Row fires only in ~6% of the cases
            let probability : u8 = ((gate & 0xF00) >> 8) as u8;

            let sub_frac = (phase_frac * row_div).fract();
            //d// println!(
            //d//     "row_div={}, pw={}, phase={} / {}",
            //d//     row_div, pulse_width, sub_frac, phase_frac);

            if probability < 0xF {
                let rand_val =
                    if self.rand_vals[col_idx].0 != line {
                        let new_rand_val = self.rng.next_open01();
                        self.rand_vals[col_idx] = (line, new_rand_val);
                        new_rand_val
                    } else {
                        self.rand_vals[col_idx].1
                    };
                //d// println!("RANDVAL: {:?} | {:9.7}", self.rand_vals[col_idx], FRACT_16THS[probability as usize]);

                if rand_val > (FRACT_16THS[probability as usize] as f64) {
                    *out = 0.0;
                    continue;
                }
            }

            if (gate & 0xF000) > 0 {
                *out = 0.0;
            } else {
                *out = if sub_frac <= pulse_width { 1.0 } else { 0.0 };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_float_eq {
        ($a:expr, $b:expr) => {
            if ($a - $b).abs() > 0.0001 {
                panic!(r#"assertion failed: `(left == right)`
      left: `{:?}`,
     right: `{:?}`"#, $a, $b)
            }
        }
    }

    #[test]
    fn check_seq_interpolate_1() {
        let mut ps = PatternSequencer::new(2);
        ps.set_col(0, &[0.0, 1.0]);

        let mut out = [0.0; 6];
        ps.col_interpolate_at_phase(0, &[0.0, 0.1, 0.50, 0.51, 0.9, 0.99999], &mut out[..]);
        assert_float_eq!(out[0], 1.0);
        assert_float_eq!(out[1], 0.8);
        assert_float_eq!(out[2], 0.0);
        assert_float_eq!(out[3], 0.02);
        assert_float_eq!(out[4], 0.8);
        assert_float_eq!(out[5], 0.99999);
    }

    #[test]
    fn check_seq_step_1() {
        let mut ps = PatternSequencer::new(2);
        ps.set_col(0, &[0.0, 1.0]);

        let mut out = [0.0; 3];
        ps.col_get_at_phase(0, &[0.1, 0.51, 0.9], &mut out[..]);
        assert_float_eq!(out[0], 0.0);
        assert_float_eq!(out[1], 1.0);
        assert_float_eq!(out[2], 1.0);
    }

    #[test]
    fn check_seq_step_2() {
        let mut ps = PatternSequencer::new(3);
        ps.set_col(0, &[0.0, 0.3, 1.0]);

        let mut out = [0.0; 6];
        ps.col_get_at_phase(0, &[0.1, 0.5, 0.51, 0.6, 0.9, 0.99], &mut out[..]);
        assert_float_eq!(out[0], 0.0);
        assert_float_eq!(out[1], 0.3);
        assert_float_eq!(out[2], 0.3);
        assert_float_eq!(out[3], 0.3);
        assert_float_eq!(out[4], 1.0);
        assert_float_eq!(out[5], 1.0);
    }

    #[test]
    fn check_seq_gate_1() {
        let mut ps = PatternSequencer::new(2);
        ps.set_col(0, &[
            f32::from_bits(0x0FFF),
            f32::from_bits(0xF000),
        ]);

        let mut out = [0.0; 6];
        ps.col_gate_at_phase(0, &[0.1, 0.5, 0.5001, 0.6, 0.9, 0.99], &mut out[..]);
        //d// println!("out: {:?}", out);

        assert_float_eq!(out[0], 1.0);
        assert_float_eq!(out[1], 1.0);
        assert_float_eq!(out[2], 0.0);
        assert_float_eq!(out[3], 0.0);
        assert_float_eq!(out[4], 0.0);
        assert_float_eq!(out[5], 0.0);
    }

    fn count_high(slice: &[f32]) -> usize {
        let mut sum = 0;
        for p in slice.iter() {
            if *p > 0.5 { sum += 1; }
        }
        sum
    }

    fn count_up(slice: &[f32]) -> usize {
        let mut sum = 0;
        let mut cur = 0.0;
        for p in slice.iter() {
            if cur < 0.1 && *p > 0.5 {
                sum += 1;
            }
            cur = *p;
        }
        sum
    }

    #[test]
    fn check_seq_gate_2() {
        let mut ps = PatternSequencer::new(3);
        ps.set_col(0, &[
            f32::from_bits(0x0FF0),
            f32::from_bits(0x0FF7),
            f32::from_bits(0x0FFF),
        ]);

        let mut phase = vec![0.0; 96];
        let inc = 1.0 / (96.0 - 1.0);
        let mut phase_run = 0.0;
        for p in phase.iter_mut() {
            *p = phase_run;
            phase_run += inc;
        }

        //d// println!("PHASE: {:?}", phase);

        let mut out = [0.0; 96];
        ps.col_gate_at_phase(0, &phase[..], &mut out[..]);
        //d// println!("out: {:?}", &out[0..32]);

        assert_eq!(count_high(&out[0..32]),   2);
        assert_eq!(count_high(&out[32..64]), 16);
        assert_eq!(count_high(&out[64..96]), 32);

        assert_eq!(count_up(&out[0..32]),   1);
        assert_eq!(count_up(&out[32..64]),  1);
        assert_eq!(count_up(&out[64..96]),  1);
    }

    #[test]
    fn check_seq_gate_div_1() {
        let mut ps = PatternSequencer::new(3);
        ps.set_col(0, &[
            f32::from_bits(0x0F80),
            f32::from_bits(0x0F87),
            f32::from_bits(0x0F8F),
        ]);

        let mut phase = vec![0.0; 3 * 64];
        let inc = 1.0 / ((3.0 * 64.0) - 1.0);
        let mut phase_run = 0.0;
        for p in phase.iter_mut() {
            *p = phase_run;
            phase_run += inc;
        }

        //d// println!("PHASE: {:?}", phase);

        let mut out = [0.0; 3 * 64];
        ps.col_gate_at_phase(0, &phase[..], &mut out[..]);

        assert_eq!(count_high(&out[0..64]),  8);
        assert_eq!(count_up(  &out[0..64]),  8);

        assert_eq!(count_high(&out[64..128]), 32);
        assert_eq!(count_up(  &out[64..128]),  8);

        assert_eq!(count_high(&out[128..192]), 64);
        assert_eq!(count_up(  &out[128..192]),  1);
    }

    #[test]
    fn check_seq_gate_div_2() {
        let mut ps = PatternSequencer::new(3);
        ps.set_col(0, &[
            f32::from_bits(0x0F00),
            f32::from_bits(0x0F07),
            f32::from_bits(0x0F0F),
        ]);

        let mut phase = vec![0.0; 6 * 64];
        let inc = 1.0 / ((6.0 * 64.0) - 1.0);
        let mut phase_run = 0.0;
        for p in phase.iter_mut() {
            *p = phase_run;
            phase_run += inc;
        }

        //d// println!("PHASE: {:?}", phase);

        let mut out = [0.0; 6 * 64];
        ps.col_gate_at_phase(0, &phase[..], &mut out[..]);

        assert_eq!(count_high(&out[0..128]), 16);
        assert_eq!(count_up(  &out[0..128]), 16);

        assert_eq!(count_high(&out[128..256]), 64);
        assert_eq!(count_up(  &out[128..256]), 16);

        assert_eq!(count_high(&out[256..384]), 128);
        assert_eq!(count_up(  &out[256..384]),   1);
    }

    #[test]
    fn check_seq_gate_div_3() {
        let mut ps = PatternSequencer::new(3);
        ps.set_col(0, &[
            f32::from_bits(0x0FE0),
            f32::from_bits(0x0FE7),
            f32::from_bits(0x0FEF),
        ]);

        let mut phase = vec![0.0; 6 * 64];
        let inc = 1.0 / ((6.0 * 64.0) - 1.0);
        let mut phase_run = 0.0;
        for p in phase.iter_mut() {
            *p = phase_run;
            phase_run += inc;
        }

        //d// println!("PHASE: {:?}", phase);

        let mut out = [0.0; 6 * 64];
        ps.col_gate_at_phase(0, &phase[..], &mut out[..]);

        assert_eq!(count_high(&out[0..128]),  8);
        assert_eq!(count_up(  &out[0..128]),  2);

        assert_eq!(count_high(&out[128..256]), 64);
        assert_eq!(count_up(  &out[128..256]),  2);

        assert_eq!(count_high(&out[256..384]), 128);
        assert_eq!(count_up(  &out[256..384]),   1);
    }

    fn run_probability_test_for(prob: u32) -> (usize, usize) {
        let rows = 100;

        let mut ps = PatternSequencer::new_default_seed(rows);
        let mut coldata = vec![0.0; rows];
        for i in 0..coldata.len() {
            coldata[i] = f32::from_bits(0x00FF | prob);
        }
        ps.set_col(0, &coldata[..]);

        let mut samples = rows;
        let mut phase = vec![0.0; samples];
        let inc = 1.0 / ((samples as f32) - 1.0);
        let mut phase_run = 0.0;
        for p in phase.iter_mut() {
            *p = phase_run;
            phase_run += inc;
        }

        let mut out = vec![0.0; samples];
        ps.col_gate_at_phase(0, &phase[..], &mut out[..]);

        (count_high(&out[..]), count_up(&out[..]))
    }

    #[test]
    fn check_seq_gate_div_rng() {
        // XXX: The result numbers are highly dependent on the
        //      sampling rate inside run_probability_test_for().
        assert_eq!(run_probability_test_for(0x000), (5,   5));
        assert_eq!(run_probability_test_for(0x100), (12, 11));
        assert_eq!(run_probability_test_for(0x200), (20, 18));
        assert_eq!(run_probability_test_for(0x300), (26, 23));
        assert_eq!(run_probability_test_for(0x400), (32, 26));
        assert_eq!(run_probability_test_for(0x500), (38, 29));
        assert_eq!(run_probability_test_for(0x600), (47, 29));
        assert_eq!(run_probability_test_for(0x700), (56, 26));
        assert_eq!(run_probability_test_for(0x800), (60, 25));
        assert_eq!(run_probability_test_for(0x900), (66, 24));
        assert_eq!(run_probability_test_for(0xA00), (70, 22));
        assert_eq!(run_probability_test_for(0xB00), (79, 18));
        assert_eq!(run_probability_test_for(0xC00), (84, 13));
        assert_eq!(run_probability_test_for(0xD00), (93,  7));
        assert_eq!(run_probability_test_for(0xE00), (96,  5));
        assert_eq!(run_probability_test_for(0xF00), (100, 1));
    }
}
