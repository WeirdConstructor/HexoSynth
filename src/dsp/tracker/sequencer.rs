// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use super::PatternColType;
use super::MAX_PATTERN_LEN;
use crate::dsp::helpers::SplitMix64;

pub struct PatternSequencer {
    rows:       usize,
    col_types:  [PatternColType; 6],
    data:       Vec<Vec<f32>>,
    rng:        SplitMix64,
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
            col_types: [PatternColType::Value; 6],
            data:      vec![vec![0.0; MAX_PATTERN_LEN]; 6],
            rng:       SplitMix64::new(0x123456789),
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
            col_types: [PatternColType::Value; 6],
            data:      vec![vec![0.0; MAX_PATTERN_LEN]; 6],
            rng:       SplitMix64::new_from_i64(seed),
        }
    }

    pub fn set_col(&mut self, col: usize, col_data: &[f32])
    {
        for (out_cell, in_cell) in self.data[col].iter_mut().zip(col_data.iter()) {
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
        &mut self, col: usize, phase: &[f32], out: &mut [f32])
    {
        let col = &self.data[col][..];

        let last_row_idx : f32 = (self.rows as f32) - 0.000001;

        for (phase, out) in phase.iter().zip(out.iter_mut()) {
            let row_phase  = phase.clamp(0.0, 1.0) * last_row_idx;
            let line       = row_phase.floor() as usize;
            let phase_frac = row_phase.fract();

            let gate : u32 = col[line].to_bits();

            if (gate & 0xF000) > 0 {
                *out = 0.0;
                continue;
            }

            // pulse_width:
            //      0xF  - Gate is on for full row
            //      0x0  - Gate is on for a very short burst
            let pulse_width : f32 = FRACT_16THS[(gate & 0x00F) as usize];
            // row_div:
            //      0xF  - Row has 1  Gate
            //      0x0  - Row is divided up into 16 Gates
            let row_div     : f32 = (((gate & 0x0F0) >> 4) + 1) as f32;
            // probability:
            //      0xF  - Gate is always triggered
            //      0x7  - Gate fires only in 50% of the cases
            //      0x0  - Gate fires only in ~6% of the cases
            let probability : u8 = ((gate & 0xF00) >> 8) as u8;

            let sub_frac = (phase_frac * row_div).fract();

            // println!(
            //     "phase_frac={:9.7}, sub_frac={:9.7}, pw={:9.7}",
            //     phase_frac, sub_frac, pulse_width);

            // TODO: FIXME: We need to calculate the random value
            //              once per row!
            if probability < 0xF {
                if self.rng.next_open01()
                   > (FRACT_16THS[probability as usize] as f64)
                {
                    continue;
                }
            }

            *out = if sub_frac <= pulse_width { 1.0 } else { 0.0 };

            // Ideas:
            // compute probability:
            //    if self.cur_row_not_played or a new row
            //       with a gate is encountered, draw one
            //       random number.
            //       set self.cur_row_not_played = is_played;
            //       if not is_played { skip ...}
            //       else { play .. }
            // pulse_width:
            //    - get the length of the row (or it's divided form)
            //    - calculate the length of the pulse width (0xF is
            //      the complete length, 0x0 is 1/16th?)
            //    - sub-div = 1 / 4 (0xB)
            //    - sub-phase =
            //          (fract / sub-div) - (fract / sub-div).floor()
            //    - check if sub-phase is inside the % of the pulsewidth
        }
    }
}

// TODO: If PatternColType::None, we don't have to play!


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
        ps.col_gate_at_phase(0, &[0.1, 0.5, 0.500001, 0.6, 0.9, 0.99], &mut out[..]);
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
            f32::from_bits(0x0F00),
            f32::from_bits(0x0F07),
            f32::from_bits(0x0F0F),
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
            f32::from_bits(0x0F70),
            f32::from_bits(0x0F77),
            f32::from_bits(0x0F7F),
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
            f32::from_bits(0x0FF0),
            f32::from_bits(0x0FF7),
            f32::from_bits(0x0FFF),
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
            f32::from_bits(0x0F10),
            f32::from_bits(0x0F17),
            f32::from_bits(0x0F1F),
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

    #[test]
    fn check_seq_gate_div_rng_1() {
        let mut ps = PatternSequencer::new_default_seed(3);
        ps.set_col(0, &[
            f32::from_bits(0x00F0),
            f32::from_bits(0x00F7),
            f32::from_bits(0x00FF),
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
}
