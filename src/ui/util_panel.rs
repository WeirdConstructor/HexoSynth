// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::UICtrlRef;
use crate::state::*;

use hexotk::{UIPos, AtomId};
use hexotk::{
    Rect, WidgetUI, Painter, WidgetData, WidgetType,
    UIEvent,
    wbox,
    define_containing_opt_shared_widget,
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
use std::cell::RefCell;

pub struct PatternViewData {
    cont: Rc<RefCell<Option<WidgetData>>>,
}

pub fn create_pattern_edit(a: &mut crate::actions::ActionState) -> WidgetData {
    let tseq_idx = a.state.current_tracker_idx;

    let data =
        a.matrix.get_pattern_data(tseq_idx)
         .unwrap();

    let id = {
        a.matrix
            .unique_index_for(&NodeId::TSeq(tseq_idx as u8))
            .unwrap_or(crate::PATTERN_VIEW_ID)
    };

    wbox!(
        PatternEditor::new_ref(6, 41),
        AtomId::new(id as u32, 0),
        center(12, 12),
        PatternEditorData::new(data))
}


impl PatternViewData {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ui_ctrl: UICtrlRef)
        -> Box<dyn std::any::Any>
    {
        Box::new(Self {
            cont: ui_ctrl.with_state(|s| s.widgets.patedit_ui.clone()),
        })
    }

    pub fn check_cont_update(&mut self, _ui: &mut dyn WidgetUI) {
    }
}

define_containing_opt_shared_widget!{PatternView, PatternViewData}

pub struct UtilPanelData {
    #[allow(dead_code)]
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
                AtomId::new(ATNID_SAMPLE_LOAD_ID, 0),
                center(12, 12),
                ListData::new(
                    "Sample:",
                    ListOutput::BySetting,
                    sample_list)));

        let wt_cont = Rc::new(Container::new());
        let mut top_cont = ContainerData::new();
        let wt_vers_text = Rc::new(Text::new(7.0));
        let txtsrc = Rc::new(TextSourceRef::new(20));
        txtsrc.set(&format!("v{}", crate::VERSION));

        let wt_btn = Rc::new(Button::new_height(60.0, 16.0, 8.0));
        top_cont
           .add(wbox!(
                wt_btn,
                AtomId::new(ATNID_HELP_BUTTON as u32, 0),
                left(3, 12),
                ButtonData::new_setting_click("Help")))
           .add(wbox!(
                wt_btn,
                AtomId::new(ATNID_SAVE_BUTTON as u32, 0),
                left(6, 12),
                ButtonData::new_setting_click("Save")))
           .add(wbox!(
                wt_vers_text,
                AtomId::new(crate::UTIL_PANEL_VER_ID as u32, 0),
                center(3, 12),
                TextData::new(txtsrc)));

        Box::new(Self {
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

    pub fn check_cont_update(&mut self, _ui: &mut dyn WidgetUI) {
    }
}

define_containing_widget_v_split!{UtilPanel, UtilPanelData, 30.0}
