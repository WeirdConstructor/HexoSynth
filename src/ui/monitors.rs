// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use crate::UICtrlRef;

use hexotk::{UIPos, AtomId, wbox};
use hexotk::{Rect, WidgetUI, Painter, WidgetData, WidgetType, UIEvent};
use hexotk::constants::*;
use hexotk::widgets::{
    Container, ContainerData,
    TextSourceRef,
    GraphMinMax, GraphMinMaxData, GraphMinMaxSource
};

use std::rc::Rc;

use crate::matrix::Cell;

use crate::dsp::NodeId;


struct MonitorsSource {
    ui_ctrl:    UICtrlRef,
    idx:        usize,
}

use crate::CellDir;

fn sigidx2celldir(idx: usize) -> CellDir {
    match idx {
        0 => CellDir::T,
        1 => CellDir::TL,
        2 => CellDir::BL,
        3 => CellDir::TR,
        4 => CellDir::BR,
        5 => CellDir::B,
        _ => CellDir::C,
    }
}

impl GraphMinMaxSource for MonitorsSource {
    fn read(&mut self, buf: &mut [(f64, f64)]) {
        self.ui_ctrl.with_matrix(|m| {
            let cell = m.monitored_cell();
            if !cell.has_dir_set(sigidx2celldir(self.idx)) {
                for b in buf.iter_mut() {
                    *b = (0.0, 0.0);
                }
                return;
            }

            let mimbuf = m.get_minmax_monitor_samples(self.idx);
            for (i, b) in buf.iter_mut().enumerate() {
                let mm = mimbuf.at(i);
                *b = (mm.0 as f64, mm.1 as f64);
            }
        });
    }
}

pub struct MonitorsData {
    ui_ctrl:        UICtrlRef,
    cont:           WidgetData,
    last_cell:      Cell,
    sig_labels:     [Rc<TextSourceRef>; 6],
}

impl MonitorsData {
    pub fn new(id: AtomId, ui_ctrl: UICtrlRef) -> Self {
        let wt_cont = Rc::new(Container::new());
        let wt_gmm  = Rc::new(GraphMinMax::new(128.0, 64.0));

        let mut cd = ContainerData::new();

        let sig_labels = [
            Rc::new(TextSourceRef::new(100)),
            Rc::new(TextSourceRef::new(100)),
            Rc::new(TextSourceRef::new(100)),
            Rc::new(TextSourceRef::new(100)),
            Rc::new(TextSourceRef::new(100)),
            Rc::new(TextSourceRef::new(100)),
        ];

        let build_minmaxdata = |idx: usize| -> Box<dyn std::any::Any> {
            GraphMinMaxData::new(
                9.0,
                sig_labels[idx].clone(),
                crate::monitor::MONITOR_MINMAX_SAMPLES,
                Box::new(MonitorsSource {
                    ui_ctrl: ui_ctrl.clone(),
                    idx,
                    //d// cnt: 0,
                }))
        };

        cd
            .new_row()
            .add(wbox!(
                wt_gmm, AtomId::new(id.node_id(), id.atom_id() + 1),
                center(6, 4), build_minmaxdata(0)))
            .add(wbox!(
                wt_gmm, AtomId::new(id.node_id(), id.atom_id() + 2),
                center(6, 4), build_minmaxdata(3)))
            .new_row()
            .add(wbox!(
                wt_gmm, AtomId::new(id.node_id(), id.atom_id() + 3),
                center(6, 4), build_minmaxdata(1)))
            .add(wbox!(
                wt_gmm, AtomId::new(id.node_id(), id.atom_id() + 4),
                center(6, 4), build_minmaxdata(4)))
            .new_row()
            .add(wbox!(
                wt_gmm, AtomId::new(id.node_id(), id.atom_id() + 5),
                center(6, 4), build_minmaxdata(2)))
            .add(wbox!(
                wt_gmm, AtomId::new(id.node_id(), id.atom_id() + 6),
                center(6, 4), build_minmaxdata(5)));

        Self {
            cont:       wbox!(wt_cont, id, center(12, 12), cd),
            last_cell:  Cell::empty(NodeId::Nop),
            ui_ctrl,
            sig_labels,
        }
    }

    fn check_labels(&mut self) {
        let ui_ctrl = self.ui_ctrl.clone();
        ui_ctrl.with_matrix(|m| {
            let c = m.monitored_cell();

            if self.last_cell != *c {
                self.last_cell = *c;

                let mut buf : [u8; 30] = [0; 30];

                if let Some((lbl, _)) = m.edge_label(&c, CellDir::T, &mut buf[..]) {
                    self.sig_labels[0].set(lbl);
                } else {
                    self.sig_labels[0].set("-");
                }

                if let Some((lbl, _)) = m.edge_label(&c, CellDir::TL, &mut buf[..]) {
                    self.sig_labels[1].set(lbl);
                } else {
                    self.sig_labels[1].set("-");
                }

                if let Some((lbl, _)) = m.edge_label(&c, CellDir::BL, &mut buf[..]) {
                    self.sig_labels[2].set(lbl);
                } else {
                    self.sig_labels[2].set("-");
                }

                if let Some((lbl, _)) = m.edge_label(&c, CellDir::TR, &mut buf[..]) {
                    self.sig_labels[3].set(lbl);
                } else {
                    self.sig_labels[3].set("-");
                }

                if let Some((lbl, _)) = m.edge_label(&c, CellDir::BR, &mut buf[..]) {
                    self.sig_labels[4].set(lbl);
                } else {
                    self.sig_labels[4].set("-");
                }

                if let Some((lbl, _)) = m.edge_label(&c, CellDir::B, &mut buf[..]) {
                    self.sig_labels[5].set(lbl);
                } else {
                    self.sig_labels[5].set("-");
                }
            }
        });
    }
}

#[derive(Debug)]
pub struct Monitors {
}

impl Monitors {
    pub fn new() -> Self {
        Monitors { }
    }
}

impl WidgetType for Monitors {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let pos = pos.shrink(UI_PADDING, UI_PADDING);

        data.with(|data: &mut MonitorsData| {
            data.check_labels();

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
