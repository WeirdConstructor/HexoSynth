// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use hexotk::MButton;

use keyboard_types::Key;

#[derive(Debug, Clone)]
pub enum Msg {
    Key         { key: Key },
    UIBtn       { id: u32 },
    ClrSelect   { clr: i64 },
    MenuHover   { item_idx: usize },
    MenuClick   { item_idx: usize },
    MatrixClick { x: usize, y: usize, btn: MButton, modkey: bool },
    MenuMouseClick {
        x: f64,
        y: f64,
        btn: MButton
    },
    MatrixMouseClick {
        x: f64,
        y: f64,
        btn: MButton
    },
    CellDragged {
        btn: MButton,
        pos_a: (usize, usize),
        pos_b: (usize, usize),
        mouse_pos: (f64, f64),
    },
}

impl Msg {
    pub fn cell_drag(btn: MButton, pos_a: (usize, usize), pos_b: (usize, usize), mouse_pos: (f64, f64)) -> Self {
        Msg::CellDragged { btn, pos_a, pos_b, mouse_pos }
    }

    pub fn key(key: Key) -> Self { Msg::Key { key } }

    pub fn ui_btn(id: u32) -> Self { Msg::UIBtn { id } }

    pub fn clr_sel(clr: i64) -> Self { Msg::ClrSelect { clr } }

    pub fn menu_hover(item_idx: usize) -> Self { Msg::MenuHover { item_idx } }

    pub fn menu_click(item_idx: usize) -> Self { Msg::MenuClick { item_idx } }

    pub fn matrix_click(x: usize, y: usize, btn: MButton, modkey: bool) -> Self {
        Msg::MatrixClick { x, y, btn, modkey }
    }

    pub fn matrix_mouse_click(x: f64, y: f64, btn: MButton) -> Self {
        Msg::MatrixMouseClick { x, y, btn }
    }

    pub fn menu_mouse_click(x: f64, y: f64, btn: MButton) -> Self {
        Msg::MenuMouseClick { x, y, btn }
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
