// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use hexotk::widgets::UIPatternModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum PatternColType {
    Note,
    Step,
    Value,
}

#[derive(Debug)]
pub struct PatternData {
    col_types:  [PatternColType; 6],
    data:       Vec<Vec<Option<u16>>>,
    out_data:   Vec<f32>;
    strings:    Vec<Vec<Option<String>>>,
    cursor:     (usize, usize),
    edit_step:  usize,
}

impl PatternData {
    pub fn new(len: usize) -> Self {
        Self {
            col_types:  [PatternColType::Value; 6],
            data:       vec![vec![None; 6]; len],
            strings:    vec![vec![None; 6]; len],
            cursor:     (2, 2),
            edit_step:  4,
        }
    }
}

impl PatternData {
    pub fn sync_out_data(&mut self, col: usize) {
        // assume 0.0 as start value
        let mut last_value = 0.0;

        match self.col_types[col] {
            PatternColType::Note => {
                for row in 0..self.data.len() {
                    let cell = self.data[row][col];
                    // - check if cell is empty
                    // - count the number of rows until:
                    //   - we hit a value
                    //   - or end of the pattern
                    // - if the cell has a value:
                    //    - convert cell u16 to note
                    //    - write all prior empty rows with the value
                }
            },
            PatternColType::Step => {
            },
            PatternColType::Value => {
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

    fn set_cursor(&mut self, row: usize, col: usize) {
        self.cursor = (row, col);
    }
    fn get_cursor(&self) -> (usize, usize) { self.cursor }
    fn set_edit_step(&mut self, es: usize) { self.edit_step = es; }
    fn get_edit_step(&mut self) -> usize { self.edit_step }
}
