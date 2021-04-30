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
    Tabs, TabsData,
};
use crate::matrix::Matrix;
use crate::ui::matrix::MatrixEditorRef;

use std::sync::{Arc, Mutex};
use std::rc::Rc;

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

        Box::new(Self {
            cont: wbox!(
                Tabs::new_ref(), crate::UTIL_PANEL_ID.into(),
                center(12, 12), tdata
            ),
            matrix,
            editor,
        })
    }
}

define_containing_widget!{UtilPanel, UtilPanelData}
