// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

pub struct PatternSequencer {
    cols: usize,
    rows: usize,
    data: Vec<f32>,
}

impl PatternSequencer {
    pub fn get_at_phase(&self, phase: f64, out: &mut [f32]) {
        let row_phase  = phase * (self.rows as f64);
        let phase_frac = row_phase.fract();
        let line       = row_phase.floor() as usize;
        let prev_line  = if line == 0 { self.rows - 1 } else { line - 1 };

        for col_idx in 0..self.cols {
            let prev = self.data[prev_line * cols + col_idx];
            let next = self.data[line      * cols + col_idx];

            out[col_idx] = prev * (1.0 - phase_frac) + next * phase_frac;
        }
    }
}
