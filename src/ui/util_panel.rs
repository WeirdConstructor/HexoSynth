use hexotk::{MButton, UIPos, AtomId};
use hexotk::{
    Rect, WidgetUI, Painter, WidgetData, WidgetType,
    UIEvent,
    wbox,
    define_containing_widget
};
use hexotk::constants::*;
use hexotk::widgets::{
//    Container, ContainerData,
//    Text, TextSourceRef, TextData,
    PatternEditor, PatternEditorData,
    Tabs, TabsData,
};
use crate::matrix::Matrix;
use crate::ui::matrix::MatrixEditorRef;

use std::sync::{Arc, Mutex};
use std::rc::Rc;

pub struct PatternViewData {
    matrix: Arc<Mutex<Matrix>>,
    cont:   WidgetData,
}

fn create_pattern_edit(matrix: Arc<Mutex<Matrix>>) -> WidgetData {
    let m = matrix.lock().unwrap();
    let data = m.get_pattern_data(0).unwrap();
    wbox!(
        PatternEditor::new_ref(6, 32),
        crate::PATTERN_VIEW_ID.into(),
        center(12, 12),
        PatternEditorData::new(data))
}

impl PatternViewData {
    pub fn new(matrix: Arc<Mutex<Matrix>>)
        -> Box<dyn std::any::Any>
    {
        let cont = create_pattern_edit(matrix.clone());

        Box::new(Self {
            matrix,
            cont,
        })
    }

    pub fn check_cont_update(&mut self, ui: &mut dyn WidgetUI) {
        let mut m = self.matrix.lock().unwrap();
        m.check_pattern_data(0);
    }
}

define_containing_widget!{PatternView, PatternViewData}

pub struct UtilPanelData {
    matrix: Arc<Mutex<Matrix>>,
    editor: MatrixEditorRef,
    cont:   WidgetData,
}

impl UtilPanelData {
    pub fn new(matrix: Arc<Mutex<Matrix>>, editor: MatrixEditorRef)
        -> Box<dyn std::any::Any>
    {
        let mut tdata = TabsData::new();

        tdata.add(
            "Tracker",
            wbox!(
                PatternView::new_ref(),
                crate::PATTERN_PANEL_ID.into(),
                center(12, 12),
                PatternViewData::new(matrix.clone())));

        Box::new(Self {
            cont: wbox!(
                Tabs::new_ref(),
                crate::UTIL_PANEL_ID.into(),
                center(12, 12),
                tdata),
            matrix,
            editor,
        })
    }

    pub fn check_cont_update(&mut self, ui: &mut dyn WidgetUI) {
    }
}

define_containing_widget!{UtilPanel, UtilPanelData}
