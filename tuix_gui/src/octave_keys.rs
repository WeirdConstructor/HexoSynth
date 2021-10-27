use crate::hexo_consts::*;
use crate::rect::*;
use crate::painter::FemtovgPainter;

use tuix::*;
use femtovg::FontId;

use std::rc::Rc;
use std::cell::RefCell;

pub const UI_GRPH_BORDER_CLR      : (f32, f32, f32) = UI_ACCENT_CLR;
pub const UI_GRPH_BORDER_HOVER_CLR: (f32, f32, f32) = UI_HLIGHT_CLR;
pub const UI_GRPH_LINE_CLR        : (f32, f32, f32) = UI_PRIM_CLR;
pub const UI_GRPH_PHASE_CLR       : (f32, f32, f32) = UI_HLIGHT_CLR;
pub const UI_GRPH_PHASE_BG_CLR    : (f32, f32, f32) = UI_HLIGHT2_CLR;
pub const UI_GRPH_BG              : (f32, f32, f32) = UI_LBL_BG_CLR;

#[derive(Clone)]
pub enum OctaveKeysMessage {
    SetMask(i64),
}

pub struct OctaveKeys {
    key_mask:       i64,
    font:           Option<FontId>,
    font_mono:      Option<FontId>,
    key_areas:      Vec<(usize, Rect)>,
    hover_index:    Option<usize>,

    on_change:      Option<Box<dyn Fn(&mut Self, &mut State, Entity, i64)>>,
}

impl OctaveKeys {
    pub fn new() -> Self {
        Self {
            key_mask:       0,
            font:           None,
            font_mono:      None,
            key_areas:      vec![],
            hover_index:    None,
            on_change:      None,
        }
    }

    pub fn on_change<F>(mut self, on_change: F) -> Self
    where
        F: 'static + Fn(&mut Self, &mut State, Entity, i64),
    {
        self.on_change = Some(Box::new(on_change));

        self
    }

}

impl OctaveKeys {
    fn get_key_index_at(&self, x: f32, y: f32) -> Option<usize> {
        let mut ret = None;

        for (idx, area) in &self.key_areas {
            if area.is_inside(x, y) {
                ret = Some(*idx);
            }
        }

        ret
    }

    pub fn toggle_index(&mut self, index: usize) {
        if index >= 64 { return; }
        self.key_mask ^= 0x1 << index;
    }
}

impl Widget for OctaveKeys {
    type Ret  = Entity;
    type Data = ();

    fn widget_name(&self) -> String {
        "octave-keys".to_string()
    }

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity.set_position_type(state, PositionType::ParentDirected)
              .set_clip_widget(state, entity)
              .set_element(state, "octave-keys")
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(grid_msg) = event.message.downcast::<OctaveKeysMessage>() {
            match grid_msg {
                OctaveKeysMessage::SetMask(key_mask) => {
                    self.key_mask = *key_mask;
                    state.insert_event(
                        Event::new(WindowEvent::Redraw).target(Entity::root()));
                },
            }
        }

        if let Some(window_event) = event.message.downcast::<WindowEvent>() {
            match window_event {
                WindowEvent::MouseUp(btn) => {
                    let (x, y) = (state.mouse.cursorx, state.mouse.cursory);

                    if let Some(key_idx) = self.get_key_index_at(x, y) {
                        self.toggle_index(key_idx);

                        if let Some(callback) = self.on_change.take() {
                            (callback)(self, state, entity, self.key_mask);
                            self.on_change = Some(callback);
                        }
                    }
                },
                WindowEvent::MouseMove(x, y) => {
                    let old_hover = self.hover_index;
                    self.hover_index = self.get_key_index_at(*x, *y);

                    if old_hover != self.hover_index {
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
        let pos = Rect {
            x: pos.x.floor(),
            y: pos.y.floor(),
            w: pos.w.floor(),
            h: pos.h.floor(),
        };

        let (xo, yo) = (
            (pos.x + pos.w / 2.0).round(),
            (pos.y + pos.h / 2.0).round()
        );

        let border_color =
            if state.hovered == entity { UI_GRPH_BORDER_HOVER_CLR }
            else { UI_GRPH_BORDER_CLR };

        let xd = (pos.w / 7.0).floor();
        let xd_pad_for_center = ((pos.w - xd * 7.0) * 0.5).floor();
        let pos = pos.shrink(xd_pad_for_center, 0.0);

        let xoffs_w = [
            (0, 0.0 * xd),   // white C
            (2, 1.0 * xd),   // white D
            (4, 2.0 * xd),   // white E
            (5, 3.0 * xd),   // white F
            (7, 4.0 * xd),   // white G
            (9, 5.0 * xd),   // white A
            (11, 6.0 * xd),  // white B
        ];

        let xoffs_b = [
            (1, 1.0 * xd),   // black C#
            (3, 2.0 * xd),   // black D#
            (6, 4.0 * xd),   // black F#
            (8, 5.0 * xd),   // black G#
            (10, 6.0 * xd),  // black A#
        ];

// TODO
//        let phase =
//            if let Some(phase) = ui.atoms().get_phase_value(id) {
//                phase as f64
//            } else { 0.0 };
        let phase = 0.0_f32;

        let phase_index = (phase * 12.0).floor() as usize;

        fn draw_key(p: &mut FemtovgPainter, key_mask: i64,
                    key: Rect, hover_idx: Option<usize>,
                    index: usize,
                    phase_index: usize)
        {
            let key_is_set = key_mask & (0x1 << index) > 0;

            let mut hover_this_key = false;
            if let Some(hover_idx) = hover_idx {
                hover_this_key = (hover_idx == index);
            }

            let (mut bg_color, mut line_color) =
                if key_is_set {
                    if hover_this_key {
                        (UI_GRPH_LINE_CLR, UI_GRPH_BG)
                    } else {
                        (UI_GRPH_PHASE_BG_CLR, UI_GRPH_BG)
                    }
                } else if hover_this_key {
                    (UI_GRPH_PHASE_BG_CLR, UI_GRPH_BG)
                } else {
                    (UI_GRPH_BG, UI_GRPH_LINE_CLR)
                };

            if phase_index == index {
                if key_is_set {
                    bg_color = UI_GRPH_BORDER_CLR;
                } else {
                    bg_color = UI_GRPH_PHASE_CLR;
                }

                line_color = UI_GRPH_BG;
            }

            p.rect_fill(line_color, key.x, key.y, key.w, key.h);
            let k2 = key.shrink(1.0, 1.0);
            p.rect_fill(bg_color, k2.x, k2.y, k2.w, k2.h);
        }

        let mut hover_idx = self.hover_index;
        if state.hovered != entity { hover_idx = None; }

        self.key_areas.clear();
        for xw in xoffs_w.iter() {
            let key =
                Rect {
                    x: pos.x + (*xw).1,
                    y: pos.y,
                    w: xd,
                    h: pos.h,
                };

            draw_key(p, self.key_mask, key, hover_idx, (*xw).0, phase_index);
            self.key_areas.push(((*xw).0, key));
        }

        let black_width = xd * 0.75;

        for xb in xoffs_b.iter() {
            let key =
                Rect {
                    x: pos.x + (*xb).1 - black_width * 0.5,
                    y: pos.y,
                    w: black_width,
                    h: pos.h * 0.5,
                };

            draw_key(p, self.key_mask, key, hover_idx, (*xb).0, phase_index);
            self.key_areas.push(((*xb).0, key));
        }
    }
}
