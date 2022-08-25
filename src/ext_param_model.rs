// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use hexotk::{ChangeRes, ParamModel};

use std::sync::atomic::{AtomicU64, Ordering};

use std::io::Write;
use std::sync::{Arc, Mutex};

struct ExtCallbacks {
    on_change_start: Option<Box<dyn FnMut() + Send>>,
    on_change_set: Option<Box<dyn FnMut(f32) + Send>>,
    on_change_end: Option<Box<dyn FnMut() + Send>>,
    on_get: Option<Box<dyn Fn() -> f32 + Send>>,
}

#[derive(Clone)]
pub struct ExtParam {
    name: String,
    cbs: Arc<Mutex<ExtCallbacks>>,
    ext_counter: Option<Arc<AtomicU64>>,
}

impl ExtParam {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cbs: Arc::new(Mutex::new(ExtCallbacks {
                on_change_start: None,
                on_change_end: None,
                on_change_set: None,
                on_get: None,
            })),
            ext_counter: None,
        }
    }

    pub fn set_getter(&mut self, get: Box<dyn Fn() -> f32 + Send>) {
        if let Ok(mut cbs) = self.cbs.lock() {
            cbs.on_get = Some(get);
        }
    }

    pub fn set_counter(&mut self, counter: Arc<AtomicU64>) {
        self.ext_counter = Some(counter);
    }

    pub fn set_changers(
        &mut self,
        change_start: Box<dyn FnMut() + Send>,
        change_set: Box<dyn FnMut(f32) + Send>,
        change_end: Box<dyn FnMut() + Send>,
    ) {
        if let Ok(mut cbs) = self.cbs.lock() {
            cbs.on_change_start = Some(change_start);
            cbs.on_change_set = Some(change_set);
            cbs.on_change_end = Some(change_end);
        }
    }
}

/// The rounding function for freq knobs (n_pit / d_pit)
macro_rules! r_norm {
    ($x: expr, $coarse: expr) => {
        if $coarse {
            ($x * 10.0).round() / 10.0
        } else {
            ($x * 100.0).round() / 100.0
        }
    };
}

impl ParamModel for ExtParam {
    fn get(&self) -> f32 {
        if let Ok(cbs) = self.cbs.lock() {
            cbs.on_get.as_ref().map(|get| (*get)()).unwrap_or(0.0)
        } else {
            0.0
        }
    }

    fn get_generation(&mut self) -> u64 {
        if let Some(ext_counter) = &self.ext_counter {
            ext_counter.load(Ordering::Relaxed)
        } else {
            0
        }
    }

    fn enabled(&self) -> bool {
        true
    }

    fn get_ui_range(&self) -> f32 {
        self.get()
    }

    fn get_ui_mod_amt(&self) -> Option<f32> {
        None
    }

    fn get_mod_amt(&self) -> Option<f32> {
        None
    }

    fn set_mod_amt(&mut self, _amt: Option<f32>) {}

    fn get_ui_steps(&self) -> (f32, f32) {
        (1.0 / 20.0, 1.0 / 100.0)
    }

    fn fmt(&self, buf: &mut [u8]) -> usize {
        let mut bw = std::io::BufWriter::new(buf);

        match write!(&mut bw, "{:6.4}", self.get()) {
            Ok(_) => bw.buffer().len(),
            _ => 0,
        }
    }

    fn fmt_mod(&self, _buf: &mut [u8]) -> usize {
        0
    }

    fn fmt_norm(&self, buf: &mut [u8]) -> usize {
        let mut bw = std::io::BufWriter::new(buf);

        match write!(bw, "{:6.4}", self.get()) {
            Ok(_) => bw.buffer().len(),
            _ => 0,
        }
    }

    fn fmt_name(&self, buf: &mut [u8]) -> usize {
        let mut bw = std::io::BufWriter::new(buf);

        match write!(bw, "{}", self.name) {
            Ok(_) => bw.buffer().len(),
            Err(_) => 0,
        }
    }

    fn fmt_norm_mod_to_string(&self) -> String {
        "".to_string()
    }

    fn get_denorm(&self) -> f32 {
        self.get()
    }

    fn set_denorm(&mut self, v: f32) {
        if let Ok(mut cbs) = self.cbs.lock() {
            cbs.on_change_start.as_mut().map(|cs| (*cs)());
            cbs.on_change_set.as_mut().map(|cs| (*cs)(v));
            cbs.on_change_end.as_mut().map(|cs| (*cs)());
        }
    }

    fn set_default(&mut self) {
        self.set_denorm(0.0);
    }

    fn change_start(&mut self) {
        if let Ok(mut cbs) = self.cbs.lock() {
            cbs.on_change_start.as_mut().map(|cs| (*cs)());
        }
    }

    fn change(&mut self, v: f32, res: ChangeRes) {
        let (min, max) = (0.0, 1.0);

        let v = match res {
            ChangeRes::Coarse => r_norm!(v.clamp(min, max), true),
            ChangeRes::Fine => r_norm!(v.clamp(min, max), false),
            ChangeRes::Free => v.clamp(min, max),
        };

        if let Ok(mut cbs) = self.cbs.lock() {
            cbs.on_change_set.as_mut().map(|cs| (*cs)(v));
        }
    }

    fn change_end(&mut self, v: f32, res: ChangeRes) {
        let (min, max) = (0.0, 1.0);

        let v = match res {
            ChangeRes::Coarse => r_norm!(v.clamp(min, max), true),
            ChangeRes::Fine => r_norm!(v.clamp(min, max), false),
            ChangeRes::Free => v.clamp(min, max),
        };

        if let Ok(mut cbs) = self.cbs.lock() {
            cbs.on_change_set.as_mut().map(|cs| (*cs)(v));
            cbs.on_change_end.as_mut().map(|cs| (*cs)());
        }
    }
}
