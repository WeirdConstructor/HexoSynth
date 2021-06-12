use crate::UICtrlRef;

use hexotk::{UIPos, AtomId};
use hexotk::{
    Rect, WidgetUI, Painter, WidgetData, WidgetType,
    UIEvent,
    wbox,
    define_containing_widget
};
use hexotk::widgets::{
//    Container, ContainerData,
//    Text, TextSourceRef, TextData,
    PatternEditor, PatternEditorData,
    Tabs, TabsData,
    List, ListData, ListOutput,
};
use crate::dsp::NodeId;

use std::rc::Rc;

pub struct PatternViewData {
    cur_tseq:   usize,
    ui_ctrl:    UICtrlRef,
    cont:       WidgetData,
}

fn create_pattern_edit(tseq_idx: usize, ui_ctrl: &UICtrlRef) -> WidgetData {
    let data =
        ui_ctrl.with_matrix(|m|
            m.get_pattern_data(tseq_idx)
             .unwrap());

    let id = {
        ui_ctrl.with_matrix(|m|
            m.unique_index_for(&NodeId::TSeq(tseq_idx as u8))
             .unwrap_or(crate::PATTERN_VIEW_ID))
    };

    wbox!(
        PatternEditor::new_ref(6, 44),
        AtomId::new(id as u32, 0),
        center(12, 12),
        PatternEditorData::new(data))
}

fn cur_tseq_idx(ui_ctrl: &UICtrlRef) -> Option<usize> {
    let nid = ui_ctrl.get_focus_id();
    if nid.to_instance(0) == NodeId::TSeq(0) {
        Some(nid.instance())
    } else {
        None
    }
}

impl PatternViewData {
    pub fn new(ui_ctrl: UICtrlRef)
        -> Box<dyn std::any::Any>
    {
        let idx =
            if let Some(idx) = cur_tseq_idx(&ui_ctrl) {
                idx
            } else { 0 };

        let cont = create_pattern_edit(idx, &ui_ctrl);

        Box::new(Self {
            cur_tseq: idx,
            ui_ctrl,
            cont,
        })
    }

    pub fn check_cont_update(&mut self, _ui: &mut dyn WidgetUI) {
        let idx =
            if let Some(idx) = cur_tseq_idx(&self.ui_ctrl) {
                idx
            } else { 0 };

        if idx != self.cur_tseq {
            self.cont     = create_pattern_edit(idx, &self.ui_ctrl);
            self.cur_tseq = idx;
        }

        self.ui_ctrl.with_matrix(|m|
            m.check_pattern_data(self.cur_tseq));
    }
}

define_containing_widget!{PatternView, PatternViewData}

pub struct UtilPanelData {
    #[allow(dead_code)]
    ui_ctrl:    UICtrlRef,
    cont:       WidgetData,
}

impl UtilPanelData {
    pub fn new(ui_ctrl: UICtrlRef)
        -> Box<dyn std::any::Any>
    {
        let mut tdata = TabsData::new();

        tdata.add(
            "Tracker",
            wbox!(
                PatternView::new_ref(),
                crate::PATTERN_VIEW_ID.into(),
                center(12, 12),
                PatternViewData::new(ui_ctrl.clone())));

        let wt_sampl_list = Rc::new(List::new(330.0, 12.0, 35));

        ui_ctrl.reload_sample_dir_list();
        let sample_list = ui_ctrl.get_sample_dir_list();
//        sample_list.push(0, String::from("bd.wav"));
//        sample_list.push(1, String::from("sd.wav"));
//        sample_list.push(2, String::from("hh.wav"));
//        sample_list.push(3, String::from("oh.wav"));
//        sample_list.push(4, String::from("tom1.wav"));
//        sample_list.push(5, String::from("tom2.wav"));
//        sample_list.push(6, String::from("bd808.wav"));
//        sample_list.push(7, String::from("bd909.wav"));
//        sample_list.push(8, String::from("0123456789012345678901234567890123456789012345678901234567890123456789"));

        tdata.add(
            "Samples",
            wbox!(
                wt_sampl_list,
                AtomId::new(UICtrlRef::ATNID_SAMPLE_LOAD_ID, 0),
                center(12, 12),
                ListData::new(
                    "Sample:",
                    ListOutput::BySetting,
                    sample_list)));

        Box::new(Self {
            ui_ctrl,
            cont: wbox!(
                Tabs::new_ref(),
                AtomId::new(crate::UTIL_PANEL_ID as u32, 0),
                center(12, 12),
                tdata),
        })
    }

    pub fn check_cont_update(&mut self, _ui: &mut dyn WidgetUI) {
    }
}

define_containing_widget!{UtilPanel, UtilPanelData}
