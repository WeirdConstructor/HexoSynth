// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

use hexotk::{UI, open_window};
//pub mod ui;
//pub mod ui_ctrl;
//mod cluster;
//mod uimsg_queue;
//mod state;
//mod actions;
//mod menu;
//mod dyn_widgets;

//use ui_ctrl::UICtrlRef;

use raw_window_handle::RawWindowHandle;

use std::rc::Rc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::io::Write;

//pub use uimsg_queue::Msg;
pub use hexodsp::*;
//pub use hexotk::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initializes the default [Matrix] setup of HexoSynth.
///
/// This routine is used for example by the tests,
/// the VST2 and Jack Standalone versions to get a default
/// and commonly initialized Matrix and DSP executor ([NodeExecutor]).
///
/// It also creates a simple preset so the user won't start out
/// with an empty matrix.
pub fn init_hexosynth() -> (Matrix, NodeExecutor) {
    let (node_conf, node_exec) = nodes::new_node_engine();
    let (w, h) = (16, 16);
    let mut matrix = Matrix::new(node_conf, w, h);

    matrix.place(0, 1, Cell::empty(NodeId::Sin(0))
                       .out(Some(0), None, None));
    matrix.place(1, 0, Cell::empty(NodeId::Amp(0))
                       .out(Some(0), None, None)
                       .input(None, None, Some(0)));
    matrix.place(2, 0, Cell::empty(NodeId::Out(0))
                       .input(None, None, Some(0)));

    let gain_p = NodeId::Amp(0).inp_param("gain").unwrap();
    matrix.set_param(gain_p, gain_p.norm(0.06).into());

    if let Err(e) = load_patch_from_file(&mut matrix, "init.hxy") {
        println!("Error loading init.hxy: {:?}", e);
    }

    let _ = matrix.sync();

    (matrix, node_exec)
}

/// Configuration structure for [open_hexosynth_with_config].
#[derive(Debug, Clone, Default)]
pub struct OpenHexoSynthConfig {
    pub show_cursor: bool,
}

impl OpenHexoSynthConfig {
    pub fn new() -> Self {
        Self {
            show_cursor: false,
        }
    }
}

/// Opens the HexoSynth GUI window and initializes the UI.
///
/// * `parent` - The parent window, only used by the VST.
/// is usually used to drive the UI from the UI tests.
/// And also when out of band events need to be transmitted to
/// HexoSynth or the GUI.
/// * `matrix` - A shared thread safe reference to the
/// [Matrix]. Can be created eg. by [init_hexosynth] or directly
/// constructed.
pub fn open_hexosynth(
    parent: Option<RawWindowHandle>,
    matrix: Arc<Mutex<Matrix>>)
{
    open_hexosynth_with_config(
        parent,
        matrix,
        OpenHexoSynthConfig::default());
}

/// The same as [open_hexosynth] but with more configuration options, see also
/// [OpenHexoSynthConfig].
pub fn open_hexosynth_with_config(
    parent: Option<RawWindowHandle>,
    matrix: Arc<Mutex<Matrix>>,
    config: OpenHexoSynthConfig
) {
    open_window(
        "HexoSynth", 1400, 787,
        parent,
        Box::new(move || {
            let mut ui = Box::new(UI::new());
            ui
        }));
}
