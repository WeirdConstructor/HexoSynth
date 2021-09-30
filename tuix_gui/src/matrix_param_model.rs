// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::hexknob::ParamModel;

use hexodsp::{Matrix, NodeId, Cell, CellDir};
use hexodsp::matrix::MatrixError;

struct ParamStore {
    matrix_gen:     RefCell<usize>,
    matrix:         Arc<Mutex<Matrix>>,
    recent_err:     Option<MatrixError>,

    params:         HashMap<AtomId, (ParamId, Atom)>,
    modamts:        HashMap<AtomId, Option<f32>>,
    variables:      HashMap<AtomId, (ParamId, Atom)>,
}

struct KnobParamModel {
    store:    Rc<RefCell<ParamStore>>,
    param_id: ParamId,
}

//pub struct UIParams {
//    params:         HashMap<AtomId, (ParamId, Atom)>,
//    modamts:        HashMap<AtomId, Option<f32>>,
//    variables:      HashMap<AtomId, (ParamId, Atom)>,
//    /// Generation counter, to check for matrix updates.
//    matrix_gen:     RefCell<usize>,
//    ui_ctrl:        UICtrlRef,
//}
//
//impl UIParams {
//    pub fn new(ui_ctrl: UICtrlRef) -> Self {
//        let matrix_gen = ui_ctrl.with_matrix(|m| m.get_generation());
//
//        let mut hsup =
//            UIParams {
//                ui_ctrl,
//                params:     HashMap::new(),
//                modamts:    HashMap::new(),
//                variables:  HashMap::new(),
//                matrix_gen: RefCell::new(matrix_gen),
//            };
//
//        hsup.sync_from_matrix();
//
//        hsup
//    }
//
//    pub fn sync_from_matrix(&mut self) {
//        // TODO: this could all lead to speed problems in the UI:
//        //       the allocation might cause a (too long?) pause.
//        //       if this is too slow, then matrix.sync() is probably also
//        //       too slow and we need to do that on an extra thread.
//        //       and all communication in UIParams needs to happen
//        //       through an Arc<Mutex<HashMap<AtomId, ...>>>.
//        let mut new_hm = HashMap::new();
//        let mut new_ma = HashMap::new();
//
//        *self.matrix_gen.borrow_mut() =
//            self.ui_ctrl.with_matrix(|m| {
//                m.for_each_atom(|unique_idx, param_id, satom, modamt| {
//                    //d// println!(
//                    //d//     "NODEID: {} => idx={}",
//                    //d//     param_id.node_id(),
//                    //d//     unique_idx);
//
//                    new_hm.insert(
//                        AtomId::new(unique_idx as u32, param_id.inp() as u32),
//                        (param_id, satom.clone().into()));
//                    new_ma.insert(
//                        AtomId::new(unique_idx as u32, param_id.inp() as u32),
//                        modamt);
//                });
//
//                m.get_generation()
//            });
//
//        self.params  = new_hm;
//        self.modamts = new_ma;
//
//        let ui_ctrl = self.ui_ctrl.clone();
//        ui_ctrl.sync_from_matrix(self);
//    }
//
//    pub fn get_param(&self, id: AtomId) -> Option<&(ParamId, Atom)> {
//        if id.node_id() > PARAM_VARIABLES_SPACE {
//            self.variables.get(&id)
//        } else {
//            self.params.get(&id)
//        }
//    }
//
//    pub fn set_var(&mut self, id: AtomId, atom: Atom) {
//        if id.node_id() > PARAM_VARIABLES_SPACE {
//            self.variables.insert(id, (ParamId::none(), atom));
//        }
//    }
//
//    pub fn set_param(&mut self, id: AtomId, atom: Atom) {
//        if id.node_id() > UI_CTRL_SPACE {
//            let ui_ctrl = self.ui_ctrl.clone();
//
//            if ui_ctrl.set_event(self, id, atom.clone()) {
//                self.variables.insert(id, (ParamId::none(), atom));
//            }
//
//        } else if id.node_id() > PARAM_VARIABLES_SPACE {
//            self.variables.insert(id, (ParamId::none(), atom));
//
//        } else {
//            let pid =
//                if let Some((pid, _)) = self.params.get(&id) {
//                    *pid
//                } else {
//                    return;
//                };
//
//            let atom =
//                if let Some(((min, max), _)) = pid.param_min_max() {
//                    if let Atom::Param(v) = atom {
//                        Atom::param(v.clamp(min, max))
//                    } else {
//                        atom
//                    }
//                } else {
//                    atom
//                };
//
//            self.params.insert(id, (pid, atom.clone()));
//            self.ui_ctrl.with_matrix(move |m| m.set_param(pid, atom.into()));
//        }
//    }
//
//    pub fn set_param_denorm(&mut self, id: AtomId, v: f32) {
//        if id.node_id() > UI_CTRL_SPACE || id.node_id() > UI_CTRL_SPACE {
//            self.set_param(id, v.into());
//
//        } else {
//            let pid =
//                if let Some((pid, _)) = self.params.get(&id) {
//                    *pid
//                } else {
//                    return;
//                };
//
//            self.set_param(id, pid.norm(v).into());
//        }
//    }
//
//    pub fn set_param_modamt(&mut self, id: AtomId, amt: Option<f32>) {
//        if id.node_id() > PARAM_VARIABLES_SPACE {
//            return;
//        }
//
//        let pid =
//            if let Some((pid, _)) = self.params.get(&id) {
//                *pid
//            } else {
//                return;
//            };
//
//        if let Some(((min, max), _)) = pid.param_min_max() {
//            self.modamts.insert(
//                id, amt.map(|v| v.clamp(min - max, max - min)));
//        } else {
//            let (min, max) = (-1.0, 1.0);
//
//            self.modamts.insert(
//                id, amt.map(|v| v.clamp(min - max, max - min)));
//        }
//
//        let amt = self.modamts.get(&id).copied().flatten();
//        self.ui_ctrl.with_matrix_catch_err(
//            move |m| m.set_param_modamt(pid, amt));
//    }
//
//    pub fn get_param_modamt(&self, id: AtomId) -> Option<f32> {
//        if id.node_id() > PARAM_VARIABLES_SPACE {
//            return None;
//        }
//
//        *self.modamts.get(&id)?
//    }
//}
//
//impl AtomDataModel for UIParams {
//    fn get_phase_value(&self, id: AtomId) -> Option<f32> {
//        let (pid, _atom) = self.get_param(id)?;
//        self.ui_ctrl.with_matrix(|m|
//            Some(m.phase_value_for(&pid.node_id())))
//    }
//
//    fn get_led_value(&self, id: AtomId) -> Option<f32> {
//        let (pid, _atom) = self.get_param(id)?;
//        self.ui_ctrl.with_matrix(|m|
//            Some(m.led_value_for(&pid.node_id())))
//    }
//
//    fn enabled(&self, id: AtomId) -> bool {
//        if self.get_param_modamt(id).is_some() {
//            true
//        } else if let Some((pid, _)) = self.params.get(&id) {
//            self.ui_ctrl.with_matrix(|m|
//                !m.param_input_is_used(*pid))
//        } else {
//            true
//        }
//    }
//
//    fn check_sync(&mut self) {
//        let cur_gen = self.ui_ctrl.with_matrix(|m| m.get_generation());
//
//        let ui_ctrl = self.ui_ctrl.clone();
//
//        if *self.matrix_gen.borrow() < cur_gen {
//            self.sync_from_matrix();
//        }
//
//        ui_ctrl.ui_start_frame(self);
//    }
//
//    fn get(&self, id: AtomId) -> Option<&Atom> {
//        Some(&self.get_param(id)?.1)
//    }
//
//    fn get_ui_range(&self, id: AtomId) -> Option<f32> {
//        if let Some((pid, _)) = self.get_param(id) {
//            let v = self.get(id)?.f();
//
//            if let Some(((min, max), _)) = pid.param_min_max() {
//                Some(((v - min) / (max - min)).abs())
//            } else {
//                Some(v)
//            }
//        } else {
//            None
//        }
//    }
//
//    fn get_ui_steps(&self, id: AtomId) -> Option<(f32, f32)> {
//        if let Some((pid, _)) = self.get_param(id) {
//            if let Some(((min, max), (coarse, fine))) = pid.param_min_max() {
//                let delta = (max - min).abs();
//                Some((delta / coarse, delta / fine))
//            } else {
//                Some((1.0 / 20.0, 1.0 / 100.0))
//            }
//        } else {
//            None
//        }
//    }
//
//    fn get_mod_amt(&self, id: AtomId) -> Option<f32> {
//        self.get_param_modamt(id)
//    }
//
//    fn get_ui_mod_amt(&self, id: AtomId) -> Option<f32> {
//        if let Some((pid, _)) = self.get_param(id) {
//            let v = self.get_param_modamt(id)?;
//
//            if let Some(((min, max), _)) = pid.param_min_max() {
//                Some(v / (max - min))
//            } else {
//                Some(v)
//            }
//        } else {
//            None
//        }
//    }
//
//    fn set_mod_amt(&mut self, id: AtomId, amt: Option<f32>) {
//        self.set_param_modamt(id, amt);
//    }
//
//    fn get_denorm(&self, id: AtomId) -> Option<f32> {
//        let (pid, atom) = self.get_param(id)?;
//        Some(pid.denorm(atom.f()))
//    }
//
//    fn set(&mut self, id: AtomId, v: Atom) {
//        self.set_param(id, v);
//    }
//
//    fn set_denorm(&mut self, id: AtomId, v: f32) {
//        self.set_param_denorm(id, v);
//    }
//
//    fn set_default(&mut self, id: AtomId) {
//        if let Some((pid, _)) = self.get_param(id) {
//            let at = pid.as_atom_def().into();
//            self.set(id, at);
//        }
//    }
//
//    fn change_start(&mut self, _id: AtomId) {
//        //d// println!("CHANGE START: {}", id);
//    }
//
//    fn change(&mut self, id: AtomId, v: f32, _single: bool, res: ChangeRes) {
//        //d// println!("CHANGE: {},{} ({})", id, v, single);
//        if let Some((pid, _)) = self.get_param(id) {
//            if let Some(((min, max), _)) = pid.param_min_max() {
//                //d// println!(
//                //d//     "CHANGE: {},{} ({}), min={}, max={}",
//                //d//     id, v, single, min, max);
//                let v =
//                    match res {
//                        ChangeRes::Coarse => pid.round(v.clamp(min, max), true),
//                        ChangeRes::Fine   => pid.round(v.clamp(min, max), false),
//                        ChangeRes::Free   => v.clamp(min, max),
//                    };
//                self.set(id, Atom::param(v));
//            } else {
//                self.set(id, Atom::param(v));
//            }
//        }
//    }
//
//    fn change_end(&mut self, id: AtomId, v: f32, res: ChangeRes) {
//        //d// println!("CHANGE END: {},{}", id, v);
//        if let Some((pid, _)) = self.get_param(id) {
//            if let Some(((min, max), _)) = pid.param_min_max() {
//                let v =
//                    match res {
//                        ChangeRes::Coarse => pid.round(v.clamp(min, max), true),
//                        ChangeRes::Fine   => pid.round(v.clamp(min, max), false),
//                        ChangeRes::Free   => v.clamp(min, max),
//                    };
//                self.set(id, Atom::param(v));
//            }
//        }
//    }
//
//    fn step_next(&mut self, id: AtomId) {
//        //d// println!("STEP NEXT!!!: {}", id);
//
//        if let Some((pid, Atom::Setting(i))) = self.get_param(id) {
//            if let Some((min, max)) = pid.setting_min_max() {
//                let new = i + 1;
//                let new =
//                    if new > max { min }
//                    else { new };
//
//                self.set(id, Atom::setting(new));
//            }
//        }
//    }
//
//    fn step_prev(&mut self, id: AtomId) {
//        if let Some((pid, Atom::Setting(i))) = self.get_param(id) {
//            if let Some((min, max)) = pid.setting_min_max() {
//                let new = i - 1;
//                let new =
//                    if new < min { max }
//                    else { new };
//
//                self.set(id, Atom::setting(new));
//            }
//        }
//    }
//
//    fn fmt_norm(&self, id: AtomId, buf: &mut [u8]) -> usize {
//        let mut bw = std::io::BufWriter::new(buf);
//
//        if let Some((_, atom)) = self.get_param(id) {
//            match write!(bw, "{:6.4}", atom.f()) {
//                Ok(_)  => bw.buffer().len(),
//                Err(_) => 0,
//            }
//        } else {
//            0
//        }
//    }
//
//    fn fmt_mod(&self, id: AtomId, buf: &mut [u8]) -> usize {
//        let modamt =
//            if let Some(ma) = self.get_mod_amt(id) {
//                ma
//            } else {
//                return 0;
//            };
//
//        let mut bw = std::io::BufWriter::new(buf);
//
//        if let Some((pid, atom)) = self.get_param(id) {
//            match pid.format(&mut bw, atom.f() + modamt) {
//                Some(Ok(_)) => bw.buffer().len(),
//                _ => 0,
//            }
//        } else {
//            0
//        }
//    }
//
//    fn fmt(&self, id: AtomId, buf: &mut [u8]) -> usize {
//        let mut bw = std::io::BufWriter::new(buf);
//
//        if let Some((pid, atom)) = self.get_param(id) {
//            match pid.format(&mut bw, atom.f()) {
//                Some(Ok(_)) => bw.buffer().len(),
//                _ => 0,
//            }
//        } else {
//            0
//        }
//    }
//}
//
