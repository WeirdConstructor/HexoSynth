// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use super::PatternColType;
use super::MAX_PATTERN_LEN;

pub struct PatternSequencer {
    rows:       usize,
    col_types:  [PatternColType; 6],
    data:       Vec<Vec<f32>>,
}

impl PatternSequencer {
    pub fn new(rows: usize) -> Self {
        Self {
            rows,
            col_types: [PatternColType::Value; 6],
            data:      vec![vec![0.0; MAX_PATTERN_LEN]; 6],
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
        &self, col: usize, phase: &[f32], out: &mut [f32])
    {
        let col = &self.data[col][..];

        let last_row_idx : f32 = (self.rows as f32) - 0.000001;

        for (phase, out) in phase.iter().zip(out.iter_mut()) {
            let row_phase  = phase * last_row_idx;
            let line       = row_phase.floor() as usize;
            let phase_frac = row_phase.fract();

            let gate : u32 = col[line].to_bits();

            // pulse_width:
            //      0xF  - Gate is on for full row
            //      0x0  - Gate is on for a very short burst
            let pulse_width : u8 =  (gate & 0x00F) as u8;
            // row_div:
            //      0xF  - Row has 1  Gate
            //      0x0  - Row is divided up into 16 Gates
            let row_div     : u8 = ((gate & 0x0F0) >> 4) as u8;
            // probability:
            //      0xF  - Gate is always triggered
            //      0x7  - Gate fires only in 50% of the cases
            //      0x0  - Gate fires only in 1% of the cases
            let probability : u8 = ((gate & 0xF00) >> 8) as u8;

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

            *out = 0.0;
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

        let mut out = [0.0; 3];
        ps.col_interpolate_at_phase(0, &[0.1, 0.5, 0.9], &mut out[..]);
        assert_float_eq!(out[0], 0.9);
        assert_float_eq!(out[1], 0.5);
        assert_float_eq!(out[2], 0.1);
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
        let mut ps = PatternSequencer::new(2);
        ps.set_col(0, &[0.0, 0.3, 1.0]);

        let mut out = [0.0; 6];
        ps.col_get_at_phase(0, &[0.1, 0.5, 0.51, 0.9, 0.99], &mut out[..]);
        assert_float_eq!(out[0], 0.0);
        assert_float_eq!(out[1], 0.0);
        assert_float_eq!(out[2], 0.3);
        assert_float_eq!(out[3], 1.0);
    }
}
