use hexotk::{UIPos, AtomId, wbox};
use hexotk::{Rect, WidgetUI, Painter, WidgetData, WidgetType, UIEvent};
use hexotk::constants::*;
use hexotk::widgets::{
    Container, ContainerData,
    Knob, KnobData,
    Button, ButtonData,
    Text, TextSourceRef, TextData,
    Graph, GraphData,
    GraphMinMax, GraphMinMaxData,
};

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

use crate::matrix::{Matrix, Cell};

use crate::dsp::NodeId;

pub struct MonitorsData {
    matrix: Arc<Mutex<Matrix>>,
    cont:   WidgetData,
    last_cell: Cell,
}

impl MonitorsData {
    pub fn new(id: AtomId, matrix: Arc<Mutex<Matrix>>) -> Self {
        let wt_cont = Rc::new(Container::new());
        let wt_gmm  = Rc::new(GraphMinMax::new(128.0, 64.0));

        let mut cd = ContainerData::new();

        let txtsrc2 = Rc::new(TextSourceRef::new(100));
        txtsrc2.set("sig");

        let fun1 =
            Box::new(move |ui: &dyn WidgetUI, idx: usize| -> (f64, f64) {
                (-1.0, 1.0)
            });

        cd.contrast_border()
            .new_row()
            .add(wbox!(
                wt_gmm,
                AtomId::new(id.node_id(), id.atom_id() + 1),
                center(12, 3),
                GraphMinMaxData::new(
                    9.0,
                    txtsrc2,
                    crate::monitor::MONITOR_MINMAX_SAMPLES,
                    fun1)));

        Self {
            matrix,
            cont: wbox!(wt_cont, id, center(12, 12), cd),
            last_cell: Cell::empty(NodeId::Nop),
        }
    }
}

#[derive(Debug)]
pub struct Monitors {
}

impl Monitors {
    pub fn new(matrix: Arc<Mutex<Matrix>>) -> Self {
        Monitors { }
    }
}

impl WidgetType for Monitors {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let pos = pos.shrink(UI_PADDING, UI_PADDING);

        data.with(|data: &mut MonitorsData| {
            data.cont.draw(ui, p, pos);
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, avail: (f64, f64)) -> (f64, f64) {
        avail
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        data.with(|data: &mut MonitorsData| {
            data.cont.event(ui, ev);
        });
    }
}
