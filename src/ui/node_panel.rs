// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::UICtrlRef;
use crate::dsp::{NodeId, NodeInfo, SAtom, GraphAtomData};
use crate::ui::monitors::{Monitors, MonitorsData};

use hexotk::{UIPos, AtomId, wbox};
use hexotk::{Rect, WidgetUI, Painter, WidgetData, WidgetType, UIEvent};
use hexotk::constants::*;
use hexotk::widgets::{
    Container, ContainerData,
    Knob, KnobData,
    Button, ButtonData,
    Text, TextSourceRef, TextData,
    Graph, GraphData,
    // List, ListData, ListItems, ListOutput,
    Entry, EntryData,
};

use std::rc::Rc;
use std::cell::RefCell;

const PANEL_HELP_TEXT_ID      : u32 = 1;
const PANEL_HELP_TEXT_CONT_ID : u32 = 2;
const PANEL_GRAPH_ID          : u32 = 3;
const PANEL_SUB_CONT_ID       : u32 = 4;
const PANEL_MAIN_CONT_ID      : u32 = 5;

struct GraphAtomDataAdapter<'a> {
    node_idx: u32,
    ui: &'a dyn WidgetUI,
}

impl<'a> GraphAtomData for GraphAtomDataAdapter<'a> {
    fn get(&self, param_idx: u32) -> Option<SAtom> {
        Some(self.ui.atoms().get(AtomId::new(self.node_idx, param_idx))?
             .clone()
             .into())
    }

    fn get_denorm(&self, param_idx: u32) -> f32 {
        self.ui.atoms()
            .get_denorm(AtomId::new(self.node_idx, param_idx))
            .unwrap_or(0.0)
    }

    fn get_norm(&self, param_idx: u32) -> f32 {
        self.ui.atoms()
            .get(AtomId::new(self.node_idx, param_idx))
            .map(|a| a.f())
            .unwrap_or(0.0)
    }

    fn get_phase(&self) -> f32 {
        self.ui.atoms()
            .get_phase_value(AtomId::new(self.node_idx, 0))
            .unwrap_or(0.0)
    }

    fn get_led(&self) -> f32 {
        self.ui.atoms()
            .get_led_value(AtomId::new(self.node_idx, 0))
            .unwrap_or(0.0)
    }
}

pub struct GenericNodeUI {
    dsp_node_id:    NodeId,
    model_node_id:  u32,
    info:           Option<NodeInfo>,
    cont:           Option<Box<WidgetData>>,

    wt_knob_01:    Rc<Knob>,
    wt_btn:        Rc<Button>,
    wt_text:       Rc<Text>,
    wt_graph:      Rc<Graph>,
    wt_editview:   Rc<Entry>,

    help_txt:      Rc<TextSourceRef>,
}

impl GenericNodeUI {
    pub fn new() -> Self {
        let wt_knob_01    = Rc::new(Knob::new(28.0, 12.0, 9.0));
        let wt_btn        = Rc::new(Button::new(62.0, 12.0));
        let wt_text       = Rc::new(Text::new(10.0));
        let wt_graph      = Rc::new(Graph::new(240.0, 100.0));
        let wt_editview   = Rc::new(Entry::new_not_editable(70.0, 10.0, 13));

        Self {
            dsp_node_id:    NodeId::Nop,
            model_node_id:  0,
            info:           None,
            cont:           None,
            help_txt:       Rc::new(TextSourceRef::new(55)),
            wt_knob_01,
            wt_btn,
            wt_text,
            wt_graph,
            wt_editview,
        }
    }

    pub fn set_target(
        &mut self, dsp_node_id: NodeId, model_node_id: u32,
        a: &mut crate::actions::ActionState)
    {
        self.dsp_node_id   = dsp_node_id;
        self.model_node_id = model_node_id;
        self.info          = Some(NodeInfo::from_node_id(dsp_node_id));

        self.rebuild(a);
    }

    fn build_atom_input(&self, pos: (u8, u8), idx: usize)
        -> Option<WidgetData>
    {
        let param_id = self.dsp_node_id.param_by_idx(idx)?;
        let param_name = param_id.name();

        match param_id.as_atom_def() {
            SAtom::Param(_) => {
                // FIXME: Widget type should be determined by the Atom enum!
                if param_name == "trig" {
                    Some(wbox!(
                        self.wt_btn.clone(),
                        AtomId::new(self.model_node_id, idx as u32),
                        center(pos.0, pos.1),
                        ButtonData::new_param_click(param_name)))
                } else {
                    let knob_type = self.wt_knob_01.clone();

                    Some(wbox!(
                        knob_type,
                        AtomId::new(self.model_node_id, idx as u32),
                        center(pos.0, pos.1),
                        KnobData::new(param_name)))
                }
            },
            SAtom::Setting(_) => {
                Some(wbox!(
                    self.wt_btn.clone(),
                    AtomId::new(self.model_node_id, idx as u32),
                    center(pos.0, pos.1),
                    ButtonData::new_toggle(param_name)))
            },
            SAtom::AudioSample((_filename, _)) => {
                Some(wbox!(
                    self.wt_editview.clone(),
                    AtomId::new(self.model_node_id, idx as u32),
                    center(pos.0, pos.1),
                    EntryData::new("Sample:")))
            },
            _ => { None },
        }
    }

    pub fn check_hover_text_for(&mut self, at_id: AtomId) {
        if at_id.node_id() == self.model_node_id {
            if let Some(info) = &self.info {
                if let Some(txt) = info.in_help(at_id.atom_id() as usize) {
                    self.help_txt.set(txt);
                }
            }
        }
    }

    pub fn rebuild(&mut self, a: &mut crate::actions::ActionState) {
        let wt_cont = Rc::new(Container::new());

        let mut param_cd = ContainerData::new();

        param_cd.contrast_border().new_row();

        for idx in 0..4 {
            if let Some(wd) = self.build_atom_input((3, 3), idx) {
                param_cd.add(wd);
            }
        }

        if self.dsp_node_id.param_by_idx(4).is_some() {
            param_cd.new_row();
            for idx in 0..4 {
                if let Some(wd) = self.build_atom_input((3, 3), 4 + idx) {
                    param_cd.add(wd);
                }
            }
        }

        if self.dsp_node_id.param_by_idx(8).is_some() {
            param_cd.new_row();
            for idx in 0..4 {
                if let Some(wd) = self.build_atom_input((3, 3), 8 + idx) {
                    param_cd.add(wd);
                }
            }
        }

        if self.dsp_node_id.param_by_idx(12).is_some() {
            param_cd.new_row();
            for idx in 0..4 {
                if let Some(wd) = self.build_atom_input((3, 3), 12 + idx) {
                    param_cd.add(wd);
                }
            }
        }

        if let Some(mut graph_fun) = self.dsp_node_id.graph_fun() {
            let node_id = self.dsp_node_id;
            let node_idx =
                a.matrix.unique_index_for(&node_id).unwrap_or(0) as u32;

            let graph_fun =
                Box::new(move |ui: &dyn WidgetUI, init: bool, x: f64, xn: f64| -> f64 {
                    let gd = GraphAtomDataAdapter { node_idx, ui };
                    graph_fun(&gd, init, x as f32, xn as f32) as f64
                });

            param_cd.new_row()
              .add(wbox!(self.wt_graph,
                   AtomId::new(crate::NODE_PANEL_ID, PANEL_GRAPH_ID),
                   center(12, 6),
                   GraphData::new(160, graph_fun)));
        }


        let mut txt_cd = ContainerData::new();
        txt_cd
            .level(1)
            .shrink(0.0, 0.0)
//            .contrast_border()
            .border();

        txt_cd.new_row()
            .add(wbox!(self.wt_text,
               AtomId::new(crate::NODE_PANEL_ID, PANEL_HELP_TEXT_ID),
               center(12, 12),
               TextData::new(self.help_txt.clone())));

        let mut panel_cd = ContainerData::new();
        panel_cd
            .new_row().add(wbox!(
                wt_cont,
                AtomId::new(crate::NODE_PANEL_ID, PANEL_SUB_CONT_ID),
                center(12, 9),
                param_cd));

        panel_cd.new_row()
          .add(wbox!(wt_cont,
               AtomId::new(crate::NODE_PANEL_ID, PANEL_HELP_TEXT_CONT_ID),
               center(12, 3),
               txt_cd));

        self.cont =
            Some(Box::new(wbox!(
                wt_cont,
                AtomId::new(crate::NODE_PANEL_ID, PANEL_MAIN_CONT_ID),
                center(12, 12),
                panel_cd)));
    }
}

pub struct NodePanelData {
    node_ui:    Rc<RefCell<GenericNodeUI>>,
    monitors:   WidgetData,
}

#[allow(clippy::new_ret_no_self)]
impl NodePanelData {
    pub fn new(ui_ctrl: UICtrlRef, node_id: u32) -> Box<dyn std::any::Any> {
//        let node_ui = Rc::new(RefCell::new(GenericNodeUI::new(ui_ctrl.clone())));
//        node_ui.borrow_mut().set_target(NodeId::Sin(0), 1);
        let node_ui = ui_ctrl.with_state(|s| s.widgets.node_ui.clone());

        let wt_monitors = Rc::new(Monitors::new());
        let monitors =
            wbox!(
                wt_monitors,
                AtomId::new(node_id, 100),
                center(12, 12),
                Box::new(MonitorsData::new(
                    AtomId::new(node_id, 101),
                    ui_ctrl)));

        Box::new(Self {
            node_ui,
            monitors,
        })
    }

//    fn check_focus_change(&mut self) {
//        if self.ui_ctrl.with_state(|s| s.node_panel_rebuild) {
//            let (node_id, uniq_idx) =
//                self.ui_ctrl.with_state(
//                    |s| (s.focus_cell.node_id(), s.focus_uniq_node_idx));
//
//            self.node_ui.borrow_mut()
//                .set_target(node_id, uniq_idx);
//
////            self.ui_ctrl.with_state(|s| { s.node_panel_rebuild = false; });
//        }
//    }
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
            p.rect_fill(UI_BG_CLR, pos.x, pos.y, pos.w, pos.h);

            let mut node_ui = data.node_ui.borrow_mut();
            if let Some(at_id) = ui.hover_atom_id() {
                node_ui.check_hover_text_for(at_id);
            }

            let monitor_height = 220.0;

            let cont_pos = pos.crop_bottom(monitor_height);

            if let Some(cont) = &mut node_ui.cont {
                cont.draw(ui, p, cont_pos);
            }

            let monitor_pos = pos.crop_top(pos.h - monitor_height);
            data.monitors.draw(ui, p, monitor_pos);
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
