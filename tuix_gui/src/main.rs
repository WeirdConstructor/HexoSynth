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
mod pattern_editor;

mod jack;
mod synth;

use painter::FemtovgPainter;
use hexgrid::{HexGrid, HexGridModel, HexCell, HexDir, HexEdge, HexHLight};
use hexknob::{HexKnob, ParamModel};
use pattern_editor::PatternEditor;
use hexo_consts::*;

use hexodsp::{Matrix, Cell, CellDir};

use std::rc::Rc;
use std::cell::RefCell;

use std::sync::{Arc, Mutex};

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
    NewRow(i64, i64, Option<String>),
    NewCol(i64, i64, VVal),
    NewHexKnob(i64, i64, Option<String>),
    NewHexGrid(i64, i64, f32, VVal),
    NewPatternEditor(i64, i64, Option<String>),
    NewButton(i64, i64, Option<String>, String, VVal),
    EmitTo(i64, i64, VVal),
    SetText(i64, String),
    AddTheme(String),
    Redraw,
}

#[derive(Debug)]
enum GUIRef {
    None,
    Ent(Entity),
}

pub struct GUIActionRecorder {
    matrix:   Arc<Mutex<Matrix>>,
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

fn vv2event(event: &VVal) -> Event {
    match &event.v_s_raw(0)[..] {
        "textbox:set_value"
            => Event::new(TextboxEvent::SetValue(event.v_s_raw(1))),
        "hexgrid:set_test_model"
            => Event::new(hexgrid::HexGridMessage::SetModel(Rc::new(RefCell::new(TestGridModel::new())))),
        _ => {
            eprintln!("Unknown Event Type sent: {}", event.s());
            Event::new(WindowEvent::Redraw)
        },
    }
}

fn vv2class(class: VVal) -> Option<String> {
    if class.is_some() {
        Some(class.s_raw())
    } else {
        None
    }
}

fn vvbuilder<'a, T>(mut builder: Builder<'a, T>, build_attribs: &VVal) -> Builder<'a, T> {
    let mut attribs = vec![];

    build_attribs.for_each(|v| {
        let val = v.v_(1);
        let key = v.v_s(0);
        attribs.push((key, val));
    });

    for (k, v) in attribs {
        builder =
            match &k[..] {
                "class" => builder.class(&v.s_raw()),
                "position" =>
                    builder.set_position_type(
                        if &v.s_raw() == "self" { PositionType::SelfDirected }
                        else { PositionType::ParentDirected }),
                _       => builder,
            };
    }

    builder
}

fn cell_port2vval(cell: &Cell, dir: CellDir) -> VVal {
    let node_id = cell.node_id();

    if let Some(i) = cell.local_port_idx(dir) {
        if dir.is_input() {
            if let Some(param) = node_id.inp_param_by_idx(i as usize) {
                VVal::new_str(param.name())
            } else {
                VVal::Int(i as i64)
            }
        } else {
            if let Some(name) = node_id.out_name_by_idx(i) {
                VVal::new_str(name)
            } else {
                VVal::Int(i as i64)
            }
        }
    } else {
        VVal::None
    }
}

#[derive(Clone)]
struct VValMatrix {
    matrix: Arc<Mutex<hexodsp::Matrix>>,
}

impl vval::VValUserData for VValMatrix {
    fn s(&self) -> String { format!("$<HexoDSP::Matrix>") }

    fn get_key(&self, key: &str) -> Option<VVal> {
        None
    }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        let m = self.matrix.lock();
        if let Ok(m) = m {
            match key {
                "get" => {
                    if args.len() != 2 {
                        return Err(StackAction::panic_msg(
                            "matrix.get[x, y] called with too few arguments"
                            .to_string()));
                    }

                    if let Some(cell) =
                        m.get(
                            env.arg(0).i() as usize,
                            env.arg(1).i() as usize)
                    {

                        let ports = VVal::vec();
                        ports.push(cell_port2vval(cell, CellDir::T));
                        ports.push(cell_port2vval(cell, CellDir::TL));
                        ports.push(cell_port2vval(cell, CellDir::BL));
                        ports.push(cell_port2vval(cell, CellDir::TR));
                        ports.push(cell_port2vval(cell, CellDir::BR));
                        ports.push(cell_port2vval(cell, CellDir::B));

                        Ok(VVal::map3(
                            "pos", VVal::ivec2(cell.pos().0 as i64, cell.pos().1 as i64),
                            "node_id",
                                VVal::pair(
                                    VVal::new_str(cell.node_id().label()),
                                    VVal::Int(cell.node_id().instance() as i64)),
                            "ports", ports))
                    } else {
                        Ok(VVal::None)
                    }
                },
                _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
            }
        } else {
             Ok(VVal::err_msg("Can't lock matrix!"))
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

impl GUIActionRecorder {
    pub fn new_vval(matrix: Arc<Mutex<Matrix>>) -> (Rc<RefCell<GUIActionRecorder>>, VVal) {
        let obj = VVal::map();

        let r =
            Rc::new(RefCell::new(
                GUIActionRecorder {
                    matrix: matrix.clone(),
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

        set_vval_method!(obj, r, emit_to, Some(3), Some(3), env, _argc, {
            r.borrow_mut().actions.push(
                GUIAction::EmitTo(
                    env.arg(0).i(), env.arg(1).i(), env.arg(2)));
            Ok(VVal::None)
        });

        set_vval_method!(obj, r, add_theme, Some(1), Some(1), env, _argc, {
            r.borrow_mut().actions.push(
                GUIAction::AddTheme(env.arg(0).s_raw()));
            Ok(VVal::None)
        });

        set_vval_method!(obj, r, new_row, Some(1), Some(2), env, _argc, {
            let mut r = r.borrow_mut();
            Ok(VVal::Int(r.new_row(env.arg(0).i(), env.arg(1))))
        });

        set_vval_method!(obj, r, new_col, Some(1), Some(2), env, _argc, {
            Ok(VVal::Int(r.borrow_mut().add(|id|
                GUIAction::NewCol(env.arg(0).i(), id, env.arg(1)))))
        });

        set_vval_method!(obj, r, new_hexknob, Some(1), Some(2), env, _argc, {
            let mut r = r.borrow_mut();
            Ok(VVal::Int(r.new_hexknob(env.arg(0).i(), env.arg(1))))
        });

        set_vval_method!(obj, r, new_hexgrid, Some(2), Some(3), env, _argc, {
            Ok(VVal::Int(
                r.borrow_mut().add(|id|
                    GUIAction::NewHexGrid(
                        env.arg(0).i(), id, env.arg(1).f() as f32, env.arg(2)))))
        });

        set_vval_method!(obj, r, new_pattern_editor, Some(1), Some(2), env, _argc, {
            let mut r = r.borrow_mut();
            Ok(VVal::Int(r.new_pattern_editor(env.arg(0).i(), env.arg(1))))
        });

        let matrix = matrix.clone();
        set_vval_method!(obj, r, matrix, Some(0), Some(0), env, _argc, {
            Ok(VVal::new_usr(VValMatrix { matrix: matrix.clone() }))
        });

        set_vval_method!(obj, r, new_button, Some(3), Some(4), env, _argc, {
            let mut r = r.borrow_mut();
            Ok(VVal::Int(r.new_button(
                env.arg(0).i(),
                env.arg(1).s_raw(),
                env.arg(2),
                env.arg(3)
            )))
        });

        r.borrow_mut().obj = obj.clone();

        (r, obj)
    }

    pub fn add<F: FnOnce(i64) -> GUIAction>(&mut self, f: F) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(f(ret_ref));
        ret_ref
    }

    pub fn new_hexknob(&mut self, parent: i64, class: VVal) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(GUIAction::NewHexKnob(parent, ret_ref, vv2class(class)));
        ret_ref
    }

    pub fn new_pattern_editor(&mut self, parent: i64, class: VVal) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(GUIAction::NewPatternEditor(parent, ret_ref, vv2class(class)));
        ret_ref
    }

    pub fn new_button(&mut self, parent: i64, label: String, on_click: VVal, class: VVal) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(GUIAction::NewButton(parent, ret_ref, vv2class(class), label, on_click));
        ret_ref
    }

    pub fn new_row(&mut self, parent: i64, class: VVal) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(GUIAction::NewRow(parent, ret_ref, vv2class(class)));
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
                GUIAction::NewRow(parent, out, class) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        self.refs[*out as usize] = GUIRef::Ent(
                            Row::new().build(state, *parent, |builder| {
                                if let Some(class) = class {
                                    builder.class(class)
                                } else {
                                    builder
                                }
                            }));
                    }
                },
                GUIAction::NewCol(parent, out, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        self.refs[*out as usize] = GUIRef::Ent(
                            Column::new().build(
                                state, *parent,
                                |builder| vvbuilder(builder, build_attribs)));
                    }
                },
                GUIAction::NewHexGrid(parent, out, tile_size, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        self.refs[*out as usize] = GUIRef::Ent(
                            HexGrid::new(*tile_size).build(
                                state, *parent,
                                |builder| vvbuilder(builder, build_attribs)));
                    }

                },
                GUIAction::NewHexKnob(parent, out, class) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        self.refs[*out as usize] = GUIRef::Ent(
                            HexKnob::new().build(state, *parent, |builder| { builder }));
                    }
                },
                GUIAction::NewPatternEditor(parent, out, class) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        self.refs[*out as usize] = GUIRef::Ent(
                            PatternEditor::new(
                                hexodsp::dsp::tracker::MAX_COLS)
                            .build(state, *parent, |builder| { builder }));
                    }
                },
                GUIAction::NewButton(parent, out, class, label, on_click) => {
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
                                .build(state, *parent, |builder| { builder }));
                    }
                },
                GUIAction::AddTheme(theme) => {
                    state.add_theme(theme);
                    println!("ADDTHEME: {}", theme);
                },
                GUIAction::SetText(entity, text) => {
                    if let Some(GUIRef::Ent(entity)) = self.refs.get(*entity as usize) {
                        entity.set_text(state, text);
                    }
                },
                GUIAction::EmitTo(entity, to, event) => {
                    if let Some(GUIRef::Ent(entity)) = self.refs.get(*entity as usize) {
                        if let Some(GUIRef::Ent(to)) = self.refs.get(*to as usize) {
                            state.insert_event(
                                vv2event(event).target(*to).origin(*entity));
                        }
                    }
                },
                GUIAction::Redraw => {
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                            .target(Entity::root()));
                },
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

struct HiddenThingie;
impl Widget for HiddenThingie {
    type Ret = Entity;
    type Data = ();

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity
    }

    fn on_draw(&mut self, state: &mut State, entity: Entity, canvas: &mut Canvas) {
    }
}

fn main() {
    synth::start(|matrix| {
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

                    let (gui_rec, gui_rec_vval) = GUIActionRecorder::new_vval(matrix.clone());

                    let thing = (HiddenThingie { }).build(state, app_data, |builder| builder);

                    state.add_font_mem(
                        "font_mono",
                        std::include_bytes!("font_mono.ttf"));

                    state.set_default_font("font_mono");

                    let mut wl_ctx = EvalContext::new_default();

                    wl_ctx.set_global_var(
                        "hexo_consts_rs",
                        &VVal::new_str(std::include_str!("hexo_consts.rs")));

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

                    gui_rec.borrow_mut().run(gui_rec_self, wl_ctx, state, thing);
                });
        app.run();
    });
}
