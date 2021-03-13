use hexotk::{UIPos, AtomId, wbox};
use hexotk::{Rect, WidgetUI, Painter, WidgetData, WidgetType, UIEvent};
use hexotk::constants::*;
use hexotk::widgets::{
    Container, ContainerData,
};

use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::matrix::{Matrix};

pub struct NodePanelData {
    #[allow(dead_code)]
    matrix: Arc<Mutex<Matrix>>,

    knobs:  Box<WidgetData>,
}

impl NodePanelData {
    pub fn new(node_id: u32, matrix: Arc<Mutex<Matrix>>) -> Box<dyn std::any::Any> {
        let wt_cont = Rc::new(Container::new());
        let knobs =
            Box::new(wbox!(
                wt_cont, AtomId::new(node_id, 1),
                center(12, 12), ContainerData::new()));

        Box::new(Self { matrix, knobs })
    }
}


#[derive(Debug)]
pub struct NodePanel { }

impl NodePanel {
    pub fn new() -> Self { NodePanel { } }
}

impl WidgetType for NodePanel {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let pos = pos.shrink(UI_PADDING, UI_PADDING);

        data.with(|data: &mut NodePanelData| {
            p.rect_fill(UI_BG_CLR, pos.x, pos.y, pos.w, pos.h);

            let pos = pos.shrink(100.0, 100.0);
            p.rect_fill(UI_PRIM_CLR, pos.x, pos.y, pos.w, pos.h);

            let knob_pos = pos.crop_top(200.0);

            data.knobs.draw(ui, p, knob_pos);
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, avail: (f64, f64)) -> (f64, f64) {
        avail
    }

    fn event(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _ev: &UIEvent) {
    }
}
