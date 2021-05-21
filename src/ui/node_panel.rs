// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use crate::UICtrlRef;
use crate::matrix::Cell;
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
    List, ListData, ListItems, ListOutput,
};

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

const PANEL_HELP_TEXT_ID      : u32 = 1;
const PANEL_HELP_TEXT_CONT_ID : u32 = 2;
const PANEL_GRAPH_ID          : u32 = 3;

struct GraphAtomDataAdapter<'a> {
    ui: &'a dyn WidgetUI,
}

impl<'a> GraphAtomData for GraphAtomDataAdapter<'a> {
    fn get(&self, node_id: usize, param_idx: u32) -> Option<SAtom> {
        Some(self.ui.atoms().get(AtomId::new(node_id as u32, param_idx))?
             .clone()
             .into())
    }

    fn get_denorm(&self, node_id: usize, param_idx: u32) -> f32 {
        self.ui.atoms()
            .get_denorm(
                AtomId::new(node_id as u32, param_idx))
            .unwrap_or(0.0)
    }
}

pub struct GenericNodeUI {
    ui_ctrl:        UICtrlRef,

    dsp_node_id:    NodeId,
    model_node_id:  u32,
    info:           Option<NodeInfo>,
    cont:           Option<Box<WidgetData>>,

    wt_knob_01:    Rc<Knob>,
    wt_knob_11:    Rc<Knob>,
    wt_btn:        Rc<Button>,
    wt_text:       Rc<Text>,
    wt_graph:      Rc<Graph>,
    wt_sampl_list: Rc<List>,

    sample_list:   ListItems,

    help_txt:      Rc<TextSourceRef>,
}

impl GenericNodeUI {
    pub fn new(ui_ctrl: UICtrlRef) -> Self {
        let wt_knob_01 =
            Rc::new(Knob::new(25.0, 12.0, 9.0));
        let wt_knob_11 =
            Rc::new(Knob::new(25.0, 12.0, 9.0).range_signed());
        let wt_btn   = Rc::new(Button::new(50.0, 12.0));
        let wt_text  = Rc::new(Text::new(12.0));
        let wt_graph = Rc::new(Graph::new(240.0, 100.0));
        let wt_sampl_list = Rc::new(List::new(60.0, 12.0, 4));

        let sample_list = ListItems::new(8);
        sample_list.push(0, String::from("bd.wav"));
        sample_list.push(1, String::from("sd.wav"));
        sample_list.push(2, String::from("hh.wav"));
        sample_list.push(3, String::from("oh.wav"));
        sample_list.push(4, String::from("tom1.wav"));
        sample_list.push(5, String::from("tom2.wav"));
        sample_list.push(6, String::from("bd808.wav"));
        sample_list.push(7, String::from("bd909.wav"));

        Self {
            ui_ctrl,
            dsp_node_id:    NodeId::Nop,
            model_node_id:  0,
            info:           None,
            cont:           None,
            help_txt:       Rc::new(TextSourceRef::new(42)),
            wt_sampl_list,
            wt_knob_01,
            wt_knob_11,
            wt_btn,
            wt_text,
            wt_graph,
            sample_list,
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
                    if let Some((min, _max)) = param_id.param_min_max() {
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
            SAtom::AudioSample((filename, _)) => {
                println!("AUDIO SAMPLE: {}", filename);
                Some(wbox!(
                    self.wt_sampl_list.clone(),
                    AtomId::new(self.model_node_id, idx as u32),
                    center(pos.0, pos.1),
                    ListData::new("Sample:", ListOutput::ByAudioSample, self.sample_list.clone())))
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

    pub fn rebuild(&mut self) {
        let wt_cont = Rc::new(Container::new());

        let mut cd = ContainerData::new();

        cd.contrast_border().new_row();

        for idx in 0..4 {
            if let Some(wd) = self.build_atom_input((3, 3), idx) {
                cd.add(wd);
            }
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

        if let Some(mut graph_fun) = self.dsp_node_id.graph_fun() {
            let graph_fun =
                Box::new(move |ui: &dyn WidgetUI, init: bool, x: f64| -> f64 {
                    let gd = GraphAtomDataAdapter { ui };
                    graph_fun(&gd, init, x as f32) as f64
                });

            cd.new_row()
              .add(wbox!(self.wt_graph,
                   AtomId::new(crate::NODE_PANEL_ID, PANEL_GRAPH_ID),
                   center(12, 3),
                   GraphData::new(30, graph_fun)));
        }

        cd.new_row()
          .add(wbox!(wt_cont,
               AtomId::new(crate::NODE_PANEL_ID, PANEL_HELP_TEXT_CONT_ID),
               center(12, 3),
               txt_cd));

        self.cont =
            Some(Box::new(wbox!(
                wt_cont,
                AtomId::new(self.model_node_id, 1),
                center(12, 12), cd)));
    }
}

pub struct NodePanelData {
    ui_ctrl:    UICtrlRef,
    node_ui:    Rc<RefCell<GenericNodeUI>>,
    monitors:   WidgetData,
    prev_focus: Cell,
}

impl NodePanelData {
    pub fn new(ui_ctrl: UICtrlRef, node_id: u32) -> Box<dyn std::any::Any> {
        let node_ui = Rc::new(RefCell::new(GenericNodeUI::new(ui_ctrl.clone())));
        node_ui.borrow_mut().set_target(NodeId::Sin(0), 1);

        let wt_monitors = Rc::new(Monitors::new());
        let monitors =
            wbox!(
                wt_monitors,
                AtomId::new(node_id, 100),
                center(12, 12),
                Box::new(MonitorsData::new(
                    AtomId::new(node_id, 101),
                    ui_ctrl.clone())));

        Box::new(Self {
            ui_ctrl,
            node_ui,
            monitors,
            prev_focus: Cell::empty(NodeId::Nop),
        })
    }

    fn check_focus_change(&mut self) {
        let cur_focus = self.ui_ctrl.get_recent_focus();

        if cur_focus != self.prev_focus {
            self.prev_focus = cur_focus;

            if cur_focus.node_id() != NodeId::Nop {
                self.ui_ctrl.with_matrix(|m| {
                    self.node_ui.borrow_mut().set_target(
                        cur_focus.node_id(),
                        m.unique_index_for(&cur_focus.node_id())
                         .unwrap_or(0) as u32);

                    m.monitor_cell(cur_focus);
                });
            }
        }
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

            let mut node_ui = data.node_ui.borrow_mut();
            if let Some(at_id) = ui.hover_atom_id() {
                node_ui.check_hover_text_for(at_id);
            }

            let monitor_height = 260.0;

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
