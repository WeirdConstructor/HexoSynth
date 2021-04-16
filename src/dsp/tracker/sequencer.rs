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
    pub fn new() -> Self {
        Self {
            rows:      16,
            col_types: [PatternColType::Value; 6],
            data:      vec![vec![0.0; 16]; 6],
        }
    }

    pub fn col_interpolate_at_phase(
        &self, col: usize, phase: &[f32], out: &mut [f32])
    {
        let col = &self.data[col][..];

        for (phase, out) in phase.iter().zip(out.iter_mut()) {
            let row_phase  = phase * (self.rows as f32);
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

        for (phase, out) in phase.iter().zip(out.iter_mut()) {
            let row_phase  = phase * (self.rows as f32);
            let line       = row_phase.floor() as usize;

            *out = col[line];
        }
    }

    pub fn col_interpolate_at_phase(
        &self, col: usize, phase: &[f32], out: &mut [f32])
    {
        let col = &self.data[col][..];

        for (phase, out) in phase.iter().zip(out.iter_mut()) {
            let row_phase  = phase * (self.rows as f32);
            let line       = row_phase.floor() as usize;
            let phase_frac = row_phase.fract();

            let gate : u32 = col[line].to_bits();

            // pulse_width:
            //      0xF  - Gate is on for full row
            //      0x0  - Gate is on for a very short burst
            let pulse_width : u8 =  gate & 0x00F;
            // row_div:
            //      0xF  - Row has 1  Gate
            //      0x0  - Row is divided up into 16 Gates
            let row_div     : u8 = (gate & 0x0F0) >> 4;
            // probability:
            //      0xF  - Gate is always triggered
            //      0x7  - Gate fires only in 50% of the cases
            //      0x0  - Gate fires only in 1% of the cases
            let probability : u8 = (gate & 0xF00) >> 8;

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





