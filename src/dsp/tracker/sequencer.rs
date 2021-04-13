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
}
