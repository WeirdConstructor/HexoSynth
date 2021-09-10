// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use tuix::*;

mod hexo_consts;
mod painter;
mod hexgrid;
mod rect;

use painter::FemtovgPainter;
use hexgrid::{HexGrid, HexGridModel, HexCell, HexDir, HexEdge, HexHLight};
use hexo_consts::MButton;

use std::rc::Rc;
use std::cell::RefCell;

struct TestGridModel {
    last_click: (usize, usize),
    drag_to:    (usize, usize),
}

impl TestGridModel {
    pub fn new() -> Self {
        Self {
            last_click: (1000, 1000),
            drag_to: (1000, 1000),
        }
    }
}

impl HexGridModel for TestGridModel {
    fn width(&self) -> usize { 16 }
    fn height(&self) -> usize { 16 }
    fn cell_visible(&self, x: usize, y: usize) -> bool {
        x < self.width() && y < self.height()
    }
    fn cell_empty(&self, x: usize, y: usize) -> bool {
        !(x < self.width() && y < self.height())
    }
    fn cell_color(&self, x: usize, y: usize) -> u8 { 0 }
    fn cell_label<'a>(&self, x: usize, y: usize, out: &'a mut [u8])
        -> Option<HexCell<'a>>
    {
        let w = self.width();
        let h = self.height();
        if x >= w || y >= h { return None; }

        let mut hlight = HexHLight::Normal;

        use std::io::Write;
        let mut cur = std::io::Cursor::new(out);
        let len =
            if self.last_click == (x, y) {
                hlight = HexHLight::Select;
                match write!(cur, "CLICK") {
                    Ok(_)  => { cur.position() as usize },
                    Err(_) => 0,
                }
            } else if self.drag_to == (x, y) {
                hlight = HexHLight::HLight;
                match write!(cur, "DRAG") {
                    Ok(_)  => { cur.position() as usize },
                    Err(_) => 0,
                }
            } else {
                match write!(cur, "{}x{}", x, y) {
                    Ok(_)  => { cur.position() as usize },
                    Err(_) => 0,
                }
            };

        if len == 0 {
            return None;
        }

        Some(HexCell {
            label:
                std::str::from_utf8(&(cur.into_inner())[0..len])
                .unwrap(),
            hlight,
            rg_colors: Some(( 1.0, 1.0,)),
        })
    }

    /// Edge: 0 top-right, 1 bottom-right, 2 bottom, 3 bottom-left, 4 top-left, 5 top
    fn cell_edge<'a>(&self, x: usize, y: usize, edge: HexDir, out: &'a mut [u8])
        -> Option<(&'a str, HexEdge)>
    {
        let w = self.width();
        let h = self.height();
        if x >= w || y >= h { return None; }

        use std::io::Write;
        let mut cur = std::io::Cursor::new(out);
        match write!(cur, "{:?}", edge) {
            Ok(_)  => {
                let len = cur.position() as usize;
                Some((
                    std::str::from_utf8(&(cur.into_inner())[0..len])
                    .unwrap(),
                    HexEdge::ArrowValue { value: (1.0, 1.0) },
                ))
            },
            Err(_) => None,
        }
    }

    fn cell_click(&mut self, x: usize, y: usize, btn: MButton) {
        self.last_click = (x, y);
        println!("CLICK! {:?} => {},{}", btn, x, y);
    }
    fn cell_drag(&mut self, x: usize, y: usize, x2: usize, y2: usize, btn: MButton) {
        println!("DRAG! {:?} {},{} => {},{}", btn, x, y, x2, y2);
        self.drag_to = (x2, y2);
    }
}

#[derive(Lens)]
pub struct UIState {
    grid_1: Rc<RefCell<dyn HexGridModel>>,
    grid_2: Rc<RefCell<dyn HexGridModel>>,
}

impl Model for UIState {
}

fn main() {
    let mut app =
        Application::new(
            WindowDescription::new(),
            |state, window| {
                let ui_state =
                    UIState {
                        grid_1: Rc::new(RefCell::new(TestGridModel::new())),
                        grid_2: Rc::new(RefCell::new(TestGridModel::new())),
                    };

                let app_data = ui_state.build(state, window);

                let row = Row::new().build(state, app_data, |builder| builder);

                let hex =
                    HexGrid::new(1, 64.0)
                        .bind(UIState::grid_1, |value| value.clone())
                        .build(state, row, |builder| builder);
                let hex2 =
                    HexGrid::new(2, 72.0)
                        .bind(UIState::grid_2, |value| value.clone())
                        .build(state, row, |builder| builder);
            });
    app.run();
}
