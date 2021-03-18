// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

static FAST_COS_TAB_LOG2_SIZE : usize = 9;
static FAST_COS_TAB_SIZE : usize      = 1 << FAST_COS_TAB_LOG2_SIZE; // =512
static mut FAST_COS_TAB : [f32; 513] = [0.0; 513];

pub fn init_cos_tab() {
    for i in 0..(FAST_COS_TAB_SIZE+1) {
        let phase : f32 =
            (i as f32)
            * ((std::f32::consts::PI * 2.0)
               / (FAST_COS_TAB_SIZE as f32));
        unsafe {
            // XXX: note: mutable statics can be mutated by multiple
            //      threads: aliasing violations or data races
            //      will cause undefined behavior
            FAST_COS_TAB[i] = phase.cos();
        }
    }
}

const PHASE_SCALE : f32 = 1.0_f32 / (std::f32::consts::PI * 2.0_f32);

pub fn fast_cos(mut x: f32) -> f32 {
    x = x.abs(); // cosine is symmetrical around 0, let's get rid of negative values

    // normalize range from 0..2PI to 1..2
    let phase = x * PHASE_SCALE;

    let index = FAST_COS_TAB_SIZE as f32 * phase;

    let fract = index.fract();
    let index = index.floor() as usize;

    unsafe {
        // XXX: note: mutable statics can be mutated by multiple
        //      threads: aliasing violations or data races
        //      will cause undefined behavior
        let left         = FAST_COS_TAB[index as usize];
        let right        = FAST_COS_TAB[index as usize + 1];

        return left + (right - left) * fract;
    }
}

pub fn fast_sin(x: f32) -> f32 {
    fast_cos(x - (std::f32::consts::PI / 2.0))
}

static mut WHITE_NOISE_TAB: [f64; 1024] = [0.0; 1024];

pub fn init_white_noise_tab() {
    let mut rng = RandGen::new();
    unsafe {
        for i in 0..WHITE_NOISE_TAB.len() {
            WHITE_NOISE_TAB[i as usize] = rng.next_open01();
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RandGen {
    r: [u64; 2],
}

// Taken from xoroshiro128 crate under MIT License
// Implemented by Matthew Scharley (Copyright 2016)
// https://github.com/mscharley/rust-xoroshiro128
pub fn next_xoroshiro128(state: &mut [u64; 2]) -> u64 {
    let s0: u64     = state[0];
    let mut s1: u64 = state[1];
    let result: u64 = s0.wrapping_add(s1);

    s1 ^= s0;
    state[0] = s0.rotate_left(55) ^ s1 ^ (s1 << 14); // a, b
    state[1] = s1.rotate_left(36); // c

    result
}

// Taken from rand::distributions
// Licensed under the Apache License, Version 2.0
// Copyright 2018 Developers of the Rand project.
pub fn u64_to_open01(u: u64) -> f64 {
    use core::f64::EPSILON;
    let float_size         = std::mem::size_of::<f64>() as u32 * 8;
    let fraction           = u >> (float_size - 52);
    let exponent_bits: u64 = (1023 as u64) << 52;
    f64::from_bits(fraction | exponent_bits) - (1.0 - EPSILON / 2.0)
}

impl RandGen {
    pub fn new() -> Self {
        RandGen {
            r: [0x193a6754a8a7d469, 0x97830e05113ba7bb],
        }
    }

    pub fn next(&mut self) -> u64 {
        next_xoroshiro128(&mut self.r)
    }

    pub fn next_open01(&mut self) -> f64 {
        u64_to_open01(self.next())
    }
}

pub fn mix(v1: f32, v2: f32, mix: f32) -> f32 {
    v1 * (1.0 - mix) + v2 * mix
}

pub fn clamp(f: f32, min: f32, max: f32) -> f32 {
         if f < min { min }
    else if f > max { max }
    else            {   f }
}

pub fn square_135(phase: f32) -> f32 {
      fast_sin(phase)
    + fast_sin(phase * 3.0) / 3.0
    + fast_sin(phase * 5.0) / 5.0
}

pub fn square_35(phase: f32) -> f32 {
      fast_sin(phase * 3.0) / 3.0
    + fast_sin(phase * 5.0) / 5.0
}

// note: MIDI note value?
pub fn note_to_freq(note: f32) -> f32 {
    440.0 * (2.0_f32).powf((note - 69.0) / 12.0)
}

// Ported from LMMS under GPLv2
// * DspEffectLibrary.h - library with template-based inline-effects
// * Copyright (c) 2006-2014 Tobias Doerffel <tobydox/at/users.sourceforge.net>
//
// Signal distortion
// gain:        0.1 - 5.0       default = 1.0
// threshold:   0.0 - 100.0     default = 0.8
// i:           signal
pub fn f_distort(gain: f32, threshold: f32, i: f32) -> f32 {
    gain * (
        i * ( i.abs() + threshold )
        / ( i * i + (threshold - 1.0) * i.abs() + 1.0 ))
}

// Ported from LMMS under GPLv2
// * DspEffectLibrary.h - library with template-based inline-effects
// * Copyright (c) 2006-2014 Tobias Doerffel <tobydox/at/users.sourceforge.net>
//
// Foldback Signal distortion
// gain:        0.1 - 5.0       default = 1.0
// threshold:   0.0 - 100.0     default = 0.8
// i:           signal
pub fn f_fold_distort(gain: f32, threshold: f32, i: f32) -> f32 {
    if i >= threshold || i < -threshold {
        gain
        * ((
            ((i - threshold) % threshold * 4.0).abs()
            - threshold * 2.0).abs()
           - threshold)
    } else {
        gain * i
    }
}

pub fn lerp(x: f32, a: f32, b: f32) -> f32 {
    (a * (1.0 - x)) + (b * x)
}

pub fn lerp64(x: f64, a: f64, b: f64) -> f64 {
    (a * (1.0 - x)) + (b * x)
}

pub fn p2range(x: f32, a: f32, b: f32) -> f32 {
    lerp(x, a, b)
}

pub fn p2range_exp(x: f32, a: f32, b: f32) -> f32 {
    let x = x * x;
    (a * (1.0 - x)) + (b * x)
}

pub fn p2range_exp4(x: f32, a: f32, b: f32) -> f32 {
    let x = x * x * x * x;
    (a * (1.0 - x)) + (b * x)
}


pub fn range2p(v: f32, a: f32, b: f32) -> f32 {
    ((v - a) / (b - a)).abs()
}

pub fn range2p_exp(v: f32, a: f32, b: f32) -> f32 {
    (((v - a) / (b - a)).abs()).sqrt()
}

pub fn range2p_exp4(v: f32, a: f32, b: f32) -> f32 {
    (((v - a) / (b - a)).abs()).sqrt().sqrt()
}

// gain: 24.0 - -90.0   default = 0.0
pub fn gain2coef(gain: f32) -> f32 {
    if gain > -90.0 {
        10.0_f32.powf(gain * 0.05)
    } else {
        0.0
    }
}

// quickerTanh / quickerTanh64 credits to mopo synthesis library:
// Under GPLv3 or any later.
// Little IO <littleioaudio@gmail.com>
// Matt Tytel <matthewtytel@gmail.com>
pub fn quicker_tanh64(v: f64) -> f64 {
    let square = v * v;
    v / (1.0 + square / (3.0 + square / 5.0))
}

pub fn quicker_tanh(v: f32) -> f32 {
    let square = v * v;
    v / (1.0 + square / (3.0 + square / 5.0))
}

// quickTanh / quickTanh64 credits to mopo synthesis library:
// Under GPLv3 or any later.
// Little IO <littleioaudio@gmail.com>
// Matt Tytel <matthewtytel@gmail.com>
pub fn quick_tanh64(v: f64) -> f64 {
    let abs_v = v.abs();
    let square = v * v;
    let num =
        v * (2.45550750702956
             + 2.45550750702956 * abs_v
             + square * (0.893229853513558
                         + 0.821226666969744 * abs_v));
    let den =
        2.44506634652299
        + (2.44506634652299 + square)
          * (v + 0.814642734961073 * v * abs_v).abs();

    num / den
}

pub fn quick_tanh(v: f32) -> f32 {
    let abs_v = v.abs();
    let square = v * v;
    let num =
        v * (2.45550750702956
             + 2.45550750702956 * abs_v
             + square * (0.893229853513558
                         + 0.821226666969744 * abs_v));
    let den =
        2.44506634652299
        + (2.44506634652299 + square)
          * (v + 0.814642734961073 * v * abs_v).abs();

    num / den
}

/// A helper function for exponential envelopes:
#[inline]
pub fn sqrt4_to_pow4(x: f32, v: f32) -> f32 {
    if v > 0.75 {
        let xsq1 = x.sqrt();
        let xsq = xsq1.sqrt();
        let v = (v - 0.75) * 4.0;
        xsq1 * (1.0 - v) + xsq * v

    } else if v > 0.5 {
        let xsq = x.sqrt();
        let v = (v - 0.5) * 4.0;
        x * (1.0 - v) + xsq * v

    } else if v > 0.25 {
        let xx = x * x;
        let v = (v - 0.25) * 4.0;
        x * v + xx * (1.0 - v)

    } else {
        let xx = x * x;
        let xxxx = xx * xx;
        let v = v * 4.0;
        xx * v + xxxx * (1.0 - v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_range2p_exp() {
        let a = p2range_exp(0.5, 1.0, 100.0);
        let x = range2p_exp(a, 1.0, 100.0);

        assert!((x - 0.5).abs() < std::f32::EPSILON);
    }

    #[test]
    fn check_range2p() {
        let a = p2range(0.5, 1.0, 100.0);
        let x = range2p(a, 1.0, 100.0);

        assert!((x - 0.5).abs() < std::f32::EPSILON);
    }
}
