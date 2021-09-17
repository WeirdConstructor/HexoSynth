// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use wlambda::*;
use tuix::*;

mod hexknob;
mod hexo_consts;
mod painter;
mod hexgrid;
mod rect;

use painter::FemtovgPainter;
use hexgrid::{HexGrid, HexGridModel, HexCell, HexDir, HexEdge, HexHLight};
use hexknob::{HexKnob, ParamModel};
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

#[derive(Debug)]
enum GUIAction {
    NewRow(i64, i64),
    NewHexKnob(i64, i64),
    NewButton(i64, i64, String, VVal),
    SetText(i64, String),
    Redraw,
}

#[derive(Debug)]
enum GUIRef {
    None,
    Ent(Entity),
}

pub struct GUIActionRecorder {
    actions:  Vec<GUIAction>,
    refs:     Vec<GUIRef>,
    ref_idx:  i64,
    obj:      VVal,
}


pub fn exec_cb(
    self_ref: Rc<RefCell<GUIActionRecorder>>,
    wl_ctx:   Rc<RefCell<EvalContext>>,
    state:    &mut State,
    entity:   Entity,
    callback: VVal)
{
    let gui_rec = self_ref.borrow().obj.clone();

    match wl_ctx.borrow_mut().call(&callback, &[gui_rec]) {
        Ok(_) => {},
        Err(e) => { panic!("Error in callback: {:?}", e); }
    }

    let sr = self_ref.clone();

    self_ref.borrow_mut().run(sr, wl_ctx, state, entity);
}

impl GUIActionRecorder {
    pub fn new_vval() -> (Rc<RefCell<GUIActionRecorder>>, VVal) {
        let obj = VVal::map();

        let r =
            Rc::new(RefCell::new(
                GUIActionRecorder {
                    actions:  vec![],
                    refs:     vec![],
                    ref_idx:  1,
                    obj:      VVal::None,
                }));

        set_vval_method!(obj, r, redraw, None, None, env, _argc, {
            r.borrow_mut().actions.push(GUIAction::Redraw);
            Ok(VVal::None)
        });

        set_vval_method!(obj, r, set_text, Some(2), Some(2), env, _argc, {
            r.borrow_mut().actions.push(
                GUIAction::SetText(
                    env.arg(0).i(), env.arg(1).s_raw()));
            Ok(VVal::None)
        });

        set_vval_method!(obj, r, new_row, Some(1), Some(1), env, _argc, {
            let mut r = r.borrow_mut();
            Ok(VVal::Int(r.new_row(env.arg(0).i())))
        });

        set_vval_method!(obj, r, new_hexknob, Some(1), Some(1), env, _argc, {
            let mut r = r.borrow_mut();
            Ok(VVal::Int(r.new_hexknob(env.arg(0).i())))
        });

        set_vval_method!(obj, r, new_button, Some(3), Some(3), env, _argc, {
            let mut r = r.borrow_mut();
            Ok(VVal::Int(r.new_button(
                env.arg(0).i(),
                env.arg(1).s_raw(),
                env.arg(2)
            )))
        });

        r.borrow_mut().obj = obj.clone();

        (r, obj)
    }

    pub fn new_hexknob(&mut self, parent: i64) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(GUIAction::NewHexKnob(parent, ret_ref));
        ret_ref
    }

    pub fn new_button(&mut self, parent: i64, label: String, on_click: VVal) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(GUIAction::NewButton(parent, ret_ref, label, on_click));
        ret_ref
    }

    pub fn new_row(&mut self, parent: i64) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(GUIAction::NewRow(parent, ret_ref));
        ret_ref
    }

    pub fn new_ref(&mut self) -> i64 {
        let idx = self.ref_idx;
        self.ref_idx += 1;
        while self.refs.len() <= (idx as usize) {
            self.refs.push(GUIRef::None);
        }
        idx
    }

    pub fn run(&mut self, self_ref: Rc<RefCell<GUIActionRecorder>>, wl_ctx: Rc<RefCell<EvalContext>>, state: &mut State, entity: Entity) {
        if self.refs.len() < 1 {
            self.refs.push(GUIRef::Ent(entity));
        } else {
            self.refs[0] = GUIRef::Ent(entity);
        }

        for act in self.actions.iter() {
            match act {
                GUIAction::NewRow(parent, out) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        self.refs[*out as usize] = GUIRef::Ent(
                            Row::new().build(state, *parent, |builder| builder));
                    }
                },
                GUIAction::NewButton(parent, out, label, on_click) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        let wl_ctx   = wl_ctx.clone();
                        let on_click = on_click.clone();
                        let sr       = self_ref.clone();

                        self.refs[*out as usize] = GUIRef::Ent(
                            Button::with_label(label)
                                .on_release(move |_, state, button| {
                                    exec_cb(
                                        sr.clone(), wl_ctx.clone(),
                                        state, button, on_click.clone());
                                })
                                .build(state, *parent, |builder| builder));
                    }
                },
                GUIAction::SetText(entity, text) => {
                    if let Some(GUIRef::Ent(entity)) = self.refs.get(*entity as usize) {
                        entity.set_text(state, text);
                    }
                },
                GUIAction::Redraw => {
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                            .target(Entity::root()));
                },
                _ => {},
            }
        }

        self.actions.clear();
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

                let (gui_rec, gui_rec_vval) = GUIActionRecorder::new_vval();

                let mut wl_ctx = EvalContext::new_default();

                match wl_ctx.eval_file("main.wl") {
                    Ok(_) => { },
                    Err(e) => { panic!("Error in main.wl: {:?}", e); }
                }

                let init_fun =
                    wl_ctx.get_global_var("init")
                       .expect("global 'init' function in main.wl defined");

                match wl_ctx.call(&init_fun, &[gui_rec_vval]) {
                    Ok(_) => {},
                    Err(e) => { panic!("Error in main.wl 'init': {:?}", e); }
                }

                let wl_ctx = Rc::new(RefCell::new(wl_ctx));

                let gui_rec_self = gui_rec.clone();

                gui_rec.borrow_mut().run(gui_rec_self, wl_ctx, state, app_data);

//                let row = Row::new().build(state, app_data, |builder| builder);
//
//                let hex =
//                    HexGrid::new(1, 64.0)
//                        .bind(UIState::grid_1, |value| value.clone())
//                        .build(state, row, |builder| builder);
//
//                let hknob =
//                    HexKnob::new()
//                    .build(state, row, |builder| {
//                        builder
//                            .set_min_left(Units::Pixels(100.0))
//                            .set_max_left(Units::Pixels(100.0))
//                    });
//                let hknob =
//                    HexKnob::new()
//                    .build(state, row, |builder| {
//                        builder
//                            .set_min_left(Units::Pixels(100.0))
//                            .set_max_left(Units::Pixels(100.0))
//                    });
//                let hknob =
//                    HexKnob::new()
//                    .build(state, row, |builder| {
//                        builder
//                            .set_min_left(Units::Pixels(100.0))
//                            .set_max_left(Units::Pixels(100.0))
//                    });
//
//                let hex2 =
//                    HexGrid::new(2, 72.0)
//                        .bind(UIState::grid_2, |value| value.clone())
//                        .build(state, row, |builder| builder);
            });
    app.run();
}
