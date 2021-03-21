// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

const SMOOTHING_TIME_MS : f32 = 10.0;

pub struct Smoother {
    slope_samples:  usize,
    value:          f32,
    inc:            f32,
    target:         f32,
    count:          usize,
    done:           bool,
}

impl Smoother {
    pub fn new() -> Self {
        Self {
            slope_samples:  0,
            value:          0.0,
            inc:            0.0,
            count:          0,
            target:         0.0,
            done:           true,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.slope_samples = ((sr * SMOOTHING_TIME_MS) / 1000.0).ceil() as usize;
    }

    #[inline]
    pub fn is_done(&self) -> bool { self.done }

    #[inline]
    #[allow(dead_code)]
    pub fn stop(&mut self) { self.done = true; }

    #[inline]
    pub fn set(&mut self, current: f32, target: f32) {
        self.value  = current;
        self.count  = self.slope_samples;
        self.inc    = (target - current) / (self.count as f32);
        self.target = target;
        self.done   = false;
    }

    #[inline]
    pub fn next(&mut self) -> f32 {
        //d// println!("NEXT: count={}, value={:6.3} inc={:6.4}",
        //d//          self.count,
        //d//          self.value,
        //d//          self.inc);
        if self.count == 0 {
            self.done = true;

            self.target
        } else {
            self.value += self.inc;
            self.count -= 1;
            self.value
        }
    }
}

pub struct PerfTimer {
    lbl:    &'static str,
    i:      std::time::Instant,
    off:    bool,
        // let tb = std::time::Instant::now();
        // let ta = std::time::Instant::now().duration_since(ta);
        // let tb = std::time::Instant::now().duration_since(tb);
        // println!("ta Elapsed: {:?}", ta);
}

impl PerfTimer {
    #[inline]
    pub fn off(mut self) -> Self {
        self.off = true;
        self
    }


    #[inline]
    pub fn new(lbl: &'static str) -> Self {
        Self {
            lbl,
            i: std::time::Instant::now(),
            off: false,
        }
    }

    #[inline]
    pub fn print(&mut self, lbl2: &str) {
        if self.off { return; }

        let t = std::time::Instant::now().duration_since(self.i);
        println!("*** PERF[{}/{}] {:?}", self.lbl, lbl2, t);
        self.i = std::time::Instant::now();
    }
}
