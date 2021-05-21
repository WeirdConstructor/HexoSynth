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
};
use crate::matrix::Matrix;
use crate::dsp::NodeId;

use std::sync::{Arc, Mutex};
use std::rc::Rc;

pub struct PatternViewData {
    ui_ctrl:    UICtrlRef,
    cont:       WidgetData,
}

fn create_pattern_edit(id: AtomId, ui_ctrl: &UICtrlRef) -> WidgetData {
    let data = ui_ctrl.with_matrix(|m| m.get_pattern_data(0).unwrap());

    wbox!(
        PatternEditor::new_ref(6, 32),
        id,
        center(12, 12),
        PatternEditorData::new(data))
}

impl PatternViewData {
    pub fn new(ui_ctrl: UICtrlRef, id: AtomId)
        -> Box<dyn std::any::Any>
    {
        let cont = create_pattern_edit(id, &ui_ctrl);

        Box::new(Self {
            ui_ctrl,
            cont,
        })
    }

    pub fn check_cont_update(&mut self, _ui: &mut dyn WidgetUI) {
        self.ui_ctrl.with_matrix(|m|
            m.check_pattern_data(0));
    }
}

define_containing_widget!{PatternView, PatternViewData}

pub struct UtilPanelData {
    ui_ctrl:    UICtrlRef,
    cont:       WidgetData,
}

impl UtilPanelData {
    pub fn new(ui_ctrl: UICtrlRef)
        -> Box<dyn std::any::Any>
    {
        let mut tdata = TabsData::new();

        let id = {
            ui_ctrl.with_matrix(|m|
                m.unique_index_for(&NodeId::TSeq(0))
                 .unwrap_or(crate::PATTERN_VIEW_ID))
        };

        let id = AtomId::new(id as u32, 0);

        tdata.add(
            "Tracker",
            wbox!(
                PatternView::new_ref(),
                crate::PATTERN_VIEW_ID.into(),
                center(12, 12),
                PatternViewData::new(ui_ctrl.clone(), id)));

        Box::new(Self {
            ui_ctrl,
            cont: wbox!(
                Tabs::new_ref(),
                crate::UTIL_PANEL_ID.into(),
                center(12, 12),
                tdata),
        })
    }

    pub fn check_cont_update(&mut self, _ui: &mut dyn WidgetUI) {
    }
}

define_containing_widget!{UtilPanel, UtilPanelData}
