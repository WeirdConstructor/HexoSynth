use crate::hexo_consts::*;
use crate::rect::*;
use crate::painter::FemtovgPainter;

use tuix::*;
use femtovg::FontId;

use std::sync::{Arc, Mutex};

pub const UI_CON_BORDER_CLR      : (f32, f32, f32) = UI_ACCENT_CLR;
pub const UI_CON_BORDER_HOVER_CLR: (f32, f32, f32) = UI_HLIGHT_CLR;
pub const UI_CON_LINE_CLR        : (f32, f32, f32) = UI_PRIM_CLR;
pub const UI_CON_HOV_CLR         : (f32, f32, f32) = UI_ACCENT_CLR;
pub const UI_CON_PHASE_CLR       : (f32, f32, f32) = UI_ACCENT_DARK_CLR;
pub const UI_CON_PHASE_BG_CLR    : (f32, f32, f32) = UI_HLIGHT2_CLR;
pub const UI_CON_BG              : (f32, f32, f32) = UI_LBL_BG_CLR;

#[derive(Clone)]
pub enum ConMessage {
    SetConnection(usize, usize),
    SetItems(Box<(Vec<String>, Vec<String>)>),
}

pub struct Connector {
    font:           Option<FontId>,
    font_mono:      Option<FontId>,
    items:          Box<(Vec<String>, Vec<String>)>,
    con:            Option<(usize, usize)>,

    active_areas:   Vec<Rect>,

    x_delta:        f32,
    hover_idx:      Option<usize>,
    drag:           bool,

    on_change:      Option<Box<dyn Fn(&mut Self, &mut State, Entity, (usize, usize))>>,
}

impl Connector {
    pub fn new() -> Self {
        Self {
            font:           None,
            font_mono:      None,
            items:          Box::new((vec![], vec![])),
            con:            None,

            active_areas:   vec![],

            x_delta:        0.0,
            hover_idx:      None,
            drag:           false,

            on_change:      None,
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
                    self.con = Some((a, b));
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                        .target(Entity::root()));
                },
                ConMessage::SetItems(items) => {
                    self.items = items;
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
//                HLStyle::Hover(_) => UI_CON_BORDER_HOVER_CLR,
//                _                 => UI_CON_BORDER_CLR,
//            };
//        let pos =
//            rect_border(p, UI_CON_BORDER, border_color, UI_CON_BG, pos);

        p.rect_fill(UI_CON_BG, pos.x, pos.y, pos.w, pos.h);

//        p.path_stroke(
//            1.0,
////                UI_CON_LINE_CLR,
//            UI_ACCENT_CLR,
//            &mut [
//                (pos.x,         0.5),
//                (pos.x + pos.w, 0.5),
//            ].iter().copied(),
//            false);
    }
}
