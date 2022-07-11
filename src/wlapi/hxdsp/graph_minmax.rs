// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

//use crate::arg_chk;
use wlambda::*;
use hexodsp::{Matrix, CellDir};
use hexotk::GraphMinMaxModel;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

struct MonitorMinMaxData {
    matrix:     Arc<Mutex<Matrix>>,
    index:      usize,
    min:        f32,
    max:        f32,
    avg:        f32,
}

fn sigidx2celldir(idx: usize) -> CellDir {
    match idx {
        0 => CellDir::T,
        1 => CellDir::TL,
        2 => CellDir::BL,
        3 => CellDir::TR,
        4 => CellDir::BR,
        5 => CellDir::B,
        _ => CellDir::C,
    }
}

impl GraphMinMaxModel for MonitorMinMaxData {
    fn get_generation(&self) -> u64 {
        0
    }

    fn read(&mut self, buf: &mut [(f32, f32)]) {
        let (mut min, mut max, mut avg) =
            (1000.0_f32, -1000.0_f32, 0.0_f32);

        if let Ok(mut m) = self.matrix.lock() {
            let cell = m.monitored_cell();
            if !cell.has_dir_set(sigidx2celldir(self.index)) {
                buf.fill((0.0, 0.0));
                return;
            }

            let mimbuf = m.get_minmax_monitor_samples(self.index);

            for (i, b) in buf.iter_mut().enumerate() {
                let mm = mimbuf.at(i);

                min = min.min(mm.0);
                max = max.max(mm.1);
                avg += mm.1 * 0.5 + mm.0 * 0.5;

                *b = (mm.0 as f32, mm.1 as f32);
            }

            avg /= buf.len() as f32;

            if min > 999.0  { min = 0.0; }
            if max < -999.0 { max = 0.0; }
        }

        self.avg = avg;
        self.min = min;
        self.max = max;
    }

    fn fmt_val(&mut self, buf: &mut [u8]) -> usize {
        use std::io::Write;
        let max_len = buf.len();
        let mut bw = std::io::BufWriter::new(buf);
        match write!(bw, "{:6.3} | {:6.3} | {:6.3}",
                     self.min, self.max, self.avg)
        {
            Ok(_)  => {
                if bw.buffer().len() > max_len { max_len }
                else { bw.buffer().len() }
            },
            Err(_) => 0,
        }
    }
}

#[derive(Clone)]
pub struct VGraphMinMaxModel(Rc<RefCell<dyn GraphMinMaxModel>>);

impl VGraphMinMaxModel {
    pub fn new_monitor_model(matrix: Arc<Mutex<Matrix>>, index: usize) -> Self {
        Self(Rc::new(RefCell::new(MonitorMinMaxData {
            matrix,
            index,
            min: 0.0,
            max: 0.0,
            avg: 0.0,
        })))
    }
}

impl VValUserData for VGraphMinMaxModel {
    fn s(&self) -> String { format!("$<UI::GraphMinMaxModel>") }
    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }

    fn call_method(&self, _key: &str, _env: &mut Env)
        -> Result<VVal, StackAction>
    {
        Ok(VVal::None)
    }
}

pub fn vv2graph_minmax_model(mut v: VVal) -> Option<Rc<RefCell<dyn GraphMinMaxModel>>> {
    v.with_usr_ref(|model: &mut VGraphMinMaxModel| model.0.clone())
}
