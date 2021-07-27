// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

pub mod ui;
pub mod ui_ctrl;
mod uimsg_queue;
mod state;
mod actions;

use ui_ctrl::{UICtrlRef, UICellTrans};

use raw_window_handle::RawWindowHandle;

use std::rc::Rc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::io::Write;

pub use uimsg_queue::Msg;
pub use hexodsp::*;
pub use hexotk::*;

use hexotk::widgets::DialogModel;
use dsp::ParamId;

pub struct UIParams {
    params:         HashMap<AtomId, (ParamId, Atom)>,
    modamts:        HashMap<AtomId, Option<f32>>,
    variables:      HashMap<AtomId, (ParamId, Atom)>,
    /// Generation counter, to check for matrix updates.
    matrix_gen:     RefCell<usize>,
    ui_ctrl:        UICtrlRef,
}

impl UIParams {
    pub fn new(ui_ctrl: UICtrlRef) -> Self {
        let matrix_gen = ui_ctrl.with_matrix(|m| m.get_generation());

        let mut hsup =
            UIParams {
                ui_ctrl,
                params:     HashMap::new(),
                modamts:    HashMap::new(),
                variables:  HashMap::new(),
                matrix_gen: RefCell::new(matrix_gen),
            };

        hsup.sync_from_matrix();

        hsup
    }

    pub fn sync_from_matrix(&mut self) {
        // TODO: this could all lead to speed problems in the UI:
        //       the allocation might cause a (too long?) pause.
        //       if this is too slow, then matrix.sync() is probably also
        //       too slow and we need to do that on an extra thread.
        //       and all communication in UIParams needs to happen
        //       through an Arc<Mutex<HashMap<AtomId, ...>>>.
        let mut new_hm = HashMap::new();
        let mut new_ma = HashMap::new();

        *self.matrix_gen.borrow_mut() =
            self.ui_ctrl.with_matrix(|m| {
                m.for_each_atom(|unique_idx, param_id, satom, modamt| {
                    //d// println!(
                    //d//     "NODEID: {} => idx={}",
                    //d//     param_id.node_id(),
                    //d//     unique_idx);

                    new_hm.insert(
                        AtomId::new(unique_idx as u32, param_id.inp() as u32),
                        (param_id, satom.clone().into()));
                    new_ma.insert(
                        AtomId::new(unique_idx as u32, param_id.inp() as u32),
                        modamt);
                });

                m.get_generation()
            });

        self.params  = new_hm;
        self.modamts = new_ma;
    }

    pub fn get_param(&self, id: AtomId) -> Option<&(ParamId, Atom)> {
        if id.node_id() > PARAM_VARIABLES_SPACE {
            self.variables.get(&id)
        } else {
            self.params.get(&id)
        }
    }

    pub fn set_param(&mut self, id: AtomId, atom: Atom) {
        if id.node_id() > UI_CTRL_SPACE {
            let ui_ctrl = self.ui_ctrl.clone();

            if ui_ctrl.set_event(self, id, atom.clone()) {
                self.variables.insert(id, (ParamId::none(), atom));
            }

        } else if id.node_id() > PARAM_VARIABLES_SPACE {
            self.variables.insert(id, (ParamId::none(), atom));

        } else {
            let pid =
                if let Some((pid, _)) = self.params.get(&id) {
                    *pid
                } else {
                    return;
                };

            let atom =
                if let Some(((min, max), _)) = pid.param_min_max() {
                    if let Atom::Param(v) = atom {
                        Atom::param(v.clamp(min, max))
                    } else {
                        atom
                    }
                } else {
                    atom
                };

            self.params.insert(id, (pid, atom.clone()));
            self.ui_ctrl.with_matrix(move |m| m.set_param(pid, atom.into()));
        }
    }

    pub fn set_param_denorm(&mut self, id: AtomId, v: f32) {
        if id.node_id() > UI_CTRL_SPACE || id.node_id() > UI_CTRL_SPACE {
            self.set_param(id, v.into());

        } else {
            let pid =
                if let Some((pid, _)) = self.params.get(&id) {
                    *pid
                } else {
                    return;
                };

            self.set_param(id, pid.norm(v).into());
        }
    }

    pub fn set_param_modamt(&mut self, id: AtomId, amt: Option<f32>) {
        if id.node_id() > PARAM_VARIABLES_SPACE {
            return;
        }

        let pid =
            if let Some((pid, _)) = self.params.get(&id) {
                *pid
            } else {
                return;
            };

        if let Some(((min, max), _)) = pid.param_min_max() {
            self.modamts.insert(
                id, amt.map(|v| v.clamp(min - max, max - min)));
        } else {
            let (min, max) = (-1.0, 1.0);

            self.modamts.insert(
                id, amt.map(|v| v.clamp(min - max, max - min)));
        }

        let amt = self.modamts.get(&id).copied().flatten();
        self.ui_ctrl.with_matrix_catch_err(
            move |m| m.set_param_modamt(pid, amt));
    }

    pub fn get_param_modamt(&self, id: AtomId) -> Option<f32> {
        if id.node_id() > PARAM_VARIABLES_SPACE {
            return None;
        }

        *self.modamts.get(&id)?
    }
}

impl AtomDataModel for UIParams {
    fn get_phase_value(&self, id: AtomId) -> Option<f32> {
        let (pid, _atom) = self.get_param(id)?;
        self.ui_ctrl.with_matrix(|m|
            Some(m.phase_value_for(&pid.node_id())))
    }

    fn get_led_value(&self, id: AtomId) -> Option<f32> {
        let (pid, _atom) = self.get_param(id)?;
        self.ui_ctrl.with_matrix(|m|
            Some(m.led_value_for(&pid.node_id())))
    }

    fn enabled(&self, id: AtomId) -> bool {
        if self.get_param_modamt(id).is_some() {
            true
        } else if let Some((pid, _)) = self.params.get(&id) {
            self.ui_ctrl.with_matrix(|m|
                !m.param_input_is_used(*pid))
        } else {
            true
        }
    }

    fn check_sync(&mut self) {
        let cur_gen = self.ui_ctrl.with_matrix(|m| m.get_generation());

        if *self.matrix_gen.borrow() < cur_gen {
            self.sync_from_matrix();
        }

        let ui_ctrl = self.ui_ctrl.clone();
        ui_ctrl.ui_start_frame(self);
    }

    fn get(&self, id: AtomId) -> Option<&Atom> {
        Some(&self.get_param(id)?.1)
    }

    fn get_ui_range(&self, id: AtomId) -> Option<f32> {
        if let Some((pid, _)) = self.get_param(id) {
            let v = self.get(id)?.f();

            if let Some(((min, max), _)) = pid.param_min_max() {
                Some(((v - min) / (max - min)).abs())
            } else {
                Some(v)
            }
        } else {
            None
        }
    }

    fn get_ui_steps(&self, id: AtomId) -> Option<(f32, f32)> {
        if let Some((pid, _)) = self.get_param(id) {
            if let Some(((min, max), (coarse, fine))) = pid.param_min_max() {
                let delta = (max - min).abs();
                Some((delta / coarse, delta / fine))
            } else {
                Some((1.0 / 20.0, 1.0 / 100.0))
            }
        } else {
            None
        }
    }

    fn get_mod_amt(&self, id: AtomId) -> Option<f32> {
        self.get_param_modamt(id)
    }

    fn get_ui_mod_amt(&self, id: AtomId) -> Option<f32> {
        if let Some((pid, _)) = self.get_param(id) {
            let v = self.get_param_modamt(id)?;

            if let Some(((min, max), _)) = pid.param_min_max() {
                Some(v / (max - min))
            } else {
                Some(v)
            }
        } else {
            None
        }
    }

    fn set_mod_amt(&mut self, id: AtomId, amt: Option<f32>) {
        self.set_param_modamt(id, amt);
    }

    fn get_denorm(&self, id: AtomId) -> Option<f32> {
        let (pid, atom) = self.get_param(id)?;
        Some(pid.denorm(atom.f()))
    }

    fn set(&mut self, id: AtomId, v: Atom) {
        self.set_param(id, v);
    }

    fn set_denorm(&mut self, id: AtomId, v: f32) {
        self.set_param_denorm(id, v);
    }

    fn set_default(&mut self, id: AtomId) {
        if let Some((pid, _)) = self.get_param(id) {
            let at = pid.as_atom_def().into();
            self.set(id, at);
        }
    }

    fn change_start(&mut self, _id: AtomId) {
        //d// println!("CHANGE START: {}", id);
    }

    fn change(&mut self, id: AtomId, v: f32, _single: bool, res: ChangeRes) {
        //d// println!("CHANGE: {},{} ({})", id, v, single);
        if let Some((pid, _)) = self.get_param(id) {
            if let Some(((min, max), _)) = pid.param_min_max() {
                //d// println!(
                //d//     "CHANGE: {},{} ({}), min={}, max={}",
                //d//     id, v, single, min, max);
                let v =
                    match res {
                        ChangeRes::Coarse => pid.round(v.clamp(min, max), true),
                        ChangeRes::Fine   => pid.round(v.clamp(min, max), false),
                        ChangeRes::Free   => v.clamp(min, max),
                    };
                self.set(id, Atom::param(v));
            } else {
                self.set(id, Atom::param(v));
            }
        }
    }

    fn change_end(&mut self, id: AtomId, v: f32, res: ChangeRes) {
        //d// println!("CHANGE END: {},{}", id, v);
        if let Some((pid, _)) = self.get_param(id) {
            if let Some(((min, max), _)) = pid.param_min_max() {
                let v =
                    match res {
                        ChangeRes::Coarse => pid.round(v.clamp(min, max), true),
                        ChangeRes::Fine   => pid.round(v.clamp(min, max), false),
                        ChangeRes::Free   => v.clamp(min, max),
                    };
                self.set(id, Atom::param(v));
            }
        }
    }

    fn step_next(&mut self, id: AtomId) {
        //d// println!("STEP NEXT!!!: {}", id);

        if let Some((pid, Atom::Setting(i))) = self.get_param(id) {
            if let Some((min, max)) = pid.setting_min_max() {
                let new = i + 1;
                let new =
                    if new > max { min }
                    else { new };

                self.set(id, Atom::setting(new));
            }
        }
    }

    fn step_prev(&mut self, id: AtomId) {
        if let Some((pid, Atom::Setting(i))) = self.get_param(id) {
            if let Some((min, max)) = pid.setting_min_max() {
                let new = i - 1;
                let new =
                    if new < min { max }
                    else { new };

                self.set(id, Atom::setting(new));
            }
        }
    }

    fn fmt_norm(&self, id: AtomId, buf: &mut [u8]) -> usize {
        let mut bw = std::io::BufWriter::new(buf);

        if let Some((_, atom)) = self.get_param(id) {
            match write!(bw, "{:6.4}", atom.f()) {
                Ok(_)  => bw.buffer().len(),
                Err(_) => 0,
            }
        } else {
            0
        }
    }

    fn fmt_mod(&self, id: AtomId, buf: &mut [u8]) -> usize {
        let modamt =
            if let Some(ma) = self.get_mod_amt(id) {
                ma
            } else {
                return 0;
            };

        let mut bw = std::io::BufWriter::new(buf);

        if let Some((pid, atom)) = self.get_param(id) {
            match pid.format(&mut bw, atom.f() + modamt) {
                Some(Ok(_)) => bw.buffer().len(),
                _ => 0,
            }
        } else {
            0
        }
    }

    fn fmt(&self, id: AtomId, buf: &mut [u8]) -> usize {
        let mut bw = std::io::BufWriter::new(buf);

        if let Some((pid, atom)) = self.get_param(id) {
            match pid.format(&mut bw, atom.f()) {
                Some(Ok(_)) => bw.buffer().len(),
                _ => 0,
            }
        } else {
            0
        }
    }
}

pub const PARAM_VARIABLES_SPACE    : u32   = 1000;
pub const UI_CTRL_SPACE            : u32   = 100000;

pub const NODE_MATRIX_ID           : u32   = 9999;
pub const NODE_PANEL_ID            : u32   = 9998;
pub const UTIL_PANEL_ID            : usize = 9997;
pub const PATTERN_PANEL_ID         : usize = 9996;
pub const PATTERN_VIEW_ID          : usize = 9995;
pub const UTIL_PANEL_TOP_ID        : usize = 9994;
pub const UTIL_PANEL_VER_ID        : usize = 9993;
pub const HELP_TABS_ID             : u32   = 7000;
pub const DIALOG_ID                : u32   = 90001;
pub const DIALOG_OK_ID             : u32   = 99;

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

/// Opens the HexoSynth GUI window and initializes the UI.
///
/// * `parent` - The parent window, only used by the VST.
/// * `drv` - Lets you pass an optional [Driver], a [Driver]
/// is usually used to drive the UI from the UI tests.
/// And also when out of band events need to be transmitted to
/// HexoSynth or the GUI.
/// * `matrix` - A shared thread safe reference to the
/// [Matrix]. Can be created eg. by [init_hexosynth] or directly
/// constructed.
pub fn open_hexosynth(
    parent: Option<RawWindowHandle>,
    drv:    Option<Driver>,
    matrix: Arc<Mutex<Matrix>>
) {
    use hexotk::widgets::{Dialog, DialogData};
    use crate::ui::matrix::NodeMatrixData;

    open_window(
        "HexoSynth", 1400, 787,
        parent,
        Box::new(move || {
            let dialog_model = Rc::new(RefCell::new(DialogModel::new()));
            let wt_diag      = Rc::new(Dialog::new());

            let ui_ctrl = UICtrlRef::new(matrix, dialog_model.clone());

            let drv =
                if let Some(drv) = drv {
                    drv
                } else {
                    let (drv, _drv_frontend) = Driver::new();
                    drv
                };

            (drv, Box::new(UI::new(
                Box::new(NodeMatrixData::new(
                    ui_ctrl.clone(),
                    UIPos::center(12, 12),
                    NODE_MATRIX_ID)),
                Box::new(wbox!(
                    wt_diag, 90000.into(), center(12, 12),
                    DialogData::new(
                        DIALOG_ID,
                        AtomId::new(DIALOG_ID, DIALOG_OK_ID),
                        dialog_model))),
                Box::new(UIParams::new(ui_ctrl)),
                (1400.0, 787.0),
            )))
    }));
}
