// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::ui::*;

use tuix::*;
use femtovg::FontId;

use std::rc::Rc;
use std::cell::RefCell;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VisBlock {
    rows:    usize,
    lbl:     String,
    inputs:  Vec<Option<String>>,
    outputs: Vec<Option<String>>,
}

pub trait BlockCodeModel {
    fn block_at(&self, x: usize, y: usize) -> Option<&VisBlock>;
}

pub struct DummyBlockCode {
    blocks: HashMap<(usize, usize), VisBlock>,
}

impl DummyBlockCode {
    pub fn new() -> Self {
        let mut s = Self {
            blocks: HashMap::new(),
        };

        s.blocks.insert((1, 2), VisBlock {
            rows: 3,
            lbl: "get: x".to_string(),
            inputs:  vec![
                None,
                Some("app".to_string()),
                Some("inp".to_string())],
            outputs: vec![None, None, Some("->".to_string())],
        });
        s.blocks.insert((2, 4), VisBlock {
            rows: 1,
            lbl: "sin".to_string(),
            inputs:  vec![Some("in".to_string())],
            outputs: vec![Some("->".to_string())],
        });
        s.blocks.insert((2, 3), VisBlock {
            rows: 1,
            lbl: "1.2".to_string(),
            inputs:  vec![None],
            outputs: vec![Some("->".to_string())],
        });
        s.blocks.insert((3, 3), VisBlock {
            rows: 2,
            lbl: "+".to_string(),
            inputs:  vec![
                Some("a".to_string()),
                Some("b".to_string())],
            outputs: vec![None, Some("->".to_string())],
        });

        s
    }
}

impl BlockCodeModel for DummyBlockCode {
    fn block_at(&self, x: usize, y: usize) -> Option<&VisBlock> {
        self.blocks.get(&(x, y))
    }
}

#[derive(Clone)]
pub enum BlockCodeMessage {
    SetCode(Rc<RefCell<dyn BlockCodeModel>>),
}

pub struct BlockCode {
    font_size:      f32,
    font:           Option<FontId>,
    font_mono:      Option<FontId>,
    code:           Rc<RefCell<dyn BlockCodeModel>>,

    block_size:     f32,

    on_change:      Option<Box<dyn Fn(&mut Self, &mut State, Entity, (usize, usize))>>,
    on_hover:       Option<Box<dyn Fn(&mut Self, &mut State, Entity, bool, usize)>>,
}

impl BlockCode {
    pub fn new() -> Self {
        Self {
            font_size:      14.0,
            font:           None,
            font_mono:      None,
            code:           Rc::new(RefCell::new(DummyBlockCode::new())),

            block_size:     30.0,

            on_change:      None,
            on_hover:       None,
        }
    }

    pub fn on_change<F>(mut self, on_change: F) -> Self
    where
        F: 'static + Fn(&mut Self, &mut State, Entity, (usize, usize)),
    {
        self.on_change = Some(Box::new(on_change));

        self
    }

    pub fn on_hover<F>(mut self, on_hover: F) -> Self
    where
        F: 'static + Fn(&mut Self, &mut State, Entity, bool, usize),
    {
        self.on_hover = Some(Box::new(on_hover));

        self
    }
}

fn draw_markers(p: &mut FemtovgPainter, x: f32, y: f32, block_w: f32, block_h: f32, marker_px: f32) {
    p.path_stroke(
        1.0,
        UI_ACCENT_DARK_CLR,
        &mut ([
            (x,             y + marker_px),
            (x,             y),
            (x + marker_px, y),
        ].iter().copied()
         .map(|p| (p.0.floor() + 0.5, p.1.floor() + 0.5))), false);

    p.path_stroke(
        1.0,
        UI_ACCENT_DARK_CLR,
        &mut ([
            (block_w + x - marker_px, y),
            (block_w + x,             y),
            (block_w + x,             y + marker_px),
        ].iter().copied()
         .map(|p| (p.0.floor() - 0.5, p.1.floor() + 0.5))), false);

    p.path_stroke(
        1.0,
        UI_ACCENT_DARK_CLR,
        &mut ([
            (x,             block_h + y - marker_px),
            (x,             block_h + y),
            (x + marker_px, block_h + y),
        ].iter().copied()
         .map(|p| (p.0.floor() + 0.5, p.1.floor() - 0.5))), false);

    p.path_stroke(
        1.0,
        UI_ACCENT_DARK_CLR,
        &mut ([
            (block_w + x - marker_px, block_h + y),
            (block_w + x,             block_h + y),
            (block_w + x,             block_h + y - marker_px),
        ].iter().copied()
         .map(|p| (p.0.floor() - 0.5, p.1.floor() - 0.5))), false);
}

impl Widget for BlockCode {
    type Ret  = Entity;
    type Data = ();

    fn widget_name(&self) -> String {
        "block-code".to_string()
    }

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity.set_clip_widget(state, entity)
              .set_element(state, "block_code")
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(grid_msg) = event.message.downcast::<BlockCodeMessage>() {
            match grid_msg {
                BlockCodeMessage::SetCode(con) => {
//                    self.con = *con;
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                        .target(Entity::root()));
                },
            }
        }

        if let Some(window_event) = event.message.downcast::<WindowEvent>() {
            match window_event {
                WindowEvent::MouseDown(MouseButton::Left) => {
//                    let (x, y) = (state.mouse.cursorx, state.mouse.cursory);
//                    self.drag = true;
//                    self.drag_src_idx = self.xy2pos(state, entity, x, y);
//
//                    if let Some((inputs, _)) = self.drag_src_idx {
//                        if inputs {
//                            if self.items.0.len() == 1 {
//                                self.drag_src_idx = Some((false, 0));
//                            }
//                        } else {
//                            if self.items.1.len() == 1 {
//                                self.drag_src_idx = Some((true, 0));
//                            }
//                        }
//                    }
//
                    state.capture(entity);
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                            .target(Entity::root()));
                },
                WindowEvent::MouseUp(MouseButton::Left) => {
//                    if let Some((_drag, con)) = self.get_current_con() {
//                        self.con = Some(con);
//
//                        if let Some(callback) = self.on_change.take() {
//                            (callback)(self, state, entity, con);
//                            self.on_change = Some(callback);
//                        }
//                    } else {
//                        self.con = None;
//                    }
//
//                    self.drag = false;
//                    self.drag_src_idx = None;
//
                    state.release(entity);
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                            .target(Entity::root()));
                },
                WindowEvent::MouseMove(x, y) => {
//                    let old_hover = self.hover_idx;
//                    self.hover_idx = self.xy2pos(state, entity, *x, *y);
//
//                    if old_hover != self.hover_idx {
//                        if let Some((inputs, idx)) = self.hover_idx {
//                            if let Some(callback) = self.on_hover.take() {
//                                (callback)(self, state, entity, inputs, idx);
//                                self.on_hover = Some(callback);
//                            }
//                        }
//
//                        state.insert_event(
//                            Event::new(WindowEvent::Redraw)
//                                .target(Entity::root()));
//                    }
                },
                _ => {},
            }
        }
    }

    fn on_draw(&mut self, state: &mut State, entity: Entity, canvas: &mut Canvas) {
        let bounds = state.data.get_bounds(entity);
        if self.font.is_none() {
            self.font      = Some(canvas.add_font_mem(std::include_bytes!("font.ttf")).expect("can load font"));
            self.font_mono = Some(canvas.add_font_mem(std::include_bytes!("font_mono.ttf")).expect("can load font"));
        }

        let p = &mut FemtovgPainter {
            canvas:     canvas,
            font:       self.font.unwrap(),
            font_mono:  self.font_mono.unwrap(),
        };

        let pos : Rect = bounds.into();
        let pos = pos.floor();

        let block_h = self.block_size;
        let block_w = block_h * 2.5;

        let cols = (pos.w / block_w).ceil() as usize;
        let rows = (pos.h / block_h).ceil() as usize;

        let code = self.code.borrow();

        p.rect_fill(UI_ACCENT_BG1_CLR, pos.x, pos.y, pos.w, pos.h);

        for row in 0..rows {
            for col in 0..cols {
                let x = col as f32 * block_w;
                let y = row as f32 * block_h;

                let marker_px = (block_h * 0.2).floor();
                draw_markers(
                    p, pos.x + x, pos.y + y,
                    block_w, block_h, marker_px);
            }
        }

        for row in 0..rows {
            for col in 0..cols {
                let x = col as f32 * block_w;
                let y = row as f32 * block_h;

                if let Some(block) = code.block_at(col, row) {
                    let w = block_w;
                    let h = block.rows as f32 * block_h;

                    p.rect_fill(UI_ACCENT_CLR, pos.x + x, pos.y + y, w, h);
                    p.rect_stroke(
                        2.0, UI_PRIM_CLR,
                        pos.x + x + 1.0,
                        pos.y + y + 1.0,
                        w - 2.0, h - 2.0);

                    let hole_px = (0.6 * block_h).ceil();

                    p.label(
                        self.block_size * 0.4,
                        0,
                        UI_PRIM_CLR,
                        pos.x + x,
                        pos.y + y,
                        w,
                        h,
                        &block.lbl);

//                                    let fs =
//                                        calc_font_size_from_text(
//                                            p, name_lbl, fs, maxwidth);
                    for (i, lbl) in block.inputs.iter().enumerate() {
                        if let Some(lbl) = lbl {
                            let row = i as f32 * block_h;
                            p.rect_fill(
                                UI_ACCENT_CLR,
                                pos.x + x,
                                pos.y + y + row
                                + ((block_h - hole_px) * 0.5).floor(),
                                3.0,
                                hole_px);

                            p.label(
                                self.block_size * 0.3,
                                -1,
                                UI_PRIM_CLR,
                                pos.x + x,
                                pos.y + row + y - 1.0,
                                (block_w * 0.5).floor(),
                                block_h,
                                lbl);
                        }
                    }

                    for (i, lbl) in block.outputs.iter().enumerate() {
                        if let Some(lbl) = lbl {
                            let row = i as f32 * block_h;
                            p.rect_fill(
                                UI_ACCENT_CLR,
                                pos.x + x + w - 3.0,
                                pos.y + y + row
                                + ((block_h - hole_px) * 0.5).floor(),
                                3.0,
                                hole_px);

                            p.label(
                                self.block_size * 0.3,
                                1,
                                UI_PRIM_CLR,
                                (pos.x + x + (block_w * 0.5)).floor(),
                                pos.y + row + y - 1.0,
                                (block_w * 0.5).floor(),
                                block_h,
                                lbl);
                        }
                    }
                }
            }
        }
    }
}
