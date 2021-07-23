use crate::UICtrlRef;

use hexotk::{UIPos, AtomId};
use hexotk::{
    Rect, WidgetUI, Painter, WidgetData, WidgetType,
    UIEvent,
    wbox,
    define_containing_widget,
    define_containing_widget_v_split,
};
use hexotk::widgets::{
    Container, ContainerData,
    Text, TextSourceRef, TextData,
    Button, ButtonData,
    PatternEditor, PatternEditorData,
    Tabs, TabsData,
    List, ListData, ListOutput,
};
use crate::dsp::NodeId;

use std::rc::Rc;

pub struct PatternViewData {
    cur_tseq:   Option<usize>,
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
        PatternEditor::new_ref(6, 41),
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
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ui_ctrl: UICtrlRef)
        -> Box<dyn std::any::Any>
    {
        let tseq_idx = cur_tseq_idx(&ui_ctrl);
        let idx =
            if let Some(idx) = tseq_idx { idx }
            else { 0 };

        let cont = create_pattern_edit(idx, &ui_ctrl);

        Box::new(Self {
            cur_tseq: tseq_idx,
            ui_ctrl,
            cont,
        })
    }

    pub fn check_cont_update(&mut self, _ui: &mut dyn WidgetUI) {
        let tseq_idx = cur_tseq_idx(&self.ui_ctrl);

        if tseq_idx != self.cur_tseq {
            self.cont     = create_pattern_edit(tseq_idx.unwrap_or(0), &self.ui_ctrl);
            self.cur_tseq = tseq_idx;
        }

        self.ui_ctrl.with_matrix(|m|
            m.check_pattern_data(self.cur_tseq.unwrap_or(0)));
    }
}

define_containing_widget!{PatternView, PatternViewData}

pub struct UtilPanelData {
    #[allow(dead_code)]
    ui_ctrl:        UICtrlRef,
    cont_top:       WidgetData,
    cont_bottom:    WidgetData,
}

impl UtilPanelData {
    #[allow(clippy::new_ret_no_self)]
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

        let wt_cont = Rc::new(Container::new());
        let mut top_cont = ContainerData::new();
        let wt_vers_text = Rc::new(Text::new(8.0));
        let txtsrc = Rc::new(TextSourceRef::new(20));
        txtsrc.set(&format!("v{}", crate::VERSION));

        let wt_btn = Rc::new(Button::new_height(60.0, 16.0, 8.0));
        top_cont
           .add(wbox!(
                wt_btn,
                AtomId::new(UICtrlRef::ATNID_HELP_BUTTON as u32, 0),
                left(10, 12),
                ButtonData::new_setting_click("Help")))
           .add(wbox!(
                wt_vers_text,
                AtomId::new(crate::UTIL_PANEL_VER_ID as u32, 0),
                center(2, 12),
                TextData::new(txtsrc)));

        Box::new(Self {
            ui_ctrl,
            cont_top: wbox!(
                wt_cont,
                AtomId::new(crate::UTIL_PANEL_TOP_ID as u32, 0),
                center(12, 12),
                top_cont),
            cont_bottom: wbox!(
                Tabs::new_ref(),
                AtomId::new(crate::UTIL_PANEL_ID as u32, 0),
                center(12, 12),
                tdata),
        })
    }

    pub fn check_cont_update(&mut self, ui: &mut dyn WidgetUI) {
        self.ui_ctrl.check_atoms(ui.atoms());
    }
}

define_containing_widget_v_split!{UtilPanel, UtilPanelData, 30.0}
