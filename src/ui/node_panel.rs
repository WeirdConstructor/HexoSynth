use hexotk::{UIPos, AtomId, wbox};
use hexotk::{Rect, WidgetUI, Painter, WidgetData, WidgetType, UIEvent};
use hexotk::constants::*;
use hexotk::widgets::{
    Container, ContainerData,
    Knob, KnobData,
    Button, ButtonData,
};

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

use crate::matrix::Matrix;

use crate::ui::matrix::MatrixEditorRef;

use crate::dsp::{NodeId, NodeInfo, SAtom};

pub struct GenericNodeUI {
    dsp_node_id:    NodeId,
    model_node_id:  u32,
    info:           Option<NodeInfo>,
    cont:           Option<Box<WidgetData>>,

    wt_knob_01:    Rc<Knob>,
    wt_knob_11:    Rc<Knob>,
    wt_btn:        Rc<Button>,
}

impl GenericNodeUI {
    pub fn new() -> Self {
        let wt_knob_01 =
            Rc::new(Knob::new(30.0, 12.0, 9.0));
        let wt_knob_11 =
            Rc::new(Knob::new(30.0, 12.0, 9.0).range_signed());
        let wt_btn = Rc::new(Button::new(50.0, 12.0));

        Self {
            dsp_node_id:    NodeId::Nop,
            model_node_id:  0,
            info:           None,
            cont:           None,
            wt_knob_01,
            wt_knob_11,
            wt_btn,
        }
    }

    pub fn set_target(&mut self, dsp_node_id: NodeId, model_node_id: u32) {
        self.dsp_node_id   = dsp_node_id;
        self.model_node_id = model_node_id;
        self.info          = Some(NodeInfo::from_node_id(dsp_node_id));

        self.rebuild();
    }

    fn build_atom_input(&self, pos: (u8, u8), idx: usize) -> Option<WidgetData> {
        let param_id = self.dsp_node_id.param_by_idx(idx)?;
        let param_name = param_id.name();

        match param_id.as_atom_def() {
            SAtom::Param(_) => {
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

                Some(wbox!(
                    knob_type,
                    AtomId::new(self.model_node_id, idx as u32),
                    center(pos.0, pos.1),
                    KnobData::new(param_name)))
            },
            SAtom::Setting(_) => {
                Some(wbox!(
                    self.wt_btn.clone(),
                    AtomId::new(self.model_node_id, idx as u32),
                    center(pos.0, pos.1),
                    ButtonData::new_toggle(param_name)))
            },
            _ => { None },
        }
    }

    pub fn rebuild(&mut self) {
        let wt_cont = Rc::new(Container::new());

        let mut cd = ContainerData::new();

        println!("REBUILD NODE UI: {} {}",
                 self.dsp_node_id,
                 self.model_node_id);
        cd.contrast_border().new_row();

        for idx in 0..4 {
            if let Some(wd) = self.build_atom_input((4, 12), idx) {
                cd.add(wd);
            }
        }

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

    prev_focus: NodeId,

    editor: MatrixEditorRef,
}

impl NodePanelData {
    pub fn new(node_id: u32, matrix: Arc<Mutex<Matrix>>, editor: MatrixEditorRef) -> Box<dyn std::any::Any> {
        let wt_cont = Rc::new(Container::new());

        let node_ui = Rc::new(RefCell::new(GenericNodeUI::new()));
        node_ui.borrow_mut().set_target(NodeId::Sin(0), 1);
        Box::new(Self {
            matrix,
            node_ui,
            editor,
            prev_focus: NodeId::Nop,
        })
    }

    fn check_focus_change(&mut self) {
        let cur_focus = self.editor.get_recent_focus();
        if cur_focus != self.prev_focus {
            self.prev_focus = cur_focus;

            if cur_focus != NodeId::Nop {
                self.node_ui.borrow_mut().set_target(
                    cur_focus,
                    self.matrix.lock().unwrap()
                        .unique_index_for(&cur_focus)
                        .unwrap_or(0)
                        as u32);
            }
        }
//        if prev_focus != cur_focus {
//            self.node_ui.set_target(
//        }
    }
}


#[derive(Debug)]
pub struct NodePanel {
}

impl NodePanel {
    pub fn new() -> Self {
        NodePanel { }
    }
}

impl WidgetType for NodePanel {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let pos = pos.shrink(UI_PADDING, UI_PADDING);

        data.with(|data: &mut NodePanelData| {
            data.check_focus_change();

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

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        data.with(|data: &mut NodePanelData| {
            let mut node_ui = data.node_ui.borrow_mut();
            if let Some(cont) = &mut node_ui.cont {
                cont.event(ui, ev);
            }
        });
    }
}
