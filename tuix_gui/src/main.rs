// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use wlambda::*;
use tuix::*;
use tuix::widgets::*;

mod hexknob;
mod hexo_consts;
mod painter;
mod hexgrid;
mod rect;
mod pattern_editor;
mod grid_models;
mod cluster;
mod matrix_param_model;
mod octave_keys;
mod cv_array;

mod jack;
mod synth;

use crate::matrix_param_model::KnobParam;
use crate::hexknob::DummyParamModel;

use painter::FemtovgPainter;
use hexgrid::{HexGrid, HexGridModel, HexCell, HexDir, HexEdge, HexHLight};
use hexknob::{HexKnob, ParamModel};
use pattern_editor::PatternEditor;
use octave_keys::OctaveKeys;
use cv_array::CvArray;
use hexo_consts::*;

use hexodsp::{Matrix, NodeId, NodeInfo, ParamId, Cell, CellDir};
use hexodsp::matrix::{MatrixObserver, MatrixError};
use hexodsp::dsp::{UICategory, SAtom};

use std::rc::Rc;
use std::cell::RefCell;

use std::sync::{Arc, Mutex};

#[derive(Debug)]
enum GUIAction {
    NewElem(i64, i64, VVal),
    NewRow(i64, i64, Option<String>),
    NewCol(i64, i64, VVal),
    NewHexKnob(i64, i64, VVal, VVal),
    NewHexGrid(i64, i64, VVal),
    NewTabs(Vec<(VVal, i64)>, i64, VVal),
    NewPopup(i64, VVal),
    NewPatternEditor(i64, i64, Option<String>),
    NewButton(i64, i64, String, VVal, VVal),
    NewOctaveKeys(i64, i64, VVal),
    NewCvArray(i64, i64, VVal),
    EmitTo(i64, i64, VVal),
    SetText(i64, String),
    AddTheme(String),
    RemoveAllChilds(i64),
    Remove(i64),
    Redraw,
}

#[derive(Debug)]
enum GUIRef {
    None,
    Ent(Entity),
}

pub struct GUIActionRecorder {
    actions:    Vec<GUIAction>,
    refs:       Vec<GUIRef>,
    free_refs:  Vec<i64>,
    ref_idx:    i64,
}


pub fn exec_cb(
    self_ref: Rc<RefCell<GUIActionRecorder>>,
    wl_ctx:   Rc<RefCell<EvalContext>>,
    state:    &mut State,
    entity:   Entity,
    callback: VVal,
    args:     &[VVal])
{
    match wl_ctx.borrow_mut().call(&callback, args) {
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
        "popup:open_at_cursor"
            => Event::new(PopupEvent::OpenAtCursor),
        "popup:close"
            => Event::new(PopupEvent::Close),
        "octave_keys:set_mask"
            => Event::new(
                octave_keys::OctaveKeysMessage::SetMask(event.v_i(1))),
        "cv_array:set_array" => {
            if let Some(ar) = vv2sample_buf(event.v_(1)) {
                Event::new(cv_array::CvArrayMessage::SetArray(ar.clone()))
            } else {
                eprintln!("Bad Event Type sent: {}, bad array arg!", event.s());
                Event::new(WindowEvent::Redraw)
            }
        },
        "hexknob:set_model" => {
            if let Some(model) = vv2hex_knob_model(event.v_(1)) {
                Event::new(hexknob::HexKnobMessage::SetModel(model))
            } else {
                eprintln!("Bad Event Type sent: {}, bad model arg!", event.s());
                Event::new(WindowEvent::Redraw)
            }
        },
        "hexgrid:set_model" => {
            if let Some(model) = vv2hex_grid_model(event.v_(1)) {
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

fn vv2units(v: &VVal) -> Units {
    let amt = v.v_f(0) as f32;
    v.v_with_s_ref(1, |s|
        match s {
            "px"       => Units::Pixels(amt),
            "%"        => Units::Percentage(amt),
            "s"        => Units::Stretch(amt),
            "auto" | _ => Units::Auto,
        })
}

fn vvbuilder<'a, T>(mut builder: Builder<'a, T>, build_attribs: &VVal) -> Builder<'a, T> {
    let mut attribs = vec![];

    println!("VVB: {}", build_attribs.s());

    build_attribs.for_eachk(|key, val| {
        attribs.push((key.to_string(), val.clone()));
    });

    for (k, v) in attribs {
        builder =
            match &k[..] {
                "class" => {
                    if v.is_vec() {
                        for i in 0..v.len() {
                            builder = builder.class(&v.v_s_raw(i));
                        }
                        builder
                    } else {
                        builder.class(&v.s_raw())
                    }
                },
                "space"       => { builder.set_space(vv2units(&v)) },
                "child_space" => { builder.set_child_space(vv2units(&v)) },
                "text"        => { builder.set_text(&v.s_raw()) },
                "row"         => { builder.set_row_index(v.i() as usize) },
                "col"         => { builder.set_col_index(v.i() as usize) },
                "row_span"    => { builder.set_row_span(v.i() as usize) },
                "col_span"    => { builder.set_col_span(v.i() as usize) },
                "row_between" => { builder.set_row_between(vv2units(&v)) },
                "col_between" => { builder.set_col_between(vv2units(&v)) },
                "z_order"     => { builder.set_z_order(v.i() as i32) },
                "width"       => { builder.set_width(vv2units(&v)) },
                "height"      => { builder.set_height(vv2units(&v)) },
                "grid_rows" => {
                    let mut rows = vec![];
                    v.for_each(|v| { rows.push(vv2units(v)); });
                    builder.set_grid_rows(rows)
                },
                "grid_cols" => {
                    let mut cols = vec![];
                    v.for_each(|v| { cols.push(vv2units(v)); });
                    builder.set_grid_cols(cols)
                },
                "layout_type" => {
                    builder.set_layout_type(
                        if &v.s_raw() == "row" {
                            LayoutType::Row
                        } else if &v.s_raw() == "grid" {
                            LayoutType::Grid
                        } else if &v.s_raw() == "column" {
                            LayoutType::Column
                        } else {
                            LayoutType::Column
                        })
                },
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

fn vv2node_id(v: &VVal) -> NodeId {
    let node_id = v.v_(0).with_s_ref(|s| NodeId::from_str(s));
    node_id.to_instance(v.v_i(1) as usize)
}

fn node_id2vv(nid: NodeId) -> VVal {
    VVal::pair(VVal::new_str(nid.name()), VVal::Int(nid.instance() as i64))
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
        node_id2vv(node_id),
        "pos",
        VVal::ivec2(
            cell.pos().0 as i64,
            cell.pos().1 as i64),
        "ports", ports)
}

fn vv2cell(v: &VVal) -> Cell {
    let node_id = vv2node_id(&v.v_k("node_id"));

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

macro_rules! arg_chk {
    ($args: expr, $count: expr, $name: literal) => {
        if $args.len() != $count {
            return Err(StackAction::panic_msg(format!(
                "{} called with wrong number of arguments",
                $name)));
        }
    }
}

macro_rules! wl_panic {
    ($str: literal) => {
        return Err(StackAction::panic_msg($str.to_string()));
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

        match key {
            "create_grid_model" => {
                arg_chk!(args, 0, "matrix.create_grid_model[]");

                let matrix = self.matrix.clone();

                return Ok(VVal::new_usr(VValHexGridModel {
                    model:
                        HexGridModelType::Matrix(
                            Rc::new(RefCell::new(
                                grid_models::MatrixUIModel::new(matrix)))),
                }));
            },
            "create_hex_knob_dummy_model" => {
                arg_chk!(args, 0, "matrix.create_hex_knob_dummy_model[]");

                return Ok(VVal::new_usr(VValHexKnobModel {
                    model: Rc::new(RefCell::new(DummyParamModel::new()))
                }));
            },
            "create_hex_knob_model" => {
                arg_chk!(args, 1, "matrix.create_hex_knob_model[param_id]");

                let matrix = self.matrix.clone();
                if let Some(param_id) = vv2param_id(env.arg(0)) {
                    return Ok(VVal::new_usr(VValHexKnobModel {
                        model: Rc::new(RefCell::new(
                            KnobParam::new(matrix, param_id)))
                    }));

                } else {
                    wl_panic!(
                        "matrix.create_hex_knob_model[param_id] requires \
                        a $<HexoDSP::ParamId> as first argument.");
                }
            },
            _ => {}
        }

        let m = self.matrix.lock();

        if let Ok(mut m) = m {
            match key {
                "get" => {
                    arg_chk!(args, 1, "matrix.get[$i(x, y)]");

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
                    arg_chk!(args, 2, "matrix.set[$i(x, y), cell]");

                    if let (Some(vv_cell), Some(pos)) = (env.arg_ref(1), env.arg_ref(0)) {
                        let cell = vv2cell(vv_cell);

                        let x = pos.v_ik("x") as usize;
                        let y = pos.v_ik("y") as usize;

                        m.place(x, y, cell);

                        Ok(VVal::Bol(true))
                    } else {
                        Ok(VVal::None)
                    }
                },
                "restore_snapshot" => {
                    arg_chk!(args, 0, "matrix.restore_snapshot[]");
                    m.restore_matrix();
                    Ok(VVal::Bol(true))
                },
                "save_snapshot" => {
                    arg_chk!(args, 0, "matrix.save_snapshot[]");
                    m.save_matrix();
                    Ok(VVal::Bol(true))
                },
                "check" => {
                    arg_chk!(args, 0, "matrix.check[]");

                    match m.check() {
                        Ok(_)  => Ok(VVal::Bol(true)),
                        Err(e) => Ok(matrix_error2vval_err(e)),
                    }
                },
                "sync" => {
                    arg_chk!(args, 0, "matrix.sync[]");

                    match m.sync() {
                        Ok(_)  => Ok(VVal::Bol(true)),
                        Err(e) => Ok(matrix_error2vval_err(e)),
                    }
                },
                "get_unused_instance_node_id" => {
                    arg_chk!(args, 1, "matrix.get_unused_instance_node_id[node_id]");

                    let node_id = vv2node_id(&args[0]);
                    let node_id = m.get_unused_instance_node_id(node_id);
                    Ok(node_id2vv(node_id))
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
struct VValCellDir {
    dir: CellDir
}

pub fn vv2cell_dir(v: &VVal) -> CellDir {
    v.with_s_ref(|s| {
        match s {
            "c" | "C" => CellDir::C,
            "t" | "T" => CellDir::T,
            "b" | "B" => CellDir::B,
            "tl" | "TL" => CellDir::TL,
            "bl" | "BL" => CellDir::BL,
            "tr" | "TR" => CellDir::TR,
            "br" | "BR" => CellDir::BR,
            _ => CellDir::C,
        }
    })
}

impl VValCellDir {
    pub fn from_vval(v: &VVal) -> Self {
        Self { dir: vv2cell_dir(v), }
    }

    pub fn from_vval_edge(v: &VVal) -> Self {
        Self {
            dir: CellDir::from(v.i() as u8),
        }
    }
}

fn cell_dir2vv(dir: CellDir) -> VVal { VVal::new_usr(VValCellDir { dir }) }

impl vval::VValUserData for VValCellDir {
    fn s(&self) -> String { format!("$<CellDir::{:?}>", self.dir) }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
            "as_edge" => {
                arg_chk!(args, 0, "cell_dir.as_edge[]");

                Ok(VVal::Int(self.dir.as_edge() as i64))
            },
            "flip" => {
                arg_chk!(args, 0, "cell_dir.flip[]");

                Ok(cell_dir2vv(self.dir.flip()))
            },
            "is_input" => {
                arg_chk!(args, 0, "cell_dir.is_input[]");

                Ok(VVal::Bol(self.dir.is_input()))
            },
            "is_output" => {
                arg_chk!(args, 0, "cell_dir.is_output[]");

                Ok(VVal::Bol(self.dir.is_output()))
            },
            "offs_pos" => {
                arg_chk!(args, 1, "cell_dir.offs_pos[$i(x, y)]");

                let p = env.arg(0);

                let pos = (
                    p.v_i(0) as usize,
                    p.v_i(1) as usize
                );

                if let Some(opos) = self.dir.offs_pos(pos) {
                    Ok(VVal::ivec2(opos.0 as i64, opos.1 as i64))
                } else {
                    Ok(VVal::None)
                }
            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

#[derive(Clone)]
struct VValCluster {
    cluster: Rc<RefCell<crate::cluster::Cluster>>,
}

impl VValCluster {
    pub fn new() -> Self {
        Self {
            cluster: Rc::new(RefCell::new(crate::cluster::Cluster::new())),
        }
    }
}

impl vval::VValUserData for VValCluster {
    fn s(&self) -> String { format!("$<HexoDSP::Cluster>") }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
            "add_cluster_at" => {
                arg_chk!(args, 2, "cluster.add_cluster_at[matrix, $i(x, y)]");

                let mut m = env.arg(0);

                if let Some(matrix) =
                    m.with_usr_ref(|m: &mut VValMatrix| { m.matrix.clone() })
                {
                    if let Ok(mut m) = matrix.lock() {
                        let v = env.arg(1);

                        self.cluster
                            .borrow_mut()
                            .add_cluster_at(
                                &mut m,
                                (v.v_i(0) as usize,
                                 v.v_i(1) as usize));
                    }
                }

                Ok(VVal::None)
            },
            "ignore_pos" => {
                arg_chk!(args, 1, "cluster.ignore_pos[$i(x, y)]");

                let v = env.arg(0);

                self.cluster.borrow_mut().ignore_pos((
                    v.v_i(0) as usize,
                    v.v_i(1) as usize));

                Ok(VVal::None)
            },
            "position_list" => {
                arg_chk!(args, 0, "cluster.position_list[]");

                let v = VVal::vec();

                self.cluster.borrow().for_poses(|pos| {
                    v.push(VVal::ivec2(pos.0 as i64, pos.1 as i64));
                });

                Ok(v)
            },
            "cell_list" => {
                arg_chk!(args, 0, "cluster.cell_list[]");

                let v = VVal::vec();

                self.cluster.borrow().for_cells(|cell| {
                    v.push(cell2vval(cell));
                });

                Ok(v)
            },
            "remove_cells" => {
                arg_chk!(args, 1, "cluster.remove_cells[matrix]");

                let mut m = env.arg(0);

                if let Some(matrix) =
                    m.with_usr_ref(|m: &mut VValMatrix| { m.matrix.clone() })
                {
                    if let Ok(mut m) = matrix.lock() {
                        self.cluster.borrow_mut().remove_cells(&mut m);
                    }
                }

                Ok(VVal::None)
            },
            "place" => {
                arg_chk!(args, 1, "cluster.place[matrix]");

                let mut m = env.arg(0);

                if let Some(matrix) =
                    m.with_usr_ref(|m: &mut VValMatrix| { m.matrix.clone() })
                {
                    if let Ok(mut m) = matrix.lock() {
                        return
                            match self.cluster.borrow_mut().place(&mut m) {
                                Ok(_) => Ok(VVal::Bol(true)),
                                Err(e) => Ok(matrix_error2vval_err(e)),
                            };
                    }
                }

                Ok(VVal::None)
            },
            "move_cluster_cells_dir_path" => {
                arg_chk!(args, 1, "cluster.move_cluster_cells_dir_path[$[CellDir, ...]]");

                let path = env.arg(0);
                let mut cd_path = vec![];

                path.for_each(|v| {
                    let mut v = v.clone();

                    if let Some(dir) =
                        v.with_usr_ref(|v: &mut VValCellDir| v.dir)
                    {
                        cd_path.push(dir);
                    } else {
                        cd_path.push(vv2cell_dir(&v));
                    }
                });

                match self.cluster
                          .borrow_mut()
                          .move_cluster_cells_dir_path(&cd_path[..])
                {
                    Ok(_)  => Ok(VVal::Bol(true)),
                    Err(e) => Ok(matrix_error2vval_err(e)),
                }
            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

#[derive(Clone)]
struct VValNodeInfo {
    node_id: NodeId,
    info:    Rc<NodeInfo>,
}

impl VValNodeInfo {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            info: Rc::new(NodeInfo::from_node_id(node_id)),
            node_id,
        }
    }
}

impl vval::VValUserData for VValNodeInfo {
    fn s(&self) -> String {
        format!(
            "$<HexoDSP::NodeInfo node={}, at_cnt={}, in_cnt={}, out_cnt={}>",
            self.node_id.name(),
            self.info.at_count(),
            self.info.in_count(),
            self.info.out_count())
    }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
            "add_cluster_at" => {
                arg_chk!(args, 2, "cluster.add_cluster_at[matrix, $i(x, y)]");
                Ok(VVal::None)
            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

#[derive(Clone)]
struct VValAtom { atom: SAtom }

impl VValAtom {
    pub fn new(atom: SAtom) -> Self {
        Self { atom }
    }
}

impl vval::VValUserData for VValAtom {
    fn s(&self) -> String {
        format!(
            "$<HexoDSP::SAtom type={}, i={}, f={:8.4}>",
            self.atom.type_str(),
            self.atom.i(),
            self.atom.f())
    }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
            "s" => {
                arg_chk!(args, 0, "atom.s[]");
                Ok(VVal::new_str_mv(self.atom.s()))
            },
            "i" => {
                arg_chk!(args, 0, "atom.i[]");
                Ok(VVal::Int(self.atom.i()))
            },
            "f" => {
                arg_chk!(args, 0, "atom.f[]");
                Ok(VVal::Flt(self.atom.f() as f64))
            },
            "micro_sample" => {
                arg_chk!(args, 0, "atom.micro_sample[]");

                if let SAtom::MicroSample(ms) = &self.atom {
                    let v = VVal::vec();
                    for s in ms.iter() {
                        v.push(VVal::Flt(*s as f64));
                    }

                    Ok(v)
                } else {
                    Ok(VVal::vec1(VVal::Flt(self.atom.f() as f64)))
                }
            },
            "default_of" => {
                arg_chk!(args, 0, "atom.default_of[]");

                Ok(VVal::Usr(Box::new(VValAtom {
                    atom: self.atom.default_of()
                })))
            },
            "is_continous" => {
                arg_chk!(args, 0, "atom.is_continous[]");

                Ok(VVal::Bol(self.atom.is_continous()))
            },
            "type_str" => {
                arg_chk!(args, 0, "atom.type_str[]");

                Ok(VVal::new_sym(self.atom.type_str()))
            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

fn vv2atom(mut v: VVal) -> Option<SAtom> {
    v.with_usr_ref(|model: &mut VValAtom| model.atom.clone())
}

#[derive(Clone)]
struct VValSampleBuf {
    buf: Arc<Mutex<Vec<f32>>>,
}

impl VValSampleBuf {
    pub fn from_vec(v: Vec<f32>) -> Self {
        Self {
            buf: Arc::new(Mutex::new(v)),
        }
    }
}

impl vval::VValUserData for VValSampleBuf {
    fn s(&self) -> String {
        let size = self.buf.lock().map_or(0, |guard| guard.len());
        format!("$<SampleBuf[{}]>", size)
    }

    fn set_key(&self, key: &VVal, val: VVal) -> Result<(), StackAction> {
        let idx = key.i() as usize;

        if let Ok(mut guard) = self.buf.lock() {
            if idx < guard.len() {
                guard[idx] = val.f() as f32;
            }
        }

        Ok(())
    }

    fn get_key(&self, key: &str) -> Option<VVal> {
        let idx = key.parse::<usize>().unwrap_or(0);
        let val =
            self.buf.lock().map_or(
                None,
                |guard| guard.get(idx).copied())?;

        Some(VVal::Flt(val as f64))
    }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
            "len" => {
                arg_chk!(args, 0, "sample_buf.len[]");

                let size = self.buf.lock().map_or(0, |guard| guard.len());
                Ok(VVal::Int(size as i64))
            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

fn vv2sample_buf(mut v: VVal) -> Option<Arc<Mutex<Vec<f32>>>> {
    v.with_usr_ref(|model: &mut VValSampleBuf| model.buf.clone())
}

fn sample_buf2vv(r: Arc<Mutex<Vec<f32>>>) -> VVal {
    VVal::new_usr(VValSampleBuf { buf: r })
}

#[derive(Clone)]
enum HexGridModelType {
    Test(Rc<RefCell<grid_models::TestGridModel>>),
    Matrix(Rc<RefCell<grid_models::MatrixUIModel>>),
}

#[derive(Clone)]
struct VValHexGridModel {
    model: HexGridModelType,
}

impl VValHexGridModel {
    fn as_hex_grid_model(&self) -> Rc<RefCell<dyn HexGridModel>> {
        match &self.model {
            HexGridModelType::Test(m)   => m.clone(),
            HexGridModelType::Matrix(m) => m.clone(),
        }
    }
}

impl VValUserData for VValHexGridModel {
    fn s(&self) -> String { format!("$<UI::HexGridModel>") }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
            "set_focus_cell" => {
                arg_chk!(args, 1, "hex_grid_model.set_focus_cell[$i(x, y)]");

                if let HexGridModelType::Matrix(m) = &self.model {
                    m.borrow_mut().set_focus_cell(
                        env.arg(0).v_i(0) as usize,
                        env.arg(0).v_i(1) as usize);
                }

                Ok(VVal::None)
            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

fn vv2hex_grid_model(mut v: VVal) -> Option<Rc<RefCell<dyn HexGridModel>>> {
    v.with_usr_ref(|model: &mut VValHexGridModel| model.as_hex_grid_model())
}

#[derive(Clone)]
struct VValHexKnobModel {
    model: Rc<RefCell<dyn ParamModel>>,
}

impl VValUserData for VValHexKnobModel {
    fn s(&self) -> String { format!("$<UI::HexKnobModel>") }
    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

fn vv2hex_knob_model(mut v: VVal) -> Option<Rc<RefCell<dyn ParamModel>>> {
    v.with_usr_ref(|model: &mut VValHexKnobModel| model.model.clone())
}

#[derive(Clone)]
struct VValParamId {
    param: ParamId,
}

impl VValUserData for VValParamId {
    fn s(&self) -> String {
        format!(
            "$<HexoDSP::ParamId node_id={}, idx={}, name={}>",
            self.param.node_id(),
            self.param.inp(),
            self.param.name())
    }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
            "as_parts" => {
                arg_chk!(args, 0, "param_id.as_parts[]");

                Ok(VVal::pair(
                    node_id2vv(self.param.node_id()),
                    VVal::Int(self.param.inp() as i64)))
            },
            "name" => {
                arg_chk!(args, 0, "param_id.name[]");

                Ok(VVal::new_str(self.param.name()))
            },
            "default_value" => {
                arg_chk!(args, 0, "param_id.as_parts[]");

                Ok(VVal::Usr(Box::new(VValAtom::new(self.param.as_atom_def()))))
            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

fn vv2param_id(mut v: VVal) -> Option<ParamId> {
    v.with_usr_ref(|s: &mut VValParamId| s.param.clone())
}

fn btn2vval(btn: tuix::MouseButton) -> VVal {
    match btn {
        tuix::MouseButton::Right    => VVal::new_sym("right"),
        tuix::MouseButton::Middle   => VVal::new_sym("middle"),
        tuix::MouseButton::Left     => VVal::new_sym("left"),
        tuix::MouseButton::Other(n) =>
            VVal::pair(VVal::new_sym("other"), VVal::Int(n as i64)),
    }
}

impl GUIActionRecorder {
    pub fn new() -> Rc<RefCell<GUIActionRecorder>> {
        Rc::new(RefCell::new(
            GUIActionRecorder {
                actions:    vec![],
                refs:       vec![],
                free_refs:  vec![],
                ref_idx:    1,
            }))
    }

    pub fn add<F: FnOnce(i64) -> GUIAction>(&mut self, f: F) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(f(ret_ref));
        ret_ref
    }

    pub fn new_pattern_editor(&mut self, parent: i64, class: VVal) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(GUIAction::NewPatternEditor(parent, ret_ref, vv2class(class)));
        ret_ref
    }

    pub fn new_button(&mut self, parent: i64, label: String, on_click: VVal, build_attribs: VVal) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(GUIAction::NewButton(parent, ret_ref, label, on_click, build_attribs));
        ret_ref
    }

    pub fn new_row(&mut self, parent: i64, class: VVal) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(GUIAction::NewRow(parent, ret_ref, vv2class(class)));
        ret_ref
    }

    pub fn new_ref(&mut self) -> i64 {
        if !self.free_refs.is_empty() {
            if let Some(id) = self.free_refs.pop() {
                return id;
            }
        }

        let idx = self.ref_idx;
        self.ref_idx += 1;
        while self.refs.len() <= (idx as usize) {
            self.refs.push(GUIRef::None);
        }
        idx
    }

    pub fn set_root(&mut self, root: Entity) {
        if self.refs.len() < 1 {
            self.refs.push(GUIRef::Ent(root));
        } else {
            self.refs[0] = GUIRef::Ent(root);
        }
    }

    pub fn run(&mut self, self_ref: Rc<RefCell<GUIActionRecorder>>,
               wl_ctx: Rc<RefCell<EvalContext>>,
               state: &mut State, entity: Entity)
    {
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
                GUIAction::NewElem(parent, out, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        self.refs[*out as usize] = GUIRef::Ent(
                            Element::new().build(
                                state, *parent,
                                |builder| vvbuilder(builder, build_attribs)));
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
                GUIAction::NewHexGrid(parent, out, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        let on_click     = build_attribs.v_k("on_click");
                        let on_cell_drag = build_attribs.v_k("on_cell_drag");

                        let sr1 = self_ref.clone();
                        let sr2 = self_ref.clone();
                        let wl_ctx1 = wl_ctx.clone();
                        let wl_ctx2 = wl_ctx.clone();

                        self.refs[*out as usize] = GUIRef::Ent(
                            HexGrid::new()
                                .on_click(move |_, state, button, x, y, btn| {
                                    exec_cb(
                                        sr1.clone(), wl_ctx1.clone(),
                                        state, button, on_click.clone(),
                                        &[VVal::ivec2(x as i64, y as i64), btn2vval(btn)]);
                                })
                                .on_cell_drag(move |_, state, button, x1, y1, x2, y2, btn| {
                                    exec_cb(
                                        sr2.clone(), wl_ctx2.clone(),
                                        state, button, on_cell_drag.clone(),
                                        &[VVal::ivec2(x1 as i64, y1 as i64),
                                          VVal::ivec2(x2 as i64, y2 as i64),
                                          btn2vval(btn)]);
                                })
                                .build(state, *parent,
                                    |builder| vvbuilder(builder, build_attribs)));
                    }

                },
                GUIAction::NewHexKnob(parent, out, param_model, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        // XXX: Unwrap is checked by the
                        //      creator of GUIAction::NewHexKnob!
                        let param_model =
                            vv2hex_knob_model(param_model.clone()).unwrap();

                        self.refs[*out as usize] = GUIRef::Ent(
                            HexKnob::new(param_model)
                                .build(state, *parent,
                                    |builder| vvbuilder(builder, build_attribs)));
                    }
                },
                GUIAction::NewOctaveKeys(parent, out, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        let on_change = build_attribs.v_k("on_change");
                        let sr1       = self_ref.clone();
                        let wl_ctx1   = wl_ctx.clone();

                        self.refs[*out as usize] = GUIRef::Ent(
                            OctaveKeys::new()
                                .on_change(move |_, state, button, mask| {
                                    exec_cb(
                                        sr1.clone(), wl_ctx1.clone(),
                                        state, button, on_change.clone(),
                                        &[VVal::Int(mask)]);
                                })
                                .build(state, *parent,
                                    |builder| vvbuilder(builder, build_attribs)));
                    }
                },
                GUIAction::NewCvArray(parent, out, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        let binary    = build_attribs.v_k("binary");
                        let on_change = build_attribs.v_k("on_change");
                        let sr1       = self_ref.clone();
                        let wl_ctx1   = wl_ctx.clone();

                        self.refs[*out as usize] = GUIRef::Ent(
                            CvArray::new(binary.b())
                                .on_change(move |_, state, button, arr| {
                                    exec_cb(
                                        sr1.clone(), wl_ctx1.clone(),
                                        state, button, on_change.clone(),
                                        &[sample_buf2vv(arr.clone())]);
                                })
                                .build(state, *parent,
                                    |builder| vvbuilder(builder, build_attribs)));
                    }
                },
                GUIAction::NewPatternEditor(parent, out, class) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        self.refs[*out as usize] = GUIRef::Ent(
                            PatternEditor::new(
                                hexodsp::dsp::tracker::MAX_COLS)
                                .build(state, *parent, |builder| builder));
                    }
                },
                GUIAction::NewPopup(out, build_attribs) => {
                    self.refs[*out as usize] = GUIRef::Ent(
                        Popup::new()
                            .build(state, Entity::root(),
                                |builder|
                                    vvbuilder(builder, build_attribs)));
                },
                GUIAction::NewButton(parent, out, label, on_click, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        let wl_ctx   = wl_ctx.clone();
                        let on_click = on_click.clone();
                        let sr       = self_ref.clone();

                        self.refs[*out as usize] = GUIRef::Ent(
                            Button::with_label(label)
                                .on_release(move |_, state, button| {
                                    exec_cb(
                                        sr.clone(), wl_ctx.clone(),
                                        state, button, on_click.clone(),
                                        &[]);
                                })
                                .build(state, *parent,
                                    |builder|
                                        vvbuilder(builder, build_attribs)));
                    }
                },
                GUIAction::NewTabs(tabs, parent, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        let tab_build_at = build_attribs.v_k("tab");

                        let (tab_bar, tab_viewport) =
                            TabView::new().build(state, *parent,
                                |builder| vvbuilder(
                                    builder, &build_attribs.v_k("tab_view")));

                        for (i, (tab_battribs, tab_cont_id)) in tabs.iter().enumerate() {
                            let name   = tab_battribs.v_s_rawk("name");
                            let title  = tab_battribs.v_s_rawk("title");
                            let catrib = tab_battribs.v_k("cont");

                            let tab =
                                Tab::new(&name)
                                    .build(state, tab_bar, |builder| {
                                        vvbuilder(builder.set_text(&title), &tab_build_at)
                                    });

                            println!("CONT: {}", catrib.s());
                            let container =
                                TabContainer::new(&name)
                                    .build(state, tab_viewport,
                                        |builder| vvbuilder(builder, &catrib));

                            if i == 0 {
                                tab.set_checked(state, true);
                            } else {
                                container.set_display(state, Display::None);
                            }

                            self.refs[*tab_cont_id as usize] =
                                GUIRef::Ent(container);
                        }
                    }
                },
                GUIAction::AddTheme(theme) => {
                    state.add_theme(theme);
                    println!("ADDTHEME: {}", theme);
                },
                GUIAction::Remove(id) => {
                    if let Some(GUIRef::Ent(entity)) = self.refs.get(*id as usize) {
                        state.remove(*entity);
                    }

                    self.refs[*id as usize] = GUIRef::None;
                    self.free_refs.push(*id);
                },
                GUIAction::RemoveAllChilds(id) => {
                    let mut removed_entities = vec![];

                    if let Some(GUIRef::Ent(entity)) = self.refs.get(*id as usize) {
                        for i in 0..state.tree.get_num_children(*entity).unwrap_or(0) {
                            if let Some(child) =
                                state.tree.get_child(*entity, i as usize)
                            {
                                removed_entities.push(child);
                            }
                        }
                    }

                    let mut removed_ids : Vec<Entity> = vec![];
                    for dead_child in removed_entities {
                        let mut remove_idx = None;

                        state.remove(dead_child);

                        for (i, r) in self.refs.iter().enumerate() {
                            if let GUIRef::Ent(entity) = r {
                                if *entity == dead_child {
                                    remove_idx = Some(i);
                                }
                            }
                        }

                        if let Some(i) = remove_idx {
                            self.refs[i] = GUIRef::None;
                            self.free_refs.push(i as i64);
                        }
                    }
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

fn ui_category2str(cat: UICategory) -> &'static str {
    match cat {
        UICategory::None   => "none",
        UICategory::Osc    => "Osc",
        UICategory::Mod    => "Mod",
        UICategory::NtoM   => "NtoM",
        UICategory::Signal => "Signal",
        UICategory::Ctrl   => "Ctrl",
        UICategory::IOUtil => "IOUtil",
    }
}

fn setup_node_id_module() -> wlambda::SymbolTable {
    let mut st = wlambda::SymbolTable::new();

    st.fun(
        "list_all", move |env: &mut Env, argc: usize| {
            let ids = VVal::vec();

            for nid in hexodsp::dsp::ALL_NODE_IDS.iter() {
                ids.push(VVal::new_str(nid.name()));
            }

            Ok(ids)
        }, Some(0), Some(0), false);

    st.fun(
        "ui_category_list", move |env: &mut Env, argc: usize| {
            let cats = VVal::vec();
            cats.push(VVal::new_sym("none"));
            cats.push(VVal::new_sym("Osc"));
            cats.push(VVal::new_sym("Mod"));
            cats.push(VVal::new_sym("NtoM"));
            cats.push(VVal::new_sym("Signal"));
            cats.push(VVal::new_sym("Ctrl"));
            cats.push(VVal::new_sym("IOUtil"));
            Ok(cats)
        }, Some(0), Some(0), false);

    st.fun(
        "ui_category_node_id_map", move |env: &mut Env, argc: usize| {
            let m = VVal::map();

            for cat in [
                UICategory::Osc,
                UICategory::Mod,
                UICategory::NtoM,
                UICategory::Signal,
                UICategory::Ctrl,
                UICategory::IOUtil
            ]
            {
                let v = VVal::vec();
                cat.get_node_ids(0, |nid| { v.push(node_id2vv(nid)); });
                m.set_key_str(ui_category2str(cat), v);
            }

            Ok(m)
        }, Some(0), Some(0), false);

    st.fun(
        "ui_category", move |env: &mut Env, argc: usize| {
            let nid = vv2node_id(&env.arg(0));
            Ok(VVal::new_sym(ui_category2str(nid.ui_category())))
        }, Some(1), Some(1), false);

    st.fun(
        "instance", move |env: &mut Env, argc: usize| {
            Ok(VVal::Int(vv2node_id(&env.arg(0)).instance() as i64))
        }, Some(1), Some(1), false);

    st.fun(
        "name", move |env: &mut Env, argc: usize| {
            Ok(VVal::new_str(vv2node_id(&env.arg(0)).name()))
        }, Some(1), Some(1), false);

    st.fun(
        "label", move |env: &mut Env, argc: usize| {
            Ok(VVal::new_str(vv2node_id(&env.arg(0)).label()))
        }, Some(1), Some(1), false);

    let mut info_map : std::collections::HashMap<String, VVal> =
        std::collections::HashMap::new();

    for nid in hexodsp::dsp::ALL_NODE_IDS.iter() {
        info_map.insert(
            nid.name().to_string(),
            VVal::new_usr(VValNodeInfo::new(*nid)));
    }

    st.fun(
        "info", move |env: &mut Env, argc: usize| {
            let nid = vv2node_id(&env.arg(0));
            Ok(info_map.get(nid.name()).map_or(VVal::None, |v| v.clone()))
        }, Some(1), Some(1), false);

    st.fun(
        "eq_variant", move |env: &mut Env, argc: usize| {
            Ok(VVal::Bol(
                            vv2node_id(&env.arg(0))
                .eq_variant(&vv2node_id(&env.arg(1)))))
        }, Some(2), Some(2), false);

    st.fun(
        "param_by_idx", move |env: &mut Env, argc: usize| {
            let nid = vv2node_id(&env.arg(0));
            let param = nid.param_by_idx(env.arg(1).i() as usize);

            Ok(param.map_or(VVal::None,
                |param| VVal::new_usr(VValParamId { param })))
        }, Some(2), Some(2), false);

    st.fun(
        "inp_param", move |env: &mut Env, argc: usize| {
            let nid = vv2node_id(&env.arg(0));
            let param = env.arg(1).with_s_ref(|s| nid.inp_param(s));

            Ok(param.map_or(VVal::None,
                |param| VVal::new_usr(VValParamId { param })))
        }, Some(2), Some(2), false);

    st.fun(
        "param_list", move |env: &mut Env, argc: usize| {
            let nid = vv2node_id(&env.arg(0));

            let atoms = VVal::vec();
            let mut i = 0;
            while let Some(param) = nid.atom_param_by_idx(i) {
                atoms.push(VVal::new_usr(VValParamId { param }));
                i += 1;
            }

            let inputs = VVal::vec();
            let mut i = 0;
            while let Some(param) = nid.inp_param_by_idx(i) {
                inputs.push(VVal::new_usr(VValParamId { param }));
                i += 1;
            }

            Ok(VVal::map2(
                "atoms",  atoms,
                "inputs", inputs,
            ))
        }, Some(1), Some(1), false);

    st.fun(
        "inp_name2idx", move |env: &mut Env, argc: usize| {
            let nid   = vv2node_id(&env.arg(0));
            let idx = env.arg(1).with_s_ref(|s| nid.inp(s));
            Ok(idx.map_or(VVal::None, |idx| VVal::Int(idx as i64)))
        }, Some(2), Some(2), false);

    st.fun(
        "out_name2idx", move |env: &mut Env, argc: usize| {
            let nid   = vv2node_id(&env.arg(0));
            let idx = env.arg(1).with_s_ref(|s| nid.out(s));
            Ok(idx.map_or(VVal::None, |idx| VVal::Int(idx as i64)))
        }, Some(2), Some(2), false);

    st.fun(
        "inp_idx2name", move |env: &mut Env, argc: usize| {
            let nid = vv2node_id(&env.arg(0));
            let name = nid.inp_name_by_idx(env.arg(1).i() as u8);
            Ok(name.map_or(VVal::None, |name| VVal::new_str(name)))
        }, Some(2), Some(2), false);

    st.fun(
        "out_idx2name", move |env: &mut Env, argc: usize| {
            let nid  = vv2node_id(&env.arg(0));
            let name = nid.out_name_by_idx(env.arg(1).i() as u8);
            Ok(name.map_or(VVal::None, |name| VVal::new_str(name)))
        }, Some(2), Some(2), false);

    st
}

#[macro_export]
macro_rules! set_modfun {
    ($st: expr, $ref: ident, $fun: tt, $min: expr, $max: expr, $env: ident, $argc: ident, $b: block) => {
        {
            let $ref = $ref.clone();
            $st.fun(
                &stringify!($fun),
                move |$env: &mut Env, $argc: usize| $b, $min, $max, false);
        }
    }
}

fn setup_vizia_module(r: Rc<RefCell<GUIActionRecorder>>) -> wlambda::SymbolTable {
    let mut st = wlambda::SymbolTable::new();

    set_modfun!(st, r, redraw, Some(0), Some(0), env, argc, {
        r.borrow_mut().actions.push(GUIAction::Redraw);
        Ok(VVal::None)
    });

    set_modfun!(st, r, set_text, Some(2), Some(2), env, _argc, {
        r.borrow_mut().actions.push(
            GUIAction::SetText(
                env.arg(0).i(), env.arg(1).s_raw()));
        Ok(VVal::None)
    });

    set_modfun!(st, r, emit_to, Some(3), Some(3), env, _argc, {
        r.borrow_mut().actions.push(
            GUIAction::EmitTo(
                env.arg(0).i(), env.arg(1).i(), env.arg(2)));
        Ok(VVal::None)
    });

    set_modfun!(st, r, add_theme, Some(1), Some(1), env, _argc, {
        r.borrow_mut().actions.push(
            GUIAction::AddTheme(env.arg(0).s_raw()));
        Ok(VVal::None)
    });

    set_modfun!(st, r, remove, Some(1), Some(1), env, _argc, {
        r.borrow_mut().actions.push(GUIAction::Remove(env.arg(0).i()));
        Ok(VVal::None)
    });

    set_modfun!(st, r, remove_all_childs, Some(1), Some(1), env, _argc, {
        r.borrow_mut().actions.push(GUIAction::RemoveAllChilds(env.arg(0).i()));
        Ok(VVal::None)
    });


    set_modfun!(st, r, new_row, Some(1), Some(2), env, _argc, {
        let mut r = r.borrow_mut();
        Ok(VVal::Int(r.new_row(env.arg(0).i(), env.arg(1))))
    });

    set_modfun!(st, r, new_col, Some(1), Some(2), env, _argc, {
        Ok(VVal::Int(r.borrow_mut().add(|id|
            GUIAction::NewCol(env.arg(0).i(), id, env.arg(1)))))
    });

    set_modfun!(st, r, new_elem, Some(1), Some(2), env, _argc, {
        Ok(VVal::Int(r.borrow_mut().add(|id|
            GUIAction::NewElem(env.arg(0).i(), id, env.arg(1)))))
    });

    set_modfun!(st, r, new_hexknob, Some(2), Some(3), env, _argc, {
        if let Some(_) = vv2hex_knob_model(env.arg(1)) {
            Ok(VVal::Int(
                r.borrow_mut().add(|id|
                    GUIAction::NewHexKnob(env.arg(0).i(), id, env.arg(1), env.arg(2)))))
        } else {
            wl_panic!(
                "ui.new_hexknob[parent_id, hex_knob_model, build_attrs] \
                not called with a $<UI::HexKnobModel>!");
        }
    });

    set_modfun!(st, r, new_octave_keys, Some(1), Some(2), env, _argc, {
        Ok(VVal::Int(
            r.borrow_mut().add(|id|
                GUIAction::NewOctaveKeys(env.arg(0).i(), id, env.arg(1)))))
    });

    set_modfun!(st, r, new_cv_array, Some(1), Some(2), env, _argc, {
        Ok(VVal::Int(
            r.borrow_mut().add(|id|
                GUIAction::NewCvArray(env.arg(0).i(), id, env.arg(1)))))
    });

    set_modfun!(st, r, new_hexgrid, Some(1), Some(2), env, _argc, {
        Ok(VVal::Int(
            r.borrow_mut().add(|id|
                GUIAction::NewHexGrid(env.arg(0).i(), id, env.arg(1)))))
    });

    set_modfun!(st, r, new_pattern_editor, Some(1), Some(2), env, _argc, {
        let mut r = r.borrow_mut();
        Ok(VVal::Int(r.new_pattern_editor(env.arg(0).i(), env.arg(1))))
    });

    set_modfun!(st, r, new_tabs, Some(2), Some(3), env, _argc, {
        let mut rr = r.borrow_mut();
        let mut tabs = vec![];
        let ids = VVal::vec();

        env.arg(1).for_each(|v| {
            let id = rr.new_ref();
            tabs.push((v.clone(), id));
            ids.push(VVal::Int(id));
        });

        rr.actions.push(
            GUIAction::NewTabs(tabs, env.arg(0).i(), env.arg(2)));

        Ok(ids)
    });

    set_modfun!(st, r, new_popup, Some(0), Some(1), env, _argc, {
        Ok(VVal::Int(
            r.borrow_mut().add(|id|
                GUIAction::NewPopup(id, env.arg(0)))))
    });

    set_modfun!(st, r, new_button, Some(3), Some(4), env, _argc, {
        let mut r = r.borrow_mut();
        Ok(VVal::Int(r.new_button(
            env.arg(0).i(),
            env.arg(1).s_raw(),
            env.arg(2),
            env.arg(3)
        )))
    });

    st
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
        "new_cluster", move |env: &mut Env, argc: usize| {
            Ok(VVal::new_usr(VValCluster::new()))
        }, Some(0), Some(0), false);

    st.fun(
        "new_sample_buf_from", move |env: &mut Env, argc: usize| {
            let mut v = vec![];
            env.arg(0).with_iter(|it| {
                for (s, _) in it {
                    v.push(s.f() as f32);
                }
            });

            Ok(VVal::new_usr(VValSampleBuf::from_vec(v)))
        }, Some(1), Some(1), false);

    st.fun(
        "dir", move |env: &mut Env, argc: usize| {
            Ok(VVal::new_usr(VValCellDir::from_vval(&env.arg(0))))
        }, Some(1), Some(1), false);

    st.fun(
        "dir_edge", move |env: &mut Env, argc: usize| {
            Ok(VVal::new_usr(VValCellDir::from_vval_edge(&env.arg(0))))
        }, Some(1), Some(1), false);

    st.fun(
        "to_atom", move |env: &mut Env, argc: usize| {
            let v = env.arg(0);
            let atom =
                match v {
                    VVal::Int(i) => SAtom::setting(i),
                    VVal::Flt(f) => SAtom::param(f as f32),
                    VVal::Sym(_) | VVal::Str(_) | VVal::Byt(_)
                        => v.with_s_ref(|s| SAtom::str(s)),
                    VVal::Pair(_) if v.v_s(0) == "audio_sample"
                        => { v.v_with_s_ref(1, |s| SAtom::audio_unloaded(s)) },
                    _ => {
                        let mut ms = vec![];
                        v.with_iter(|iter| {
                            for (v, _) in iter {
                                ms.push(v.f() as f32);
                            }
                        });
                        SAtom::MicroSample(ms)
                    },
                };
            Ok(VVal::new_usr(VValAtom::new(atom)))
        }, Some(1), Some(1), false);

    st.fun(
        "dir_path_from_to", move |env: &mut Env, argc: usize| {
            let from = env.arg(0);
            let to   = env.arg(1);

            let path =
                CellDir::path_from_to(
                    (from.v_i(0) as usize, from.v_i(1) as usize),
                    (to.v_i(0) as usize, to.v_i(1) as usize));

            let pth = VVal::vec();
            for p in path.iter() {
                pth.push(cell_dir2vv(*p));
            }

            Ok(pth)
        }, Some(2), Some(2), false);

    st.fun(
        "create_test_hex_grid_model", |env: &mut Env, argc: usize| {
            Ok(VVal::new_usr(VValHexGridModel {
                model:
                    HexGridModelType::Test(
                        Rc::new(RefCell::new(
                            grid_models::TestGridModel::new()))),
            }))
        }, Some(0), Some(0), false);

    st
}

struct MatrixRecorder {
    changes: Mutex<Vec<VVal>>,
}

impl MatrixRecorder {
    pub fn new() -> Self {
        Self {
            changes: Mutex::new(vec![]),
        }
    }

    pub fn get_records(&self) -> VVal {
        if let Ok(mut changes) = self.changes.lock() {

            if changes.is_empty() {
                VVal::None

            } else {
                let vec = VVal::vec();
                for c in changes.iter() {
                    vec.push(c.clone());
                }
                changes.clear();
                vec
            }
        } else {
            VVal::None
        }
    }
}

impl MatrixObserver for MatrixRecorder {
    fn update_prop(&self, key: &str) {
        if let Ok(mut changes) = self.changes.lock() {
            changes.push(
                VVal::pair(
                    VVal::new_sym("matrix_property"),
                    VVal::new_str(key)));
        }
    }

    fn update_monitor(&self, cell: &Cell) {
        if let Ok(mut changes) = self.changes.lock() {
            changes.push(
                VVal::pair(
                    VVal::new_sym("matrix_monitor"),
                    cell2vval(cell)));
        }
    }

    fn update_param(&self, param_id: &ParamId) {
        if let Ok(mut changes) = self.changes.lock() {
            changes.push(
                VVal::pair(
                    VVal::new_sym("matrix_param"),
                    VVal::new_usr(VValParamId { param: param_id.clone() })));
        }
    }

    fn update_matrix(&self) {
        if let Ok(mut changes) = self.changes.lock() {
            changes.push(
                VVal::pair(VVal::new_sym("matrix_graph"), VVal::None));
        }
    }

    fn update_all(&self) {
        if let Ok(mut changes) = self.changes.lock() {
            changes.push(
                VVal::pair(VVal::new_sym("matrix_all"), VVal::None));
        }
    }
}

fn main() {
    synth::start(move |matrix| {
        let matrix_recorder = Arc::new(MatrixRecorder::new());
        if let Ok(mut matrix) = matrix.lock() {
            matrix.set_observer(matrix_recorder.clone());
        }

        let global_env = wlambda::GlobalEnv::new_default();
        global_env.borrow_mut().set_module("hx",        setup_hx_module(matrix));
        global_env.borrow_mut().set_module("node_id",   setup_node_id_module());

        let mut wl_ctx  = wlambda::EvalContext::new(global_env.clone());
        let wl_ctx      = Rc::new(RefCell::new(wl_ctx));
        let wl_ctx_idle = wl_ctx.clone();

        let mut idle_cb = VVal::None;

        let gui_rec = GUIActionRecorder::new();
        let gui_rec_idle = gui_rec.clone();

        let mut root_entity = Rc::new(RefCell::new(Entity::null()));

        let mut app =
            Application::new(
                WindowDescription::new(),
                |state, window| {
                    let gui_rec_self = gui_rec.clone();

                    *(root_entity.borrow_mut()) =
                        (HiddenThingie { })
                        .build(state, window, |builder| builder);

                    gui_rec_self.borrow_mut().set_root(*root_entity.borrow());

                    state.add_font_mem(
                        "font_serif",
                        std::include_bytes!("font.ttf"));

                    state.add_font_mem(
                        "font_mono",
                        std::include_bytes!("font_mono.ttf"));

                    state.set_default_font("font_serif");

                    let vizia_st = setup_vizia_module(gui_rec.clone());
                    global_env.borrow_mut().set_module("vizia", vizia_st);

                    match wl_ctx.borrow_mut().eval_file("wllib/main.wl") {
                        Ok(_) => { },
                        Err(e) => { panic!("Error in main.wl: {:?}", e); }
                    }

                    let init_fun =
                        wl_ctx.borrow_mut().get_global_var("init")
                           .expect("global 'init' function in main.wl defined");

                    match wl_ctx.borrow_mut().call(&init_fun, &[]) {
                        Ok(_) => {},
                        Err(e) => { panic!("Error in main.wl 'init': {:?}", e); }
                    }

                    idle_cb = wl_ctx.borrow_mut().get_global_var("idle").unwrap_or(VVal::None);

                    gui_rec.borrow_mut().run(gui_rec_self, wl_ctx, state, *(root_entity.borrow()));
                })
            .on_idle(move |state| {
                let recs = matrix_recorder.get_records();

                if idle_cb.is_some() {
                    match wl_ctx_idle
                            .borrow_mut()
                            .call(&idle_cb, &[recs])
                    {
                        Ok(_) => {},
                        Err(e) => {
                            panic!("Error in main.wl 'idle': {:?}", e);
                        }
                    }
                }

                let gui_rec_self = gui_rec_idle.clone();

                gui_rec_idle
                    .borrow_mut()
                    .run(gui_rec_self, wl_ctx_idle.clone(),
                         state, *root_entity.borrow());
            });
        app.run();
    });
}
