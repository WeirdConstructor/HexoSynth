use crate::ui::node_panel::GenericNodeUI;
use std::rc::Rc;
use std::cell::RefCell;

use hexotk::WidgetData;

#[derive(Clone)]
pub struct DynamicWidgets {
    pub node_ui:    Rc<RefCell<GenericNodeUI>>,
    pub patedit_ui: Rc<RefCell<Option<WidgetData>>>,
}

impl DynamicWidgets {
    pub fn new() -> Self {
        let patedit_ui : Rc<RefCell<Option<WidgetData>>> =
            Rc::new(RefCell::new(None));
        Self {
            node_ui:    Rc::new(RefCell::new(GenericNodeUI::new())),
            patedit_ui,
        }
    }
}

impl std::fmt::Debug for DynamicWidgets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>)
        -> Result<(), std::fmt::Error>
    {
        f.write_fmt(format_args!("DebugWidgets()"))
    }
}

