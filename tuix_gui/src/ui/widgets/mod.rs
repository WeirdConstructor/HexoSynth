// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

mod connector;
mod cv_array;
mod hexgrid;
mod hexknob;
mod octave_keys;
mod pattern_editor;
mod block_code;

pub use connector::*;
pub use cv_array::*;
pub use hexgrid::*;
pub use hexknob::*;
pub use octave_keys::*;
pub use pattern_editor::*;
pub use block_code::*;

#[macro_export]
macro_rules! hxclr {
    ($i: expr) => {
        (
            ($i >> 16 & 0xFF) as f32 / 255.0,
            ($i >> 8  & 0xFF) as f32 / 255.0,
            ($i       & 0xFF) as f32 / 255.0,
        )
    }
}

pub fn darken_clr(depth: u32, clr: (f32, f32, f32)) -> (f32, f32, f32) {
    if depth == 0 { return clr; }
    ((clr.0 * (1.0 / (1.2_f32).powf(depth as f32))).clamp(0.0, 1.0),
     (clr.1 * (1.0 / (1.2_f32).powf(depth as f32))).clamp(0.0, 1.0),
     (clr.2 * (1.0 / (1.2_f32).powf(depth as f32))).clamp(0.0, 1.0))
}

pub fn lighten_clr(depth: u32, clr: (f32, f32, f32)) -> (f32, f32, f32) {
    if depth == 0 { return clr; }
    ((clr.0 * (1.2_f32).powf(depth as f32)).clamp(0.0, 1.0),
     (clr.1 * (1.2_f32).powf(depth as f32)).clamp(0.0, 1.0),
     (clr.2 * (1.2_f32).powf(depth as f32)).clamp(0.0, 1.0))
}

pub fn tpl2clr(clr: (f32, f32, f32)) -> tuix::Color {
    tuix::Color::rgb(
        (clr.0 * 255.0) as u8,
        (clr.1 * 255.0) as u8,
        (clr.2 * 255.0) as u8)
}

pub const UI_BOX_H          : f32 = 200.0;
pub const UI_BOX_BORD       : f32 =   3.0;
pub const UI_MARGIN         : f32 =   4.0;
pub const UI_PADDING        : f32 =   6.0;
pub const UI_ELEM_TXT_H     : f32 =  16.0;
pub const UI_SAFETY_PAD     : f32 =   1.0;

pub const UI_BG_CLR               : (f32, f32, f32) = hxclr!(0x414a51); // 473f49
pub const UI_BG2_CLR              : (f32, f32, f32) = hxclr!(0x4b535a); // 594f5d
pub const UI_BG3_CLR              : (f32, f32, f32) = hxclr!(0x545b61); // 645868
pub const UI_TXT_CLR              : (f32, f32, f32) = hxclr!(0xdcdcf0);
pub const UI_BORDER_CLR           : (f32, f32, f32) = hxclr!(0x163239); // 2b0530);
pub const UI_LBL_BG_CLR           : (f32, f32, f32) = hxclr!(0x111920); // hxclr!(0x16232f); // 1a2733); // 200e1f);
pub const UI_LBL_BG_ALT_CLR       : (f32, f32, f32) = hxclr!(0x2d4d5e); // 323237
pub const UI_ACCENT_CLR           : (f32, f32, f32) = hxclr!(0x922f93); // b314aa);
pub const UI_ACCENT_DARK_CLR      : (f32, f32, f32) = hxclr!(0x1e333d); // 4d184a); // 4d184a);
pub const UI_ACCENT_BG1_CLR       : (f32, f32, f32) = hxclr!(0x111920); // UI_LBL_BG_CLR; // hxclr!(0x111920); // UI_LBL_BG_CLR; // hxclr!(0x27091b); // 381c38); // 200e1f);
pub const UI_ACCENT_BG2_CLR       : (f32, f32, f32) = hxclr!(0x192129); // 2c132a);
pub const UI_PRIM_CLR             : (f32, f32, f32) = hxclr!(0x03fdcb); // 69e8ed
pub const UI_PRIM2_CLR            : (f32, f32, f32) = hxclr!(0x228f9d); // 1aaeb3
pub const UI_HLIGHT_CLR           : (f32, f32, f32) = hxclr!(0xecf9ce); // e9f840
pub const UI_HLIGHT2_CLR          : (f32, f32, f32) = hxclr!(0xbcf9cd); // b5c412
pub const UI_SELECT_CLR           : (f32, f32, f32) = hxclr!(0xd73988); // 0xdc1821);
pub const UI_INACTIVE_CLR         : (f32, f32, f32) = hxclr!(0x6f8782);
pub const UI_INACTIVE2_CLR        : (f32, f32, f32) = hxclr!(0xa6dbd0);
