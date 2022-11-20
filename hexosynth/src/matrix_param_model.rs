// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use hexotk::{ChangeRes, ParamModel};

use hexodsp::{Matrix, NodeInfo, ParamId};

use std::io::Write;
use std::sync::{Arc, Mutex};

pub struct KnobParam {
    matrix: Arc<Mutex<Matrix>>,
    param_id: ParamId,
    node_info: NodeInfo,
}

impl KnobParam {
    pub fn new(matrix: Arc<Mutex<Matrix>>, param_id: ParamId) -> Self {
        Self { matrix, param_id, node_info: NodeInfo::from(param_id.node_id().name()) }
    }

    pub fn with_ref<F, R: Default>(&self, fun: F) -> R
    where
        F: FnOnce(&mut Matrix, &ParamId) -> R,
    {
        match self.matrix.lock() {
            Ok(mut lock) => fun(&mut *lock, &self.param_id),
            Err(e) => {
                eprintln!("Couldn't lock matrix!: {}", e);
                R::default()
            }
        }
    }
}

impl ParamModel for KnobParam {
    fn get(&self) -> f32 {
        self.with_ref(|m, param_id| m.get_param(param_id).map(|a| a.f()).unwrap_or(0.0))
    }

    fn get_generation(&mut self) -> u64 {
        self.with_ref(|m, _param_id| m.get_generation() as u64)
    }

    /// Should return true if the UI for the parameter can be changed
    /// by the user. In HexoSynth this might return false if the
    /// corresponding input is controlled by an output port.
    fn enabled(&self) -> bool {
        if self.get_mod_amt().is_some() {
            true
        } else {
            if let Ok(m) = self.matrix.lock() {
                !m.param_input_is_used(self.param_id)
            } else {
                false
            }
        }
    }

    /// Should return a value in the range 0.0 to 1.0 for displayed knob position.
    /// For instance: a normalized value in the range -1.0 to 1.0 needs to be mapped
    /// to 0.0 to 1.0 by: `(x + 1.0) * 0.5`
    fn get_ui_range(&self) -> f32 {
        let v = self.get();

        if let Some(((min, max), _)) = self.param_id.param_min_max() {
            ((v - min) / (max - min)).abs()
        } else {
            v
        }
    }

    /// Should return the modulation amount for the 0..1 UI knob range.
    /// Internally you should transform that into the appropriate
    /// modulation amount in relation to what [get_ui_range] returns.
    fn get_ui_mod_amt(&self) -> Option<f32> {
        if let Some(((min, max), _)) = self.param_id.param_min_max() {
            self.get_mod_amt().map(|v| v / (max - min))
        } else {
            self.get_mod_amt()
        }
    }

    /// Should return the modulation amount like it will be applied to the
    /// inputs.
    fn get_mod_amt(&self) -> Option<f32> {
        if let Ok(m) = self.matrix.lock() {
            m.get_param_modamt(&self.param_id)
        } else {
            None
        }
    }

    /// Set the UI modulation amount like it will be used in the
    /// modulation later and be returned from [get_mod_amt].
    fn set_mod_amt(&mut self, amt: Option<f32>) {
        if let Ok(mut m) = self.matrix.lock() {
            // XXX: We ignore errors here, because setting a mod
            //      amount does indeed cause a matrix sync, but
            //      it does not change anything that might cause
            //      an error!
            let _ = m.set_param_modamt(self.param_id, amt);
        }
    }

    /// Should return a coarse step and a fine step for the normalized value.
    /// If none are returned, the UI will assume default steps of:
    ///
    /// * Default coarse: 0.05
    /// * Default fine: 0.01
    fn get_ui_steps(&self) -> (f32, f32) {
        if let Some(((min, max), (coarse, fine))) = self.param_id.param_min_max() {
            let delta = (max - min).abs();
            (delta / coarse, delta / fine)
        } else {
            (1.0 / 20.0, 1.0 / 100.0)
        }
    }

    fn fmt(&self, buf: &mut [u8]) -> usize {
        let mut bw = std::io::BufWriter::new(buf);

        match self.param_id.format(&mut bw, self.get()) {
            Some(Ok(_)) => bw.buffer().len(),
            _ => 0,
        }
    }

    fn fmt_mod(&self, buf: &mut [u8]) -> usize {
        let modamt = if let Some(ma) = self.get_mod_amt() {
            ma
        } else {
            return 0;
        };

        let mut bw = std::io::BufWriter::new(buf);

        match self.param_id.format(&mut bw, self.get() + modamt) {
            Some(Ok(_)) => bw.buffer().len(),
            _ => 0,
        }
    }

    fn fmt_norm(&self, buf: &mut [u8]) -> usize {
        let mut bw = std::io::BufWriter::new(buf);

        match write!(bw, "{:6.4}", self.get()) {
            Ok(_) => bw.buffer().len(),
            Err(_) => 0,
        }
    }

    fn fmt_name(&self, buf: &mut [u8]) -> usize {
        let mut bw = std::io::BufWriter::new(buf);

        match write!(bw, "{}", self.node_info.in_name(self.param_id.inp() as usize).unwrap_or("?"))
        {
            Ok(_) => bw.buffer().len(),
            Err(_) => 0,
        }
    }

    fn fmt_norm_mod_to_string(&self) -> String {
        if let Some(v) = self.get_mod_amt() {
            format!("{:6.3}", v)
        } else {
            "".to_string()
        }
    }

    fn get_denorm(&self) -> f32 {
        self.param_id.denorm(self.get())
    }

    fn set_denorm(&mut self, v: f32) {
        if let Ok(mut m) = self.matrix.lock() {
            m.set_param(self.param_id, self.param_id.norm(v).into())
        }
    }

    fn set_default(&mut self) {
        if let Ok(mut m) = self.matrix.lock() {
            let at = self.param_id.as_atom_def().into();
            m.set_param(self.param_id, at);

            // XXX: We ignore errors here, because setting a mod
            //      amount does indeed cause a matrix sync, but
            //      it does not change anything that might cause
            //      an error!
            let _ = m.set_param_modamt(self.param_id, None);
        }
    }

    fn change_start(&mut self) {}
    fn change(&mut self, v: f32, res: ChangeRes) {
        let pid = self.param_id;

        if let Some(((min, max), _)) = pid.param_min_max() {
            //d// println!(
            //d//     "CHANGE: {},{} ({}), min={}, max={}",
            //d//     id, v, single, min, max);
            let v = match res {
                ChangeRes::Coarse => pid.round(v.clamp(min, max), true),
                ChangeRes::Fine => pid.round(v.clamp(min, max), false),
                ChangeRes::Free => v.clamp(min, max),
            };

            if let Ok(mut m) = self.matrix.lock() {
                m.set_param(pid, v.into())
            }
        } else {
            if let Ok(mut m) = self.matrix.lock() {
                m.set_param(pid, v.into())
            }
        }
    }
    fn change_end(&mut self, v: f32, res: ChangeRes) {
        self.change(v, res);
    }
}
