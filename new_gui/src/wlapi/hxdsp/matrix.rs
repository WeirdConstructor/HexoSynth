// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::arg_chk;
use crate::wl_panic;

use crate::wlapi::*;
use crate::ui::{DummyParamModel};

use crate::matrix_param_model::KnobParam;

use wlambda::*;
use hexodsp;

use hexodsp::{Matrix, NodeId, Cell, CellDir};
use hexodsp::matrix::{MatrixError};

use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

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
pub struct VValMatrix {
    matrix: Arc<Mutex<hexodsp::Matrix>>,
}

impl vval::VValUserData for VValMatrix {
    fn s(&self) -> String { format!("$<HexoDSP::Matrix>") }

    fn get_key(&self, _key: &str) -> Option<VVal> {
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
                                MatrixUIModel::new(matrix)))),
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
                "set_param" => {
                    arg_chk!(args, 2, "matrix.set_param[param_id, atom or value]");

                    let pid = vv2param_id(env.arg(0));
                    let at  = vv2atom(env.arg(1));

                    if let Some(pid) = pid {
                        m.set_param(pid, at);
                        Ok(VVal::Bol(true))
                    } else {
                        Ok(VVal::None)
                    }
                },
                "get_param" => {
                    arg_chk!(args, 1, "matrix.get_param[param_id]");

                    let pid = vv2param_id(env.arg(0));

                    if let Some(pid) = pid {
                        if let Some(at) = m.get_param(&pid) {
                            Ok(atom2vv(at))
                        } else {
                            Ok(VVal::None)
                        }
                    } else {
                        Ok(VVal::None)
                    }
                },
                "get_param_modamt" => {
                    arg_chk!(args, 1, "matrix.get_param_modamt[param_id]");

                    let pid = vv2param_id(env.arg(0));

                    if let Some(pid) = pid {
                        if let Some(ma) = m.get_param_modamt(&pid) {
                            Ok(VVal::opt(VVal::Flt(ma as f64)))
                        } else {
                            Ok(VVal::opt_none())
                        }
                    } else {
                        Ok(VVal::None)
                    }
                },
                "set_param_modamt" => {
                    arg_chk!(args, 2,
                        "matrix.set_param_modamt[param_id, none or float]");

                    let pid = vv2param_id(env.arg(0));
                    let ma = env.arg(1);

                    if let Some(pid) = pid {
                        let ma =
                            if ma.is_some() { Some(ma.f() as f32) }
                            else { None };

                        match m.set_param_modamt(pid, ma) {
                            Ok(_)  => Ok(VVal::Bol(true)),
                            Err(e) => Ok(matrix_error2vval_err(e)),
                        }
                    } else {
                        Ok(VVal::None)
                    }
                },
                "param_input_is_used" => {
                    arg_chk!(args, 1, "matrix.param_input_is_used[param_id]");
                    if let Some(pid) = vv2param_id(env.arg(0)) {
                        Ok(VVal::Bol(m.param_input_is_used(pid)))
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
                "save_patch" => {
                    arg_chk!(args, 1, "matrix.check[filepath]");

                    use hexodsp::matrix_repr::save_patch_to_file;

                    match save_patch_to_file(&mut m, &env.arg(0).s_raw()) {
                        Ok(_) => {
                            let cwd =
                                std::env::current_dir()
                                    .unwrap_or_else(|_|
                                        std::path::PathBuf::from("."));

                            Ok(VVal::new_str(cwd.to_str().unwrap_or("?")))
                        },
                        Err(e) => {
                            Ok(VVal::err_msg(&format!("{}", e)))
                        },
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
pub struct VValCellDir {
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

pub fn cell_dir2vv(dir: CellDir) -> VVal { VVal::new_usr(VValCellDir { dir }) }

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

pub fn cell2vval(cell: &Cell) -> VVal {
    let node_id = cell.node_id();

    let ports = VVal::vec();
    ports.push(cell_port2vval(cell, CellDir::TR));
    ports.push(cell_port2vval(cell, CellDir::BR));
    ports.push(cell_port2vval(cell, CellDir::B));
    ports.push(cell_port2vval(cell, CellDir::BL));
    ports.push(cell_port2vval(cell, CellDir::TL));
    ports.push(cell_port2vval(cell, CellDir::T));

    VVal::map3(
        "node_id",
        node_id2vv(node_id),
        "pos",
        VVal::ivec2(
            cell.pos().0 as i64,
            cell.pos().1 as i64),
        "ports", ports)
}

pub fn vv2cell(v: &VVal) -> Cell {
    let node_id = vv2node_id(&v.v_k("node_id"));

    let mut m_cell = Cell::empty(node_id);

    let ports = v.v_k("ports");

    cell_set_port(&mut m_cell, ports.v_(0), CellDir::TR);
    cell_set_port(&mut m_cell, ports.v_(1), CellDir::BR);
    cell_set_port(&mut m_cell, ports.v_(2), CellDir::B);
    cell_set_port(&mut m_cell, ports.v_(3), CellDir::BL);
    cell_set_port(&mut m_cell, ports.v_(4), CellDir::TL);
    cell_set_port(&mut m_cell, ports.v_(5), CellDir::T);

    m_cell
}


#[derive(Clone)]
pub struct VValCluster {
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

pub fn matrix2vv(matrix: Arc<Mutex<Matrix>>) -> VVal {
    VVal::new_usr(VValMatrix { matrix })
}