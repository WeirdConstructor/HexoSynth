use crate::hexo_consts::*;
use crate::rect::*;
use crate::painter::FemtovgPainter;

use tuix::*;
use femtovg::FontId;

use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub enum CvArrayMessage {
    SetArray(Arc<Mutex<Vec<f32>>>),
}

impl PartialEq for CvArrayMessage {
    fn eq(&self, other: &CvArrayMessage) -> bool {
        match self {
            CvArrayMessage::SetArray(_) =>
                if let CvArrayMessage::SetArray(_) = other { true }
                else { false },
        }
    }
}

pub struct CvArray {
    cv_model:       Arc<Mutex<Vec<f32>>>,
    font:           Option<FontId>,
    font_mono:      Option<FontId>,
    key_areas:      Vec<(usize, Rect)>,
    hover_index:    Option<usize>,

    on_change:      Option<Box<dyn Fn(&mut Self, &mut State, Entity, Arc<Mutex<Vec<f32>>>)>>,
}

impl CvArray {
    pub fn new() -> Self {
        Self {
            cv_model:       Arc::new(Mutex::new(vec![0.0; 16])),
            font:           None,
            font_mono:      None,
            key_areas:      vec![],
            hover_index:    None,
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

impl CvArray {
    fn get_key_index_at(&self, x: f32, y: f32) -> Option<usize> {
        let mut ret = None;

        for (idx, area) in &self.key_areas {
            if area.is_inside(x, y) {
                ret = Some(*idx);
            }
        }

        ret
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
                WindowEvent::MouseUp(btn) => {
                    let (x, y) = (state.mouse.cursorx, state.mouse.cursory);

                    if let Some(key_idx) = self.get_key_index_at(x, y) {
//                        self.toggle_index(key_idx);

                        if let Some(callback) = self.on_change.take() {
                            let cv_ref = self.cv_model.clone();
                            (callback)(self, state, entity, cv_ref);
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

    }
}
