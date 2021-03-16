use hexotk::{UIPos, AtomId, wbox};
use hexotk::{Rect, WidgetUI, Painter, WidgetData, WidgetType, UIEvent};
use hexotk::constants::*;
use hexotk::widgets::{
    Container, ContainerData,
    Knob, KnobData,
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

    wt_knob_01:    Rc<Knob>,
    wt_knob_11:    Rc<Knob>,
}

impl GenericNodeUI {
    pub fn new() -> Self {
        let wt_knob_01 =
            Rc::new(Knob::new(30.0, 12.0, 9.0));
        let wt_knob_11 =
            Rc::new(Knob::new(30.0, 12.0, 9.0).range_signed());

        Self {
            dsp_node_id:    NodeId::Nop,
            model_node_id:  0,
            info:           None,
            cont:           None,
            wt_knob_01,
            wt_knob_11,
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

        let mut cd = ContainerData::new();

        let param_id =
            // FIXME: We should skip these params or just not enumerate there!
            self.dsp_node_id.inp_param_by_idx(0).unwrap();

        let param_name = param_id.name();

        let knob_type =
            if let Some((min, max)) = param_id.param_min_max() {
                if min < 0.0 {
                    self.wt_knob_11.clone()
                } else {
                    self.wt_knob_01.clone()
                }
            } else {
                // FIXME: Widget type should be determined by the Atom enum!
                self.wt_knob_01.clone()
            };

        cd.contrast_border()
          .new_row()
          .add(wbox!(
            knob_type,
            AtomId::new(self.model_node_id, 0),
            center(12, 12),
            KnobData::new(param_name)));

        // TODO:
        // - detect all the inputs
        // - detect which are atoms (or all atoms)
        // - enumerate all for the AtomId below.
        // - figure out their value ranges for the knobs.
        //   => Maybe define an extra data type for this?
        //      struct InputParamDesc { min, max, type }?
        //      enum UIParamDesc { Knob { min: f32, max: f32 }, ... }
        // - implement a transformation of UIParamDesc to WidgetData.
        // - Implement a Settings input
        // - Implement a sample input with string input.
        // => Then dive off into preset serialization?!
        // => Then dive off into sampler implementation
        //    - With autom. Test!?

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

//    knobs:  Box<WidgetData>,
}

impl NodePanelData {
    pub fn new(node_id: u32, matrix: Arc<Mutex<Matrix>>) -> Box<dyn std::any::Any> {
        let wt_cont = Rc::new(Container::new());

        let node_ui = Rc::new(RefCell::new(GenericNodeUI::new()));
        node_ui.borrow_mut().set_target(NodeId::Sin(0), 1);
        Box::new(Self {
            matrix,
            node_ui,
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

            let mut node_ui = data.node_ui.borrow_mut();
            if let Some(cont) = &mut node_ui.cont {
                cont.draw(ui, p, pos);
            }
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, avail: (f64, f64)) -> (f64, f64) {
        avail
    }

    fn event(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _ev: &UIEvent) {
    }
}
