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
mod grid_models;

mod jack;
mod synth;

use painter::FemtovgPainter;
use hexgrid::{HexGrid, HexGridModel, HexCell, HexDir, HexEdge, HexHLight};
use hexknob::{HexKnob, ParamModel};
use pattern_editor::PatternEditor;
use hexo_consts::*;

use hexodsp::{Matrix, NodeId, Cell, CellDir};
use hexodsp::matrix::MatrixError;

use std::rc::Rc;
use std::cell::RefCell;

use std::sync::{Arc, Mutex};

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
        "hexgrid:set_model" => {
            if let Some(model) = vval2hex_grid_model(event.v_(1)) {
                Event::new(hexgrid::HexGridMessage::SetModel(model))
            } else {
                eprintln!("Bad Event Type sent: {}, bad model arg!", event.s());
                Event::new(WindowEvent::Redraw)
            }
        },
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

fn output2vval(node_id: NodeId, out: u8) -> VVal {
    if let Some(name) = node_id.out_name_by_idx(out) {
        VVal::new_str(name)
    } else {
        VVal::Int(out as i64)
    }
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
            output2vval(node_id, i)
        }
    } else {
        VVal::None
    }
}

fn cell_set_port(cell: &mut Cell, v: VVal, dir: CellDir) -> bool {
    if v.is_none() {
        return true;
    }
    let name = v.s_raw();
    let node_id = cell.node_id();

    if dir.is_input() {
        if let Some(i) = node_id.inp(&name) {
            cell.set_io_dir(dir, i as usize);
            true
        } else {
            false
        }
    } else {
        if let Some(i) = node_id.out(&name) {
            cell.set_io_dir(dir, i as usize);
            true
        } else {
            false
        }
    }
}

fn vval2node_id(v: &VVal) -> NodeId {
    let node_id = v.v_(0).with_s_ref(|s| NodeId::from_str(s));
    node_id.to_instance(v.v_i(1) as usize)
}

fn cell2vval(cell: &Cell) -> VVal {
    let node_id = cell.node_id();

    let ports = VVal::vec();
    ports.push(cell_port2vval(cell, CellDir::T));
    ports.push(cell_port2vval(cell, CellDir::TL));
    ports.push(cell_port2vval(cell, CellDir::BL));
    ports.push(cell_port2vval(cell, CellDir::TR));
    ports.push(cell_port2vval(cell, CellDir::BR));
    ports.push(cell_port2vval(cell, CellDir::B));

    VVal::map3(
        "node_id",
        VVal::pair(
            VVal::new_str(node_id.name()),
            VVal::Int(node_id.instance() as i64)),
        "pos",
        VVal::ivec2(
            cell.pos().0 as i64,
            cell.pos().1 as i64),
        "ports", ports)
}

fn vval2cell(v: &VVal) -> Cell {
    let node_id = vval2node_id(&v.v_k("node_id"));

    let mut m_cell = Cell::empty(node_id);

    let ports = v.v_k("ports");

    cell_set_port(&mut m_cell, ports.v_(0), CellDir::T);
    cell_set_port(&mut m_cell, ports.v_(1), CellDir::TL);
    cell_set_port(&mut m_cell, ports.v_(2), CellDir::BL);
    cell_set_port(&mut m_cell, ports.v_(3), CellDir::TR);
    cell_set_port(&mut m_cell, ports.v_(4), CellDir::BR);
    cell_set_port(&mut m_cell, ports.v_(5), CellDir::B);

    m_cell
}

fn matrix_error2vval_err(err: MatrixError) -> VVal {
    let err_val =
        match err {
            MatrixError::CycleDetected => VVal::new_sym("cycle-detected"),
            MatrixError::PosOutOfRange => VVal::new_sym("pos-out-of-range"),
            MatrixError::NonEmptyCell { cell } =>
                VVal::pair(
                    VVal::new_sym("non-empty-cell"),
                    cell2vval(&cell)),
            MatrixError::DuplicatedInput { output1, output2 } =>
                VVal::vec3(
                    VVal::new_sym("duplicated-input"),
                    output2vval(output1.0, output1.1),
                    output2vval(output2.0, output2.1)),
        };

    VVal::Err(Rc::new(RefCell::new((
        err_val,
        wlambda::vval::SynPos::empty()))))
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

        if let Ok(mut m) = m {
            match key {
                "get" => {
                    if args.len() != 1 {
                        return Err(StackAction::panic_msg(
                            "matrix.get[$i(x, y)] called with wrong number of arguments"
                            .to_string()));
                    }

                    if let Some(cell) =
                        m.get(
                            env.arg(0).v_i(0) as usize,
                            env.arg(0).v_i(1) as usize)
                    {
                        Ok(cell2vval(cell))
                    } else {
                        Ok(VVal::None)
                    }
                },
                "set" => {
                    if args.len() != 2 {
                        return Err(StackAction::panic_msg(
                            "matrix.set[$i(x, y), cell] called with wrong number of arguments"
                            .to_string()));
                    }

                    if let (Some(vv_cell), Some(pos)) = (env.arg_ref(1), env.arg_ref(0)) {
                        let cell = vval2cell(vv_cell);

                        let x = pos.v_ik("x") as usize;
                        let y = pos.v_ik("y") as usize;

                        m.place(x, y, cell);

                        Ok(VVal::Bol(true))
                    } else {
                        Ok(VVal::None)
                    }
                },
                "restore_snapshot" => {
                    m.restore_matrix();
                    Ok(VVal::Bol(true))
                },
                "save_snapshot" => {
                    m.save_matrix();
                    Ok(VVal::Bol(true))
                },
                "check" => {
                    if args.len() != 0 {
                        return Err(StackAction::panic_msg(
                            "matrix.check[] called with wrong number of arguments"
                            .to_string()));
                    }

                    match m.check() {
                        Ok(_)  => Ok(VVal::Bol(true)),
                        Err(e) => Ok(matrix_error2vval_err(e)),
                    }
                },
                "sync" => {
                    if args.len() != 0 {
                        return Err(StackAction::panic_msg(
                            "matrix.check[] called with wrong number of arguments"
                            .to_string()));
                    }

                    match m.sync() {
                        Ok(_)  => Ok(VVal::Bol(true)),
                        Err(e) => Ok(matrix_error2vval_err(e)),
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

#[derive(Clone)]
struct VValHexGridModel {
    model: Rc<RefCell<dyn HexGridModel>>,
}

impl VValUserData for VValHexGridModel {
    fn s(&self) -> String { format!("$<UI::HexGridModel>") }
    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

fn vval2hex_grid_model(mut v: VVal) -> Option<Rc<RefCell<dyn HexGridModel>>> {
    v.with_usr_ref(|model: &mut VValHexGridModel| model.model.clone())
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

fn setup_hx_module(matrix: Arc<Mutex<Matrix>>) -> wlambda::SymbolTable {
    let mut st = wlambda::SymbolTable::new();

    st.set(
        "hexo_consts_rs",
        VVal::new_str(std::include_str!("hexo_consts.rs")));

    st.fun(
        "get_main_matrix_handle", move |env: &mut Env, argc: usize| {
            Ok(VVal::new_usr(VValMatrix { matrix: matrix.clone() }))
        }, Some(0), Some(0), false);

    st.fun(
        "create_matrix_grid_model", |env: &mut Env, argc: usize| {

            if let Some(matrix) =
                env.arg(0).with_usr_ref(|model: &mut VValMatrix| model.matrix.clone())
            {
                Ok(VVal::new_usr(VValHexGridModel {
                    model:
                        Rc::new(RefCell::new(
                            grid_models::MatrixUIModel::new(matrix))),
                }))
            }
            else
            {
                Ok(VVal::err_msg("The passed argument is not a matrix object."))
            }
        }, Some(1), Some(1), false);

    st.fun(
        "create_test_hex_grid_model", |env: &mut Env, argc: usize| {
            Ok(VVal::new_usr(VValHexGridModel {
                model: Rc::new(RefCell::new(grid_models::TestGridModel::new())),
            }))
        }, Some(0), Some(0), false);

    st
}

fn main() {
    synth::start(|matrix| {
        let mut app =
            Application::new(
                WindowDescription::new(),
                |state, window| {
                    let (gui_rec, gui_rec_vval) = GUIActionRecorder::new_vval(matrix.clone());

                    let thing = (HiddenThingie { }).build(state, window, |builder| builder);

                    state.add_font_mem(
                        "font_mono",
                        std::include_bytes!("font_mono.ttf"));

                    state.set_default_font("font_mono");

                    let global_env = wlambda::GlobalEnv::new_default();
                    global_env.borrow_mut().set_module("hx", setup_hx_module(matrix));

                    let mut wl_ctx = wlambda::EvalContext::new(global_env);

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
