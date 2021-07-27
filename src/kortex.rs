// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use hexodsp::{Cell, CellDir, NodeId, ParamId};

pub enum UIMessage {
}

pub struct UIMsgEnv {
    msg:        UIMessage,
    node_id:    Option<NodeId>,
    param_id:   Option<ParamId>,
    cell_a:     Option<Cell>,
    cell_b:     Option<Cell>,
    dir_a:      Option<CellDir>,
    dir_b:      Option<CellDir>,
}

pub struct Kortex {
    messages:   Vec<UIMsgEnv>,
    work_queue: Option<Vec<UIMsgEnv>>,
}

impl Kortex {
    pub fn new() -> Self {
        Self {
            messages:   vec![],
            work_queue: Some(vec![]),
        }
    }

    pub fn emit(&mut self, msg: UIMsgEnv) {
        self.messages.push(msg);
    }

    pub fn start_work(&mut self) -> Option<Vec<UIMsgEnv>> {
        if let Some(wq) = self.work_queue.as_mut() {
            std::mem::swap(wq, &mut self.messages);
        }

        self.work_queue.take()
    }

    pub fn end_work(&mut self, v: Vec<UIMsgEnv>) {
        self.work_queue = Some(v);
    }
}
