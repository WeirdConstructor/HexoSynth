use hexotk::{UIPos, AtomId, wbox};
use hexotk::{Rect, WidgetUI, Painter, WidgetData, WidgetType, UIEvent};
use hexotk::constants::*;
use hexotk::widgets::{
    Container, ContainerData,
};

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

use crate::matrix::{Matrix};

use crate::dsp::{NodeId, NodeInfo};

pub struct GenericNodeUI {
    dsp_node_id:    NodeId,
    model_node_id:  u32,
    info:           Option<NodeInfo>,
    cont:           Option<Box<WidgetData>>,
}

impl GenericNodeUI {
    pub fn new() -> Self {
        Self {
            dsp_node_id:    NodeId::Nop,
            model_node_id:  0,
            info:           None,
            cont:           None,
        }
    }

    pub fn set_target(&mut self, dsp_node_id: NodeId, model_node_id: u32) {
        self.dsp_node_id   = dsp_node_id;
        self.model_node_id = model_node_id;
        self.info          = Some(NodeInfo::from_node_id(dsp_node_id));

        self.rebuild();
    }

    pub fn rebuild(&mut self) {
        let wt_cont = Rc::new(Container::new());

        let cd = ContainerData::new();

        // TODO:
        // - detect all the inputs
        // - detect which are atoms (or all atoms)
        // - enumerate all for the AtomId below.
        // - figure out their value ranges for the knobs.
        //   => Maybe define an extra data type for this?
        //      struct InputParamDesc { min, max, type }?
        //      enum UIParamDesc { Knob { min: f32, max: f32 }, ... }
        // - implement a transformation of UIParamDesc to WidgetData.

        self.cont =
            Some(Box::new(wbox!(
                wt_cont,
                AtomId::new(self.model_node_id, 1),
                center(12, 12), cd)));
    }
}

pub struct NodePanelData {
    #[allow(dead_code)]
    matrix: Arc<Mutex<Matrix>>,

    node_ui: Rc<RefCell<GenericNodeUI>>,

    knobs:  Box<WidgetData>,
}

impl NodePanelData {
    pub fn new(node_id: u32, matrix: Arc<Mutex<Matrix>>) -> Box<dyn std::any::Any> {
        let wt_cont = Rc::new(Container::new());
        let knobs =
            Box::new(wbox!(
                wt_cont, AtomId::new(node_id, 1),
                center(12, 12), ContainerData::new()));

        Box::new(Self {
            matrix,
            knobs,
            node_ui: Rc::new(RefCell::new(GenericNodeUI::new())),
        })
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

            let pos = pos.shrink(10.0, 10.0);
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
