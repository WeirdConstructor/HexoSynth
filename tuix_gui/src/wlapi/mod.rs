// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

pub mod hxdsp;
pub mod hex_knob;

pub use hxdsp::*;
pub use hex_knob::*;

#[macro_export]
macro_rules! arg_chk {
    ($args: expr, $count: expr, $name: literal) => {
        if $args.len() != $count {
            return Err(StackAction::panic_msg(format!(
                "{} called with wrong number of arguments",
                $name)));
        }
    }
}

#[macro_export]
macro_rules! wl_panic {
    ($str: literal) => {
        return Err(StackAction::panic_msg($str.to_string()));
    }
}

