// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::ui_ctrl::UICtrlRef;

use hexodsp::{Cell, CellDir, NodeId, ParamId};
use hexotk::{MButton};

use keyboard_types::Key;

#[derive(Debug, Clone)]
pub enum Msg {
    Key     { key: Key },
    UIBtn   { id: u32 },
    CellDragged {
        btn: MButton,
        pos_a: (usize, usize),
        pos_b: (usize, usize),
    },
}

impl Msg {
    pub fn cell_drag(btn: MButton, pos_a: (usize, usize), pos_b: (usize, usize)) -> Self {
        Msg::CellDragged { btn, pos_a, pos_b }
    }

    pub fn key(key: Key) -> Self { Msg::Key { key } }

    pub fn ui_btn(id: u32) -> Self {
        Msg::UIBtn { id }
    }
}

pub struct UIMsgQueue {
    messages:   Vec<Msg>,
    work_queue: Option<Vec<Msg>>,
}

impl UIMsgQueue {
    pub fn new() -> Self {
        Self {
            messages:   vec![],
            work_queue: Some(vec![]),
        }
    }

    pub fn has_new_messages(&self) -> bool {
        !self.messages.is_empty()
    }

    pub fn emit(&mut self, msg: Msg) {
        self.messages.push(msg);
    }

    pub fn start_work(&mut self) -> Option<Vec<Msg>> {
        if let Some(wq) = self.work_queue.as_mut() {
            std::mem::swap(wq, &mut self.messages);
        }

        self.work_queue.take()
    }

    pub fn end_work(&mut self, mut v: Vec<Msg>) {
        v.clear();
        self.work_queue = Some(v);
    }
}
