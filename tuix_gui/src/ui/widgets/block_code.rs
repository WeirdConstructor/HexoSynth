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
    rows:     usize,
    contains: (Option<usize>, Option<usize>),
    expanded: bool,
    lbl:      String,
    inputs:   Vec<Option<String>>,
    outputs:  Vec<Option<String>>,
}

pub trait BlockCodeModel {
    fn area_size(&self, id: usize) -> (usize, usize);
    fn area_label(&self, id: usize) -> &str;
    fn block_at(&self, id: usize, x: usize, y: usize) -> Option<&VisBlock>;
    fn get_block_origin_at(&self, id: usize, x: usize, y: usize) -> Option<(usize, usize)>;
}

pub struct DummyBlockCode {
    blocks: HashMap<(usize, usize, usize), VisBlock>,
}

impl DummyBlockCode {
    pub fn new() -> Self {
        let mut s = Self {
            blocks: HashMap::new(),
        };

        s.blocks.insert((0, 1, 2), VisBlock {
            rows: 3,
            contains: (None, None),
            expanded: false,
            lbl: "get: x".to_string(),
            inputs:  vec![
                None,
                Some("a".to_string()),
                Some("i".to_string())],
            outputs: vec![None, None, Some(">".to_string())],
        });
        s.blocks.insert((0, 2, 4), VisBlock {
            rows: 1,
            contains: (None, None),
            expanded: false,
            lbl: "sin".to_string(),
            inputs:  vec![Some("".to_string())],
            outputs: vec![Some(">".to_string())],
        });
        s.blocks.insert((0, 2, 3), VisBlock {
            rows: 1,
            contains: (None, None),
            expanded: false,
            lbl: "1.2".to_string(),
            inputs:  vec![None],
            outputs: vec![Some(">".to_string())],
        });
        s.blocks.insert((0, 3, 3), VisBlock {
            rows: 2,
            contains: (None, None),
            expanded: false,
            lbl: "+".to_string(),
            inputs:  vec![
                Some("".to_string()),
                Some("".to_string())],
            outputs: vec![None, Some(">".to_string())],
        });

        s.blocks.insert((0, 4, 4), VisBlock {
            rows: 1,
            contains: (Some(1), Some(2)),
            expanded: true,
            lbl: "if".to_string(),
            inputs:  vec![Some("c".to_string())],
            outputs: vec![Some(">".to_string())],
        });

        s.blocks.insert((1, 0, 0), VisBlock {
            rows: 1,
            contains: (None, None),
            expanded: true,
            lbl: "1.0".to_string(),
            inputs:  vec![None],
            outputs: vec![Some(">".to_string())],
        });

        s.blocks.insert((2, 0, 0), VisBlock {
            rows: 1,
            contains: (None, None),
            expanded: true,
            lbl: "2.0".to_string(),
            inputs:  vec![None],
            outputs: vec![Some(">".to_string())],
        });

        s
    }
}

impl BlockCodeModel for DummyBlockCode {
    fn area_label(&self, id: usize) -> &str {
        match id {
            0 => "Main",
            1 => "then",
            2 => "else",
            _ => "?",
        }
    }

    fn area_size(&self, id: usize) -> (usize, usize) {
        match id {
            1 => (1, 1),
            2 => (1, 1),
            _ => (16, 16),
        }
    }

    fn block_at(&self, id: usize, x: usize, y: usize) -> Option<&VisBlock> {
        self.blocks.get(&(id, x, y))
    }

    fn get_block_origin_at(&self, id: usize, x: usize, y: usize)
        -> Option<(usize, usize)>
    {
        for ((area_id, bx, by), b) in self.blocks.iter() {
            if *area_id == id {
                if x == *bx && y >= *by && y <= *by + (b.rows - 1) {
                    return Some((*bx, *by));
                }
            }
        }

        None
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

    areas:          Vec<Vec<(usize, Rect)>>,
    hover:          (usize, usize, usize),

    on_change:      Option<Box<dyn Fn(&mut Self, &mut State, Entity, (usize, usize))>>,
    on_expand:      Option<Box<dyn Fn(&mut Self, &mut State, Entity, usize)>>,
    on_req_val:     Option<Box<dyn Fn(&mut Self, &mut State, Entity, (usize, usize, usize), bool)>>,
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

            areas:          vec![],
            hover:          (0, 0, 0),

            on_change:      None,
            on_expand:      None,
            on_req_val:     None,
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

    pub fn on_expand<F>(mut self, on_expand: F) -> Self
    where
        F: 'static + Fn(&mut Self, &mut State, Entity, usize),
    {
        self.on_expand = Some(Box::new(on_expand));

        self
    }

    pub fn on_req_val<F>(mut self, on_req_val: F) -> Self
    where
        F: 'static + Fn(&mut Self, &mut State, Entity, (usize, usize, usize), bool),
    {
        self.on_req_val = Some(Box::new(on_req_val));

        self
    }

    pub fn on_hover<F>(mut self, on_hover: F) -> Self
    where
        F: 'static + Fn(&mut Self, &mut State, Entity, bool, usize),
    {
        self.on_hover = Some(Box::new(on_hover));

        self
    }

    pub fn reset_areas(&mut self) {
        for a in self.areas.iter_mut() {
            a.clear();
        }
    }

    pub fn store_area_pos(&mut self, area_id: usize, level: usize, pos: Rect) {
        if level >= self.areas.len() {
            self.areas.resize_with(area_id + 1, || vec![]);
        }

        self.areas[level].push((area_id, pos));
    }

    pub fn draw_area(&mut self, p: &mut FemtovgPainter, area_id: usize, pos: Rect, level: usize) {
        p.clip_region(pos.x, pos.y, pos.w, pos.h);

        let block_h = self.block_size;
        let block_w = block_h * 2.0;

        let cols = (pos.w / block_w).ceil() as usize;
        let rows = (pos.h / block_h).ceil() as usize;

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

//        p.label(
//            self.block_size * 0.7,
//            -1,
//            UI_PRIM2_CLR,
//            (pos.x + block_h * 0.2).floor(),
//            pos.y,
//            block_w * 3.0,
//            block_h,
//            self.code.borrow().area_label(0));

        let mut next_areas = vec![];

        for row in 0..rows {
            for col in 0..cols {
                let x = col as f32 * block_w;
                let y = row as f32 * block_h;

                let mut hover_here =
                      self.hover.0 == area_id
                   && col == self.hover.1
                   && row == self.hover.2;

                if self.hover.0 == area_id {
                    if let Some((bx, by)) =
                        self.code.borrow().get_block_origin_at(
                            self.hover.0,
                            self.hover.1,
                            self.hover.2)
                    {
                        hover_here = bx == col && by == row;
                    }
                }

                let bg_color =
                    if hover_here { UI_ACCENT_CLR }
                    else { UI_ACCENT_BG2_CLR };
                let border_color =
                    if hover_here { UI_HLIGHT_CLR }
                    else { UI_PRIM_CLR };

                if let Some(block) = self.code.borrow().block_at(area_id, col, row) {
                    let w = block_w;
                    let h = block.rows as f32 * block_h;

                    p.rect_fill(bg_color, pos.x + x, pos.y + y, w, h);

                    p.rect_stroke(
                        2.0, border_color,
                        pos.x + x + 1.0,
                        pos.y + y + 1.0,
                        w - 2.0, h - 2.0);

                    let hole_px = (0.6 * block_h).ceil();

                    p.label(
                        self.block_size * 0.5,
                        0, UI_PRIM_CLR,
                        pos.x + x, pos.y + y, w, h, &block.lbl);

//                                    let fs =
//                                        calc_font_size_from_text(
//                                            p, name_lbl, fs, maxwidth);

                    for (i, lbl) in block.inputs.iter().enumerate() {
                        if let Some(lbl) = lbl {
                            let row = i as f32 * block_h;
                            p.rect_fill(
                                bg_color,
                                pos.x + x,
                                pos.y + y + row
                                + ((block_h - hole_px) * 0.5).floor(),
                                3.0,
                                hole_px);

                            p.label(
                                self.block_size * 0.4,
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
                                bg_color,
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

                    if let Some(cont_id) = block.contains.0 {
                        let (area_w, area_h) =
                            self.code.borrow().area_size(cont_id);
                        let bpos = Rect {
                            x: pos.x + x, // + border
                            y: pos.y + y + h,
                            w: (area_w as f32 * block_w + block_w * 0.3).floor(),
                            h: (area_h as f32 * block_h + block_w * 0.3).floor(),
                        };

                        next_areas.push((cont_id, bpos, border_color, bg_color));

                        if let Some(cont_id) = block.contains.1 {
                            let (area_w, area_h) =
                                self.code.borrow().area_size(cont_id);
                            let bpos = Rect {
                                x: bpos.x,
                                y: bpos.y + bpos.h,
                                w: (area_w as f32 * block_w + block_w * 0.3).floor(),
                                h: (area_h as f32 * block_h + block_w * 0.3).floor(),
                            };

                            next_areas.push((cont_id, bpos, border_color, bg_color));
                        }
                    }

                } else if hover_here {
                    p.rect_stroke(
                        2.0, UI_HLIGHT_CLR,
                        pos.x + x + 1.0,
                        pos.y + y + 1.0,
                        block_w - 2.0, block_h - 2.0);
                }
            }
        }

        for cont_area in next_areas.into_iter() {
            let (cont_id, pos, border_color, bg_color) = cont_area;

            let (area_w, area_h) = self.code.borrow().area_size(cont_id);
            let apos = Rect {
                x: pos.x + 4.0, // + border
                y: pos.y + 4.0,
                w: pos.w,
                h: pos.h,
            };
            p.rect_fill(
                border_color,
                apos.x - 4.0, apos.y - 4.0, apos.w + 8.0, apos.h + 8.0);
            p.rect_fill(
                UI_ACCENT_BG1_CLR,
                apos.x - 2.0, apos.y - 2.0, apos.w + 4.0, apos.h + 4.0);
            p.rect_fill(
                bg_color,
                (pos.x + block_w * 0.25).floor(),
                pos.y - 2.0,
                (block_w * 0.5).floor(),
                8.0);

            self.store_area_pos(cont_id, level + 1, Rect {
                x: apos.x - 4.0,
                y: apos.y - 4.0,
                w: apos.w + 8.0,
                h: apos.h + 8.0,
            });
            self.draw_area(p, cont_id, apos, level + 1);
        }

        p.reset_clip_region();
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
                    let old_hover = self.hover;
                    let mut found = false;

                    let block_h = self.block_size;
                    let block_w = block_h * 2.0;

                    for lvl in self.areas.iter().rev() {
                        for a in lvl.iter() {
                            let (id, pos) = *a;

                            if pos.is_inside(*x, *y) {
                                let xo = *x - pos.x;
                                let yo = *y - pos.y;
                                let xi = (xo / block_w).floor() as usize;
                                let yi = (yo / block_h).floor() as usize;

                                self.hover = (a.0, xi, yi);
                                found = true;
                                break;
                            }
                        }
                        if found { break; }
                    }

                    if old_hover != self.hover {
                        state.insert_event(
                            Event::new(WindowEvent::Redraw)
                                .target(Entity::root()));
                    }
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

        self.reset_areas();

        self.store_area_pos(0, 0, pos);
        self.draw_area(p, 0, pos, 0);
    }
}
