use hexotk::widgets::hexgrid::HexGridModel;
use hexotk::{MButton, ActiveZone, UIPos, ParamID};
use hexotk::{Rect, WidgetUI, Painter, WidgetData, WidgetType, UIEvent};
use hexotk::constants::*;
use hexotk::widgets::{HexGrid, HexGridData};

use std::rc::Rc;
use std::cell::RefCell;

use crate::matrix::*;
use std::sync::Arc;
use std::sync::Mutex;

use crate::dsp::{UICategory};

enum MenuItem {
    Category { lbl: &'static str, cat: UICategory },
}

impl MenuItem {
    pub fn as_str<'a>(&'a self) -> &'a str {
        match self {
            MenuItem::Category { lbl, .. } => lbl,
        }
    }
}

enum MenuMode {
    CategorySelect,
}

pub struct MatrixUIMenu {
    matrix: Arc<Mutex<Matrix>>,
    items:  RefCell<Vec<MenuItem>>,
    mode:   RefCell<MenuMode>,
}

impl MatrixUIMenu {
    pub fn new(matrix: Arc<Mutex<Matrix>>) -> Self {
        Self {
            matrix,
            items:  RefCell::new(vec![]),
            mode:   RefCell::new(MenuMode::CategorySelect),
        }
    }

    pub fn set_category_mode(&self) {
        (*self.mode.borrow_mut()) = MenuMode::CategorySelect;
        let mut items = self.items.borrow_mut();
        items.clear();
        items.push(MenuItem::Category {
            lbl: "Osc",
            cat: UICategory::Oscillators
        });
        items.push(MenuItem::Category {
            lbl: "X->Y",
            cat: UICategory::XtoY,
        });
        items.push(MenuItem::Category {
            lbl: "Time",
            cat: UICategory::Time,
        });
        items.push(MenuItem::Category {
            lbl: "N->M",
            cat: UICategory::NtoM,
        });
        items.push(MenuItem::Category {
            lbl: "I/O",
            cat: UICategory::IOUtil,
        });
    }
}

impl HexGridModel for MatrixUIMenu {
    fn width(&self)  -> usize { 3 }
    fn height(&self) -> usize { 3 }

    fn cell_click(&self, x: usize, y: usize, btn: MButton) {
        println!("MENU CLICK CELL: {},{}: {:?}", x, y, btn);
    }

    fn cell_empty(&self, x: usize, y: usize) -> bool {
        if x >= 3 || y >= 3 { return true; }
        false
    }

    fn cell_visible(&self, x: usize, y: usize) -> bool {
        if x >= 3 || y >= 3 { return false; }
        if x == 0 && y == 0 || x == 2 && y == 0 { return false; }
        true
    }

    fn cell_label<'a>(&self, x: usize, y: usize, mut buf: &'a mut [u8]) -> Option<&'a str> {
        if x >= 3 || y >= 3 { return None; }
        let items = self.items.borrow_mut();
        let lbl =
            match (x, y) {
                (0, 0) => Some(items.get(6)?.as_str()),
                (1, 0) => Some(items.get(0)?.as_str()),
                (2, 0) => Some(items.get(1)?.as_str()),

                (0, 1) => Some(items.get(1)?.as_str()),
                (1, 1) => Some("Next>"),
                (2, 1) => Some(items.get(3)?.as_str()),

                (0, 2) => Some(items.get(5)?.as_str()),
                (1, 2) => Some("<Back"),
                (2, 2) => Some(items.get(4)?.as_str()),
                _      => None,
            };

        if let Some(lbl) = lbl {
            let len = buf.len().min(lbl.as_bytes().len());
            buf[0..len].copy_from_slice(&lbl.as_bytes()[0..len]);

            if let Ok(s) = std::str::from_utf8(&buf[0..len]) {
                Some(s)
            } else {
                None
            }

        } else {
            None
        }
    }

    fn cell_edge<'a>(&self, x: usize, y: usize, edge: u8, out: &'a mut [u8]) -> Option<&'a str> {
        None
    }
}

pub struct MatrixUIModel {
    matrix: Arc<Mutex<Matrix>>,
    menu:   Rc<MatrixUIMenu>,
    w:      usize,
    h:      usize,
}

impl HexGridModel for MatrixUIModel {
    fn width(&self) -> usize { self.w }
    fn height(&self) -> usize { self.h }

    fn cell_click(&self, x: usize, y: usize, btn: MButton) {
        println!("MENU CLICK CELL: {},{}: {:?}", x, y, btn);
    }

    fn cell_empty(&self, x: usize, y: usize) -> bool {
        if x >= self.w || y >= self.h { return true; }
        false
    }

    fn cell_visible(&self, x: usize, y: usize) -> bool {
        if x >= self.w || y >= self.h { return false; }
        true
    }

    fn cell_label<'a>(&self, x: usize, y: usize, buf: &'a mut [u8]) -> Option<&'a str> {
        if x >= self.w || y >= self.h { return None; }
        let m = self.matrix.lock().unwrap();
        if let Some(cell) = m.get(x, y) {
            Some(cell.label(buf)?)
        } else {
            None
        }
    }

    fn cell_edge<'a>(&self, x: usize, y: usize, edge: u8, out: &'a mut [u8]) -> Option<&'a str> {
        None
    }
}

pub struct NodeMatrixData {
    hex_grid:     Box<WidgetData>,
    hex_menu:     Box<WidgetData>,

    matrix_model: Rc<MatrixUIModel>,

    display_menu: Option<(f64, f64)>,
}

impl NodeMatrixData {
    pub fn new(matrix: Arc<Mutex<Matrix>>, pos: UIPos, node_id: u32) -> WidgetData {
        let wt_nmatrix  = Rc::new(NodeMatrix::new());

        let size = {
            let m = matrix.lock().unwrap();
            m.size()
        };

        let menu_model   = Rc::new(MatrixUIMenu::new(matrix.clone()));
        let matrix_model = Rc::new(MatrixUIModel {
            matrix,
            menu: menu_model.clone(),
            w: size.0,
            h: size.1,
        });

        let wt_hexgrid =
            Rc::new(HexGrid::new(14.0, 10.0));
        let wt_hexgrid_menu =
            Rc::new(HexGrid::new_y_offs(14.0, 10.0).bg_color(UI_GRID_BG2_CLR));

        WidgetData::new(
            wt_nmatrix,
            ParamID::new(node_id, 1),
            pos,
            Box::new(Self {
                hex_grid: WidgetData::new_tl_box(
                    wt_hexgrid.clone(),
                    ParamID::new(node_id, 1),
                    HexGridData::new(matrix_model.clone())),
                hex_menu: WidgetData::new_tl_box(
                    wt_hexgrid_menu.clone(),
                    ParamID::new(node_id, 2),
                    HexGridData::new(menu_model)),
                matrix_model,
                display_menu: None,
            }))
    }
}

#[derive(Debug, Clone)]
pub struct NodeMatrix {
}

impl NodeMatrix {
    pub fn new() -> Self {
        Self { }
    }
}

impl WidgetType for NodeMatrix {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        data.with(|data: &mut NodeMatrixData| {
            (*data.hex_grid).draw(ui, p, pos);

            if let Some(mouse_pos) = data.display_menu {
                let menu_w = 270.0;
                let menu_h = 280.0;

                let menu_rect =
                    Rect::from(
                        mouse_pos.0 - menu_w * 0.5,
                        mouse_pos.1 - menu_h * 0.5,
                        menu_w,
                        menu_h)
                    .move_into(&pos);

                (*data.hex_menu).draw(ui, p, menu_rect);
            }
        });
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        match ev {
            UIEvent::Click { id, button, .. } => {
                println!("EV: {:?} id={}, data.id={}", ev, *id, data.id());
                if id.node_id() == data.id().node_id() {
                    data.with(|data: &mut NodeMatrixData| {
                        if let Some(_) = data.display_menu {
                            data.hex_menu.event(ui, ev);
                            data.display_menu = None;
                        } else {
                            match ev {
                                UIEvent::Click { x, y, .. } => {
                                    data.matrix_model.menu.set_category_mode();
                                    data.display_menu = Some((*x, *y));
                                },
                                _ => {}
                            }
                        }
                    });
                    ui.queue_redraw();
                }
            },
            _ => {
                println!("EV: {:?}", ev);
            },
        }
    }
}
