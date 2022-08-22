// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use hexodsp::matrix::MatrixObserver;
use hexodsp::{Cell, HxMidiEvent, ParamId};
use wlambda::*;

use std::sync::Mutex;

use super::*;

pub struct MatrixRecorder {
    changes: Mutex<Vec<VVal>>,
}

impl MatrixRecorder {
    pub fn new() -> Self {
        Self { changes: Mutex::new(vec![]) }
    }

    pub fn get_records(&self) -> VVal {
        if let Ok(mut changes) = self.changes.lock() {
            if changes.is_empty() {
                VVal::None
            } else {
                let vec = VVal::vec();
                for c in changes.iter() {
                    vec.push(c.clone());
                }
                changes.clear();
                vec
            }
        } else {
            VVal::None
        }
    }
}

impl MatrixObserver for MatrixRecorder {
    fn update_prop(&self, key: &str) {
        if let Ok(mut changes) = self.changes.lock() {
            changes.push(VVal::pair(VVal::new_sym("matrix_property"), VVal::new_str(key)));
        }
    }

    fn update_monitor(&self, cell: &Cell) {
        if let Ok(mut changes) = self.changes.lock() {
            changes.push(VVal::pair(VVal::new_sym("matrix_monitor"), cell2vval(cell)));
        }
    }

    fn update_param(&self, param_id: &ParamId) {
        if let Ok(mut changes) = self.changes.lock() {
            changes.push(VVal::pair(VVal::new_sym("matrix_param"), param_id2vv(param_id.clone())));
        }
    }

    fn update_matrix(&self) {
        if let Ok(mut changes) = self.changes.lock() {
            changes.push(VVal::pair(VVal::new_sym("matrix_graph"), VVal::None));
        }
    }

    fn update_all(&self) {
        if let Ok(mut changes) = self.changes.lock() {
            changes.push(VVal::pair(VVal::new_sym("matrix_all"), VVal::None));
        }
    }

    fn midi_event(&self, midi_ev: HxMidiEvent) {
        if let Ok(mut changes) = self.changes.lock() {
            let ev_vv = match midi_ev {
                HxMidiEvent::NoteOn { channel, note, vel } => {
                    let v = VVal::map3(
                        "channel",
                        VVal::Int(channel as i64),
                        "note",
                        VVal::Int(note as i64),
                        "velocity",
                        VVal::Flt(vel as f64),
                    );
                    v.set_key_str("type", VVal::new_sym("note_on"));
                    v
                }
                HxMidiEvent::NoteOff { channel, note } => VVal::map3(
                    "type",
                    VVal::new_sym("note_off"),
                    "channel",
                    VVal::Int(channel as i64),
                    "note",
                    VVal::Int(note as i64),
                ),
                HxMidiEvent::CC { channel, cc, value } => {
                    let v = VVal::map3(
                        "channel",
                        VVal::Int(channel as i64),
                        "cc",
                        VVal::Int(cc as i64),
                        "value",
                        VVal::Flt(value as f64),
                    );
                    v.set_key_str("type", VVal::new_sym("cc"));
                    v
                }
            };
            changes.push(VVal::pair(VVal::new_sym("midi_event"), ev_vv));
        }
    }
}
