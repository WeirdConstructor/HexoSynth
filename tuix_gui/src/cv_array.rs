use crate::hexo_consts::*;
use crate::rect::*;
use crate::painter::FemtovgPainter;

use tuix::*;
use femtovg::FontId;

use std::sync::{Arc, Mutex};

pub const UI_GRPH_LINE_CLR        : (f32, f32, f32) = UI_PRIM_CLR;
pub const UI_GRPH_HOV_CLR         : (f32, f32, f32) = UI_ACCENT_CLR;
pub const UI_GRPH_PHASE_CLR       : (f32, f32, f32) = UI_ACCENT_DARK_CLR;
pub const UI_GRPH_PHASE_BG_CLR    : (f32, f32, f32) = UI_HLIGHT2_CLR;
pub const UI_GRPH_BG              : (f32, f32, f32) = UI_LBL_BG_CLR;

#[derive(Clone)]
pub enum CvArrayMessage {
    SetArray(Arc<Mutex<Vec<f32>>>),
}

pub struct CvArray {
    cv_model:       Arc<Mutex<Vec<f32>>>,
    font:           Option<FontId>,
    font_mono:      Option<FontId>,
    x_delta:        f32,
    binary:         bool,
    hover_idx:      Option<usize>,
    drag:           bool,

    on_change:      Option<Box<dyn Fn(&mut Self, &mut State, Entity, Arc<Mutex<Vec<f32>>>)>>,
}

impl CvArray {
    pub fn new(binary: bool) -> Self {
        Self {
            cv_model:       Arc::new(Mutex::new(vec![0.0; 16])),
            font:           None,
            font_mono:      None,
            on_change:      None,
            x_delta:        0.0,
            hover_idx:      None,
            drag:           false,
            binary,
        }
    }

    pub fn on_change<F>(mut self, on_change: F) -> Self
    where
        F: 'static + Fn(&mut Self, &mut State, Entity, Arc<Mutex<Vec<f32>>>),
    {
        self.on_change = Some(Box::new(on_change));

        self
    }

}

impl CvArray {
    fn set(&mut self, state: &mut State, entity: Entity, idx: usize, v: f32) {
        let mut changed = false;

        if let Ok(mut cv) = self.cv_model.lock() {
            if idx < cv.len() {
                if self.binary {
                    cv[idx] = if v > 0.5 { 1.0 } else { 0.0 };
                } else {
                    cv[idx] = v;
                }
                changed = true;
            }
        }

        if changed {
            if let Some(callback) = self.on_change.take() {
                (callback)(self, state, entity, self.cv_model.clone());
                self.on_change = Some(callback);
            }
        }
    }

    fn pos2value(&self, state: &mut State, entity: Entity, _x: f32, y: f32) -> f32 {
        let bounds = state.data.get_bounds(entity);
        let pos : Rect = bounds.into();
        let pos = pos.floor();

        let yo = y - pos.y;
        let v = yo / pos.h;
        1.0 - v.clamp(0.0, 1.0)
    }

    fn pos2idx(&self, state: &mut State, entity: Entity, x: f32, _y: f32) -> Option<usize> {
        let bounds = state.data.get_bounds(entity);
        let pos : Rect = bounds.into();
        let pos = pos.floor();

        if x >= pos.x && x <= (pos.x + pos.w) {
            let xo = x - pos.x;
            let i = (xo / self.x_delta).floor() as usize;
            Some(i)
        } else {
            None
        }
    }
}

impl Widget for CvArray {
    type Ret  = Entity;
    type Data = ();

    fn widget_name(&self) -> String {
        "cv-array".to_string()
    }

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity.set_position_type(state, PositionType::ParentDirected)
              .set_clip_widget(state, entity)
              .set_element(state, "cv-array")
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(grid_msg) = event.message.downcast::<CvArrayMessage>() {
            match grid_msg {
                CvArrayMessage::SetArray(array) => {
                    self.cv_model = array.clone();
                    state.insert_event(
                        Event::new(WindowEvent::Redraw).target(Entity::root()));
                },
            }
        }

        if let Some(window_event) = event.message.downcast::<WindowEvent>() {
            match window_event {
                WindowEvent::MouseDown(MouseButton::Left) => {
                    let (x, y) = (state.mouse.cursorx, state.mouse.cursory);
                    self.drag = true;

                    if let Some(idx) = self.pos2idx(state, entity, x, y) {
                        let v = self.pos2value(state, entity, x, y);
                        self.set(state, entity, idx, v);
                    }

                    state.capture(entity);
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                            .target(Entity::root()));
                },
                WindowEvent::MouseUp(MouseButton::Left) => {
                    let (x, y) = (state.mouse.cursorx, state.mouse.cursory);
                    self.drag = false;

                    if let Some(idx) = self.pos2idx(state, entity, x, y) {
                        let v = self.pos2value(state, entity, x, y);
                        self.set(state, entity, idx, v);
                    }

                    state.release(entity);
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                            .target(Entity::root()));
                },
                WindowEvent::MouseMove(x, y) => {
                    let old_hover = self.hover_idx;
                    self.hover_idx = self.pos2idx(state, entity, *x, *y);

                    if let Some(idx) = self.hover_idx {
                        if self.drag {
                            let v = self.pos2value(state, entity, *x, *y);
                            self.set(state, entity, idx, v);
                            state.insert_event(
                                Event::new(WindowEvent::Redraw)
                                    .target(Entity::root()));
                        }
                    }

                    if old_hover != self.hover_idx {
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

//        let highlight = ui.hl_style_for(id, None);
//        let border_color =
//            match highlight {
//                HLStyle::Hover(_) => UI_GRPH_BORDER_HOVER_CLR,
//                _                 => UI_GRPH_BORDER_CLR,
//            };
//        let pos =
//            rect_border(p, UI_GRPH_BORDER, border_color, UI_GRPH_BG, pos);

        p.rect_fill(UI_GRPH_BG, pos.x, pos.y, pos.w, pos.h);

        let data =
            if let Ok(data) = self.cv_model.lock() {
                data
            } else {
                return;
            };

        let xd = pos.w / (data.len() as f32);
        let xd = xd.floor();

        self.x_delta = xd.max(1.0);

        let phase = 0.0;
// TODO:
//        let phase =
//            if let Some(phase) = ui.atoms().get_phase_value(id) {
//                phase as f32
//            } else { 0.0 };

        let mut x = 0.0;
        let phase_delta = 1.0 / (data.len() as f32);
        let mut xphase = 0.0;

        for i in 0..data.len() {
            let v = {
                if i < data.len() { data[i] }
                else { 0.0 }
            };

            let hover_highlight =
                if let Some(hov_i) = self.hover_idx { hov_i == i }
                else { false };
            let hover_highlight =
                hover_highlight
                && (state.hovered == entity || self.drag);

            let h = pos.h * (1.0 - v);

            let (mut color, mut phase_bg_color) =
                if phase >= xphase && phase < (xphase + phase_delta) {
                    (UI_GRPH_PHASE_CLR, Some(UI_GRPH_PHASE_BG_CLR))
                } else {
                    (UI_GRPH_LINE_CLR, None)
                };

            if hover_highlight {
                color = UI_GRPH_HOV_CLR;
                phase_bg_color = Some(UI_GRPH_PHASE_BG_CLR);
            }

            xphase += phase_delta;

            //d// println!("h={:6.2} pos.h={:6.2}", h, pos.h);

            // draw the last a little bit wider to prevent the gap
            let w =
                if i == (data.len() - 1) {
                    xd + 0.5
                } else {
                    xd
                };

            if let Some(bg_color) = phase_bg_color {
                p.rect_fill(
                    bg_color,
                    (pos.x + x).ceil() - 0.5,
                    pos.y - 0.5,
                    w,
                    pos.h);
            }

            if pos.h - h > 0.5 {
                p.rect_fill(
                    color,
                    (pos.x + x).ceil() - 0.5,
                    (pos.y + h) - 0.5,
                    w,
                    pos.h - h + 1.5);
            }

            x += xd;
        }

        p.path_stroke(
            1.0,
//                UI_GRPH_LINE_CLR,
            UI_ACCENT_CLR,
            &mut [
                (pos.x,         0.5),
                (pos.x + pos.w, 0.5),
            ].iter().copied(),
            false);

        let mut x = xd;
        for _i in 0..(data.len() - 1) {
            p.path_stroke(
                1.0,
                UI_GRPH_LINE_CLR,
                &mut [
                    ((pos.x + x).floor() - 0.5, pos.y.floor()),
                    ((pos.x + x).floor() - 0.5, pos.y + pos.h),
                ].iter().copied(),
                false);

            x += xd;
        }
    }
}
