// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use hexotk::widgets::UIPatternModel;
use super::PatternColType;
use super::MAX_PATTERN_LEN;

pub enum PatternUpdateMsg {
    UpdateColumn {
        col_type:    PatternColType,
        pattern_len: usize,
        data:        [f32; MAX_PATTERN_LEN]
    },
}

#[derive(Debug)]
pub struct PatternData {
    col_types:  [PatternColType; 6],
    data:       Vec<Vec<Option<u16>>>,
    out_data:   Vec<[f32; MAX_PATTERN_LEN]>,
    strings:    Vec<Vec<Option<String>>>,
    cursor:     (usize, usize),
    rows:       usize,
    edit_step:  usize,
}

impl PatternData {
    pub fn new(rows: usize) -> Self {
        Self {
            col_types:  [PatternColType::Value; 6],
            data:       vec![vec![None; 6]; MAX_PATTERN_LEN],
            out_data:   vec![[0.0; MAX_PATTERN_LEN]; 6],
            strings:    vec![vec![None; 6]; MAX_PATTERN_LEN],
            cursor:     (2, 2),
            edit_step:  4,
            rows,
        }
    }
}

impl PatternData {
    pub fn get_out_data(&self) -> &[[f32; MAX_PATTERN_LEN]] {
        &self.out_data
    }

    pub fn sync_out_data(&mut self, col: usize) {
        let mut out_col = &mut self.out_data[col];

        match self.col_types[col] {
            PatternColType::Value => {
                let mut start_value = 0.0;
                let mut start_idx   = 0;
                let mut end_idx     = 0;

                while end_idx <= self.rows {
                    let mut break_after_write = false;
                    let cur_value =
                        if end_idx == self.rows {
                            end_idx -= 1;
                            break_after_write = true;
                            Some(self.data[end_idx][col]
                                .map(|v| (v as f32) / (0xFFF as f32))
                                .unwrap_or(0.0))
                        } else {
                            self.data[end_idx][col].map(|v|
                                (v as f32) / (0xFFF as f32))
                        };

                    if let Some(end_value) = cur_value {
                        out_col[start_idx] = start_value;
                        out_col[end_idx]   = end_value;

                        let delta_rows = end_idx - start_idx;

                        if delta_rows > 1 {
                            for idx in (start_idx + 1)..end_idx {
                                let x =
                                      (idx - start_idx) as f32
                                    / (delta_rows as f32);
                                out_col[idx] =
                                    start_value * (1.0 - x) + end_value * x;
                            }
                        }

                        start_value = end_value;
                        start_idx   = end_idx;
                        end_idx     = end_idx + 1;

                        if break_after_write {
                            break;
                        }

                    } else {
                        end_idx += 1;
                    }
                }
            },
            PatternColType::Note => {
                let mut cur_value = 0.0;

                for row in 0..self.rows {
                    if let Some(new_value) = self.data[row][col] {
                        cur_value =
                            ((new_value as i32 - 69) as f32 * 0.1) / 12.0;
                    }

                    out_col[row] = cur_value;
                }
            },
            PatternColType::Step => {
                let mut cur_value = 0.0;

                for row in 0..self.rows {
                    if let Some(new_value) = self.data[row][col] {
                        cur_value = (new_value as f32) / (0xFFF as f32);
                    }

                    out_col[row] = cur_value;
                }
            },
            PatternColType::Gate => {
                for row in 0..self.rows {
                    out_col[row] =
                        if let Some(new_value) = self.data[row][col] {
                            f32::from_bits(new_value as u32)
                        } else {
                            f32::from_bits(0xF000 as u32)
                        };
                }
            },
        }
    }
}

impl UIPatternModel for PatternData {
    fn get_cell(&mut self, row: usize, col: usize) -> Option<&str> {
        if row >= self.data.len()    { return None; }
        if col >= self.data[0].len() { return None; }

        if self.strings[row][col].is_none() {
            if let Some(v) = self.data[row][col] {
                self.strings[row][col] = Some(format!("{:03x}", v));
            } else {
                return None;
            }
        }

        Some(self.strings[row][col].as_ref().unwrap())
    }

    fn clear_cell(&mut self, row: usize, col: usize) {
        if row >= self.data.len()    { return; }
        if col >= self.data[0].len() { return; }

        self.data[row][col]    = None;
        self.strings[row][col] = None;
    }

    fn set_cell_note(&mut self, row: usize, col: usize, _note: &str) {
        if row >= self.data.len()    { return; }
        if col >= self.data[0].len() { return; }

        self.data[row][col]    = Some(0x0);
        self.strings[row][col] = None;
    }

    fn get_cell_value(&mut self, row: usize, col: usize) -> u16 {
        if row >= self.data.len()    { return 0; }
        if col >= self.data[0].len() { return 0; }

        self.data[row][col].unwrap_or(0)
    }

    fn set_cell_value(&mut self, row: usize, col: usize, val: u16) {
        if row >= self.data.len()    { return; }
        if col >= self.data[0].len() { return; }

        self.data[row][col]    = Some(val);
        self.strings[row][col] = None;
    }

    fn is_col_note(&self, col: usize) -> bool {
        if let Some(ct) = self.col_types.get(col) {
            *ct == PatternColType::Note
        } else {
            false
        }
    }

    fn is_col_step(&self, col: usize) -> bool {
        if let Some(ct) = self.col_types.get(col) {
            *ct == PatternColType::Step
        } else {
            false
        }
    }

    fn is_col_gate(&self, col: usize) -> bool {
        if let Some(ct) = self.col_types.get(col) {
            *ct == PatternColType::Gate
        } else {
            false
        }
    }

    fn cols(&self) -> usize { self.data[0].len() }

    fn rows(&self) -> usize { self.data.len() }

    fn set_col_note_type(&mut self, col: usize) {
        if col >= self.col_types.len() { return; }
        self.col_types[col] = PatternColType::Note;
    }

    fn set_col_step_type(&mut self, col: usize) {
        if col >= self.col_types.len() { return; }
        self.col_types[col] = PatternColType::Step;
    }

    fn set_col_value_type(&mut self, col: usize) {
        if col >= self.col_types.len() { return; }
        self.col_types[col] = PatternColType::Value;
    }

    fn set_col_gate_type(&mut self, col: usize) {
        if col >= self.col_types.len() { return; }
        self.col_types[col] = PatternColType::Gate;
    }

    fn set_cursor(&mut self, row: usize, col: usize) {
        self.cursor = (row, col);
    }
    fn get_cursor(&self) -> (usize, usize) { self.cursor }
    fn set_edit_step(&mut self, es: usize) { self.edit_step = es; }
    fn get_edit_step(&mut self) -> usize { self.edit_step }
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
    fn check_linear_value_corner_case1_0_to_1() {
        let mut pats = PatternData::new(3);

        for col in 0..6 {
            pats.set_col_value_type(col);
            pats.set_cell_value(0, col, 0);
            pats.set_cell_value(2, col, 0xFFF);
            pats.sync_out_data(col);

            let out_data = pats.get_out_data();

            let inc = 1.0 / 2.0;
            for i in 1..2 {
                let delta =
                    out_data[col][i]
                    - out_data[col][i - 1];
                assert_float_eq!(delta, inc);
            }
        }
    }

    #[test]
    fn check_linear_value_corner_case2_0_to_1() {
        let mut pats = PatternData::new(4);

        for col in 0..6 {
            pats.set_col_value_type(col);
            pats.set_cell_value(0, col, 0);
            pats.set_cell_value(3, col, 0xFFF);
            pats.sync_out_data(col);

            let out_data = pats.get_out_data();

            let inc = 1.0 / 3.0;
            for i in 1..3 {
                let delta =
                    out_data[col][i]
                    - out_data[col][i - 1];
                assert_float_eq!(delta, inc);
            }
        }
    }

    #[test]
    fn check_linear_value_out_0_to_1() {
        let mut pats = PatternData::new(16);

        for col in 0..6 {
            pats.set_col_value_type(col);
            pats.set_cell_value(0,  col, 0);
            pats.set_cell_value(15, col, 0xFFF);
            pats.sync_out_data(col);

            let out_data = pats.get_out_data();

            let inc = 1.0 / 15.0;

            //d// println!("out: {:?}", &out_data[col][0..16]);
            for i in 1..16 {
                let delta =
                    out_data[col][i]
                    - out_data[col][i - 1];
                assert_float_eq!(delta, inc);
            }
        }
    }

    #[test]
    fn check_linear_value_out_1_to_0() {
        let mut pats = PatternData::new(16);

        for col in 0..6 {
            pats.set_col_value_type(col);
            pats.set_cell_value(0, col, 0xFFF);
            pats.sync_out_data(col);

            let out_data = pats.get_out_data();

            let inc = 1.0 / 15.0;

            for i in 1..16 {
                let delta =
                    out_data[col][i]
                    - out_data[col][i - 1];
                assert_float_eq!(delta.abs(), inc);
            }
        }
    }

    #[test]
    fn check_linear_value_out_cast1_1_to_1() {
        let mut pats = PatternData::new(16);

        for col in 0..6 {
            pats.set_col_value_type(col);
            pats.set_cell_value(7, col, 0xFFF);
            pats.set_cell_value(8, col, 0xFFF);
            pats.sync_out_data(col);

            let out_data = pats.get_out_data();

            let inc = 1.0 / 15.0;

            //d// println!("out: {:?}", &out_data[col][0..16]);
            for i in 0..8 {
                assert_float_eq!(
                    out_data[col][i],
                    out_data[col][15 - i]);
            }
        }
    }

    #[test]
    fn check_linear_value_out_case2_1_to_1() {
        let mut pats = PatternData::new(16);

        for col in 0..6 {
            pats.set_col_value_type(col);
            pats.set_cell_value(6, col, 0xFFF);
            pats.set_cell_value(9, col, 0xFFF);
            pats.sync_out_data(col);

            let out_data = pats.get_out_data();

            let inc = 1.0 / 15.0;

            //d// println!("out: {:?}", &out_data[col][0..16]);
            for i in 0..8 {
                assert_float_eq!(
                    out_data[col][i],
                    out_data[col][15 - i]);
            }
        }
    }

    #[test]
    fn check_linear_value_out_case3_1_to_1() {
        let mut pats = PatternData::new(16);

        for col in 0..6 {
            pats.set_col_value_type(col);
            pats.set_cell_value(6, col, 0xFFF);
            pats.set_cell_value(7, col, 0x0);
            pats.set_cell_value(8, col, 0x0);
            pats.set_cell_value(9, col, 0xFFF);
            pats.sync_out_data(col);

            let out_data = pats.get_out_data();

            let inc = 1.0 / 15.0;

            //d// println!("out: {:?}", &out_data[col][0..16]);
            for i in 0..8 {
                assert_float_eq!(
                    out_data[col][i],
                    out_data[col][15 - i]);
            }
        }
    }

    #[test]
    fn check_linear_value_out_case4_1_to_1() {
        let mut pats = PatternData::new(16);

        for col in 0..6 {
            pats.set_col_value_type(col);
            pats.set_cell_value(5, col, 0xFFF);
            pats.set_cell_value(7, col, 0x0);
            pats.set_cell_value(8, col, 0x0);
            pats.set_cell_value(10, col, 0xFFF);
            pats.sync_out_data(col);

            let out_data = pats.get_out_data();

            let inc = 1.0 / 15.0;

            //d// println!("out: {:?}", &out_data[col][0..16]);

            assert_float_eq!(0.5, out_data[col][6]);
            assert_float_eq!(0.5, out_data[col][9]);

            for i in 0..8 {
                assert_float_eq!(
                    out_data[col][i],
                    out_data[col][15 - i]);
            }
        }
    }

    #[test]
    fn check_pattern_step_out() {
        let mut pats = PatternData::new(16);

        for col in 0..6 {
            pats.set_col_step_type(col);
            pats.set_cell_value(4,  col, 0x450);
            pats.set_cell_value(5,  col, 0x0);
            pats.set_cell_value(7,  col, 0x7ff);
            pats.set_cell_value(9,  col, 0x800);
            pats.set_cell_value(10, col, 0xfff);
            pats.sync_out_data(col);

            let out_data = pats.get_out_data();
            assert_float_eq!(out_data[col][0], 0.0);
            assert_float_eq!(out_data[col][4], 0.26959708);
            assert_float_eq!(out_data[col][5], 0.0);
            assert_float_eq!(out_data[col][7], 0.4998779);
            assert_float_eq!(out_data[col][8], 0.4998779);
            assert_float_eq!(out_data[col][9], 0.50012213);
            assert_float_eq!(out_data[col][10], 1.0);
            assert_float_eq!(out_data[col][15], 1.0);
        }
    }

    #[test]
    fn check_pattern_note_out() {
        let mut pats = PatternData::new(16);

        for col in 0..6 {
            pats.set_col_note_type(col);
            pats.set_cell_value(4, col, 0x45);
            pats.set_cell_value(5, col, 0x0);
            pats.set_cell_value(7, col, 0x45 - 12);
            pats.set_cell_value(10, col, 0x45 + 12);
            pats.sync_out_data(col);

            let out_data = pats.get_out_data();
            assert_float_eq!(out_data[col][0], 0.0);
            assert_float_eq!(out_data[col][4], 0.0);
            assert_float_eq!(out_data[col][5], -0.575);
            assert_float_eq!(out_data[col][7], -0.1);
            assert_float_eq!(out_data[col][9], -0.1);
            assert_float_eq!(out_data[col][10], 0.1);
            assert_float_eq!(out_data[col][15], 0.1);
        }
    }
}