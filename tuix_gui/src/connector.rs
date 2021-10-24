use crate::hexo_consts::*;
use crate::rect::*;
use crate::painter::{FemtovgPainter, calc_font_size_from_text};

use tuix::*;
use femtovg::FontId;

use std::sync::{Arc, Mutex};

pub const UI_CON_BORDER_CLR      : (f32, f32, f32) = UI_ACCENT_CLR;
pub const UI_CON_BORDER_HOVER_CLR: (f32, f32, f32) = UI_HLIGHT_CLR;
pub const UI_CON_HOV_CLR         : (f32, f32, f32) = UI_HLIGHT_CLR;
pub const UI_CON_PHASE_CLR       : (f32, f32, f32) = UI_ACCENT_DARK_CLR;
pub const UI_CON_PHASE_BG_CLR    : (f32, f32, f32) = UI_HLIGHT2_CLR;
pub const UI_CON_BG              : (f32, f32, f32) = UI_LBL_BG_CLR;
pub const UI_CON_BORDER_W        : f32             = 2.0;

#[derive(Clone)]
pub enum ConMessage {
    SetConnection(usize, usize),
    SetItems(Box<(Vec<(String, bool)>, Vec<(String, bool)>)>),
}

pub struct Connector {
    font_size:      f32,
    font:           Option<FontId>,
    font_mono:      Option<FontId>,
    items:          Box<(Vec<(String, bool)>, Vec<(String, bool)>)>,
    con:            Option<(usize, usize)>,

    active_areas:   Vec<Rect>,

    xcol:           f32,
    yrow:           f32,
    hover_idx:      Option<(bool, usize)>,
    drag_src_idx:   Option<(bool, usize)>,
    drag:           bool,

    on_change:      Option<Box<dyn Fn(&mut Self, &mut State, Entity, (usize, usize))>>,
    on_hover:       Option<Box<dyn Fn(&mut Self, &mut State, Entity, bool, usize)>>,
}

impl Connector {
    pub fn new() -> Self {
        Self {
            font_size:      14.0,
            font:           None,
            font_mono:      None,
            items:          Box::new((vec![], vec![])),
            con:            None,

            active_areas:   vec![],

            xcol:           0.0,
            yrow:           0.0,
            hover_idx:      None,
            drag_src_idx:   None,
            drag:           false,

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

    fn xy2pos(&self, state: &mut State, entity: Entity, x: f32, y: f32)
        -> Option<(bool, usize)>
    {
        let bounds = state.data.get_bounds(entity);
        let pos : Rect = bounds.into();
        let x = x - pos.x;
        let y = y - pos.y;

        let w_half = pos.w * 0.5;

        let old_hover = self.hover_idx;

        if y > 0.0 && x > 0.0 {
            let idx = (y / self.yrow).floor() as usize;
            let inputs = x > w_half;

            if inputs {
                if idx < self.items.1.len() {
                    Some((inputs, idx))
                } else {
                    None
                }
            } else {
                if idx < self.items.0.len() {
                    Some((inputs, idx))
                } else {
                    None
                }
            }
        } else {
            None
        }
    }

    fn get_current_con(&self) -> Option<(bool, (usize, usize))> {
        let (a_inp, a) =
            if let Some((inputs, row)) = self.drag_src_idx {
                (inputs, row)
            } else {
                return self.con.map(|con| (false, con));
            };

        let (b_inp, b) =
            if let Some((inputs, row)) = self.hover_idx {
                (inputs, row)
            } else {
                return self.con.map(|con| (false, con));
            };

        if a_inp == b_inp {
            return self.con.map(|con| (false, con));
        }

        let (a, b) =
            if b_inp { (a, b) }
            else     { (b, a) };

        if !self.items.0.get(a).map(|x| x.1).unwrap_or(false) {
            return self.con.map(|con| (false, con));
        }

        if !self.items.1.get(b).map(|x| x.1).unwrap_or(false) {
            return self.con.map(|con| (false, con));
        }

        Some((true, (a, b)))
    }
}

impl Widget for Connector {
    type Ret  = Entity;
    type Data = ();

    fn widget_name(&self) -> String {
        "connector".to_string()
    }

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity.set_clip_widget(state, entity)
              .set_element(state, "connector")
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(grid_msg) = event.message.downcast::<ConMessage>() {
            match grid_msg {
                ConMessage::SetConnection(a, b) => {
                    self.con = Some((*a, *b));
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                        .target(Entity::root()));
                },
                ConMessage::SetItems(items) => {
                    self.items = items.clone();
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                        .target(Entity::root()));
                },
            }
        }

        if let Some(window_event) = event.message.downcast::<WindowEvent>() {
            match window_event {
                WindowEvent::MouseDown(MouseButton::Left) => {
                    let (x, y) = (state.mouse.cursorx, state.mouse.cursory);
                    self.drag = true;
                    self.drag_src_idx = self.xy2pos(state, entity, x, y);

                    state.capture(entity);
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                            .target(Entity::root()));
                },
                WindowEvent::MouseUp(MouseButton::Left) => {
                    let (x, y) = (state.mouse.cursorx, state.mouse.cursory);

                    if let Some((drag, con)) = self.get_current_con() {
                        self.con = Some(con);

                        if let Some(callback) = self.on_change.take() {
                            (callback)(self, state, entity, con);
                            self.on_change = Some(callback);
                        }
                    } else {
                        self.con = None;
                    }

                    self.drag = false;
                    self.drag_src_idx = None;

                    state.release(entity);
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                            .target(Entity::root()));
                },
                WindowEvent::MouseMove(x, y) => {
                    let old_hover = self.hover_idx;
                    self.hover_idx = self.xy2pos(state, entity, *x, *y);

                    if old_hover != self.hover_idx {
                        if let Some((inputs, idx)) = self.hover_idx {
                            if let Some(callback) = self.on_hover.take() {
                                (callback)(self, state, entity, inputs, idx);
                                self.on_hover = Some(callback);
                            }
                        }

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

        let row_h = self.items.0.len().max(self.items.1.len());

        // XXX: - 1.0 on height and width for some extra margin so that the
        //      rectangles for the ports are not clipped.
        let yrow = ((pos.h - 2.0 * UI_CON_BORDER_W) / (row_h as f32)).floor();
        let xcol = ((pos.w - 2.0 * UI_CON_BORDER_W) / 3.0).floor();

        self.xcol = xcol;
        self.yrow = yrow;

        let pos = Rect {
            x: pos.x + UI_CON_BORDER_W,
            y: pos.y + UI_CON_BORDER_W,
            w: xcol * 3.0,
            h: yrow * (row_h as f32),
        };

        p.rect_fill(UI_ACCENT_BG1_CLR,
            pos.x - UI_CON_BORDER_W,
            pos.y - UI_CON_BORDER_W,
            pos.w + UI_CON_BORDER_W,
            pos.h + UI_CON_BORDER_W);

        let does_hover_this_widget =
            state.hovered == entity;

        for row in 0..row_h {
            let yo      = row as f32 * yrow;
            let txt_pad = 2.0 * UI_CON_BORDER_W;
            let txt_w   = xcol - 2.0 * txt_pad;

            if let Some((lbl, active)) = self.items.0.get(row) {
                p.rect_stroke(
                    UI_CON_BORDER_W,
                    UI_CON_BORDER_CLR,
                    pos.x, pos.y + yo, xcol, yrow);

                let fs =
                    calc_font_size_from_text(p, &lbl, self.font_size, txt_w);
                p.label(
                    fs, -1, if *active { UI_PRIM_CLR } else { UI_INACTIVE_CLR },
                    pos.x + txt_pad, pos.y + yo,
                    txt_w, yrow, &lbl);
            }

            if let Some((lbl, active)) = self.items.1.get(row) {
                p.rect_stroke(
                    UI_CON_BORDER_W,
                    UI_CON_BORDER_CLR,
                    pos.x + 2.0 * xcol - 1.0, pos.y + yo, xcol, yrow);

                let fs =
                    calc_font_size_from_text(p, &lbl, self.font_size, txt_w);
                p.label(
                    fs, 1, if *active { UI_PRIM_CLR } else { UI_INACTIVE_CLR },
                    pos.x + txt_pad + 2.0 * xcol - UI_CON_BORDER_W, pos.y + yo,
                    txt_w, yrow, &lbl);
            }
        }

        if let Some((inputs, row)) = self.hover_idx {
            let items = if inputs { &self.items.1 } else { &self.items.0 };

            if let Some((lbl, active)) = items.get(row) {
                if *active {
                    let xo = if inputs { xcol * 2.0 - 1.0 } else { 0.0 };
                    let yo = row as f32 * yrow;

                    if does_hover_this_widget {
                        p.rect_stroke(
                            UI_CON_BORDER_W,
                            UI_CON_HOV_CLR,
                            pos.x + xo, pos.y + yo, xcol, yrow);
                    }
                }
            }
        }

        if let Some((inputs, row)) = self.drag_src_idx {
            let xo = if inputs { xcol * 2.0 - 1.0 } else { 0.0 };
            let yo = row as f32 * yrow;

            if self.drag {
                p.rect_stroke(
                    UI_CON_BORDER_W,
                    UI_SELECT_CLR,
                    pos.x + xo, pos.y + yo, xcol, yrow);
            }
        }

        if let Some((drag, (a, b))) = self.get_current_con() {
            let ay = a as f32 * yrow;
            let by = b as f32 * yrow;

            p.path_stroke(
                4.0,
                if drag { UI_CON_HOV_CLR } else { UI_PRIM_CLR },
                &mut [
                    (xcol,                         ay + yrow * 0.5),
                    (xcol + xcol * 0.25,           ay + yrow * 0.5),
                    (2.0 * xcol - xcol * 0.25,     by + yrow * 0.5),
                    (2.0 * xcol - UI_CON_BORDER_W, by + yrow * 0.5),
                ].iter().copied().map(|(x, y)|
                    ((pos.x + x).floor(),
                     (pos.y + y).floor())),
                false);
        }
    }
}
