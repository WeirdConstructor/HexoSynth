// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::arg_chk;
use crate::wl_panic;

use crate::wlapi::*;

use super::super::VOctaveKeysModel;
use super::super::VVPatEditFb;
use super::super::VVPatModel;
use super::super::VValHexKnobModel;

use crate::matrix_param_model::KnobParam;

use hexodsp;
use wlambda::*;

use hexodsp::matrix::MatrixError;
use hexodsp::{Cell, CellDir, Matrix, MatrixCellChain, NodeId};

use hexotk::DummyParamModel;
pub use hexotk::PatternEditorFeedback;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub struct MatrixPatEditFb {
    matrix: Arc<Mutex<Matrix>>,
    node_id: NodeId,
}

impl std::fmt::Debug for MatrixPatEditFb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "MatrixPatEditFb()")
    }
}

impl PatternEditorFeedback for MatrixPatEditFb {
    fn get_phase(&self) -> f32 {
        if let Ok(m) = self.matrix.lock() {
            m.phase_value_for(&self.node_id)
        } else {
            0.0
        }
    }
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

fn matrix_error2vval_err(err: MatrixError) -> VVal {
    let err_val = match err {
        MatrixError::CycleDetected => VVal::new_sym("cycle-detected"),
        MatrixError::PosOutOfRange => VVal::new_sym("pos-out-of-range"),
        MatrixError::NonEmptyCell { cell } => {
            VVal::pair(VVal::new_sym("non-empty-cell"), cell2vval(&cell))
        }
        MatrixError::DuplicatedInput { output1, output2 } => VVal::vec3(
            VVal::new_sym("duplicated-input"),
            output2vval(output1.0, output1.1),
            output2vval(output2.0, output2.1),
        ),
    };

    VVal::Err(Rc::new(RefCell::new((err_val, wlambda::vval::SynPos::empty()))))
}

fn build_cell_chain(dir: CellDir, v: &VVal) -> MatrixCellChain {
    let mut chain = MatrixCellChain::new(dir);

    v.v_k("chain").with_iter(|iter| {
        for (v, _) in iter {
            if v.len() == 1 {
                chain.node(&v.v_s_raw(0));
            } else if v.len() == 2 {
                chain.node_out(&v.v_s_raw(0), &v.v_s_raw(1));
            } else {
                if v.v_(2).is_none() {
                    chain.node_inp(&v.v_s_raw(0), &v.v_s_raw(1));
                } else {
                    chain.node_io(&v.v_s_raw(0), &v.v_s_raw(1), &v.v_s_raw(2));
                }
            };
        }
    });

    chain
}

#[derive(Clone)]
pub struct VValMatrix {
    matrix: Arc<Mutex<hexodsp::Matrix>>,
}

impl vval::VValUserData for VValMatrix {
    fn s(&self) -> String {
        format!("$<HexoDSP::Matrix>")
    }

    fn get_key(&self, _key: &str) -> Option<VVal> {
        None
    }

    fn call_method(&self, key: &str, env: &mut Env) -> Result<VVal, StackAction> {
        let args = env.argv_ref();

        match key {
            "create_grid_model" => {
                arg_chk!(args, 0, "matrix.create_grid_model[]");

                let matrix = self.matrix.clone();

                return Ok(VVal::new_usr(VValHexGridModel {
                    model: HexGridModelType::Matrix(Rc::new(RefCell::new(MatrixUIModel::new(
                        matrix,
                    )))),
                }));
            }
            "create_hex_knob_dummy_model" => {
                arg_chk!(args, 0, "matrix.create_hex_knob_dummy_model[]");

                return Ok(VVal::new_usr(VValHexKnobModel {
                    model: Rc::new(RefCell::new(DummyParamModel::new())),
                }));
            }
            "create_hex_knob_model" => {
                arg_chk!(args, 1, "matrix.create_hex_knob_model[param_id]");

                let matrix = self.matrix.clone();
                if let Some(param_id) = vv2param_id(env.arg(0)) {
                    return Ok(VVal::new_usr(VValHexKnobModel {
                        model: Rc::new(RefCell::new(KnobParam::new(matrix, param_id))),
                    }));
                } else {
                    wl_panic!(
                        "matrix.create_hex_knob_model[param_id] requires \
                        a $<HexoDSP::ParamId> as first argument."
                    );
                }
            }
            "create_octave_keys_model" => {
                arg_chk!(args, 1, "matrix.create_octave_keys_model[param_id]");

                let matrix = self.matrix.clone();
                if let Some(param_id) = vv2param_id(env.arg(0)) {
                    return Ok(VVal::new_usr(VOctaveKeysModel::new(matrix, param_id)));
                } else {
                    wl_panic!(
                        "matrix.create_octave_keys_model[param_id] requires \
                        a $<HexoDSP::ParamId> as first argument."
                    );
                }
            }
            "create_graph_model" => {
                arg_chk!(args, 1, "matrix.create_graph_model[node_id]");

                let matrix = self.matrix.clone();
                let node_id = vv2node_id(&args[0]);
                if node_id.graph_fun().is_some() {
                    return Ok(VVal::new_usr(VGraphModel::new(matrix, node_id)));
                } else {
                    return Ok(VVal::None);
                }
            }
            "create_graph_minmax_monitor" => {
                arg_chk!(args, 1, "matrix.create_graph_minmax_monitor[index]");

                let matrix = self.matrix.clone();
                return Ok(VVal::new_usr(VGraphMinMaxModel::new_monitor_model(
                    matrix,
                    args[0].i() as usize,
                )));
            }
            "get_scope_handle" => {
                arg_chk!(args, 1, "matrix.get_scope_handle[scope_node_id]");

                return Ok(VVal::new_usr(VScopeModel::new(
                    self.matrix.clone(),
                    vv2node_id(&args[0]),
                )));
            }
            _ => {}
        }

        let m_clone = self.matrix.clone();

        let m = self.matrix.lock();

        if let Ok(mut m) = m {
            match key {
                "get" => {
                    arg_chk!(args, 1, "matrix.get[$i(x, y)]");

                    if let Some(cell) =
                        m.get(env.arg(0).v_i(0) as usize, env.arg(0).v_i(1) as usize)
                    {
                        Ok(cell2vval(cell))
                    } else {
                        Ok(VVal::None)
                    }
                }
                "set" => {
                    arg_chk!(args, 2, "matrix.set[$i(x, y), cell]");

                    if let (Some(vv_cell), Some(pos)) = (env.arg_ref(1), env.arg_ref(0)) {
                        let cell = vv2cell(vv_cell);

                        let x = pos.v_i(0) as usize;
                        let y = pos.v_i(1) as usize;

                        let (mw, mh) = m.size();
                        if x >= mw || y >= mh {
                            return Ok(VVal::None);
                        };

                        m.place(x, y, cell);

                        Ok(VVal::Bol(true))
                    } else {
                        Ok(VVal::None)
                    }
                }
                "size" => {
                    arg_chk!(args, 0, "matrix.size[]");
                    let (mw, mh) = m.size();
                    Ok(VVal::ivec2(mw as i64, mh as i64))
                }
                "place_chain" => {
                    arg_chk!(args, 3, "matrix.place_chain[pos, dir, chain]");

                    let (x, y) = (args[0].v_i(0) as usize, args[0].v_i(1) as usize);

                    let dir = vv2cell_dir(&args[1]);
                    let mut chain = build_cell_chain(dir, &args[2]);

                    //d// println!("CHAIN: {:#?}", chain);

                    args[2].v_k("params").with_iter(|it| {
                        let mut i = 0;
                        for (link_params, _) in it {
                            chain.params_for_idx(i);

                            link_params.with_iter(|it| {
                                for (v, _) in it {
                                    if let VVal::Flt(denorm) = v.v_(1) {
                                        chain.set_denorm(&v.v_s_raw(0), denorm as f32);
                                    } else {
                                        chain.set_atom(&v.v_s_raw(0), vv2atom(v.v_(1)));
                                    }
                                }
                            });

                            i += 1;
                        }
                    });

                    if let Err(e) = chain.place(&mut m, x, y) {
                        Ok(VVal::err_msg(&format!("Couldn't place DSP chain: {:?}", e)))
                    } else {
                        Ok(VVal::None)
                    }
                }
                "set_param" => {
                    arg_chk!(args, 2, "matrix.set_param[param_id, atom or value]");

                    let pid = vv2param_id(env.arg(0));
                    let at = vv2atom(env.arg(1));

                    if let Some(pid) = pid {
                        m.set_param(pid, at);
                        Ok(VVal::Bol(true))
                    } else {
                        Ok(VVal::None)
                    }
                }
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
                }
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
                }
                "set_param_modamt" => {
                    arg_chk!(args, 2, "matrix.set_param_modamt[param_id, none or float]");

                    let pid = vv2param_id(env.arg(0));
                    let ma = env.arg(1);

                    if let Some(pid) = pid {
                        let ma = if ma.is_some() { Some(ma.f() as f32) } else { None };

                        match m.set_param_modamt(pid, ma) {
                            Ok(_) => Ok(VVal::Bol(true)),
                            Err(e) => Ok(matrix_error2vval_err(e)),
                        }
                    } else {
                        Ok(VVal::None)
                    }
                }
                "inject_midi_event" => {
                    arg_chk!(args, 1, "matrix.inject_midi_event[${type=str,cc=num,note=num,channel=num,value=flt}]");

                    let typ = args[0].v_s_rawk("type");

                    let event = match &typ[..] {
                        "note_on" => hexodsp::HxMidiEvent::NoteOn {
                            channel: args[0].v_ik("channel") as u8,
                            note: args[0].v_ik("note") as u8,
                            vel: args[0].v_fk("velocity") as f32,
                        },
                        "note_off" => hexodsp::HxMidiEvent::NoteOff {
                            channel: args[0].v_ik("channel") as u8,
                            note: args[0].v_ik("note") as u8,
                        },
                        "cc" => hexodsp::HxMidiEvent::CC {
                            channel: args[0].v_ik("channel") as u8,
                            cc: args[0].v_ik("cc") as u8,
                            value: args[0].v_fk("value") as f32,
                        },
                        _ => {
                            return Ok(VVal::err_msg(&format!(
                                "Not a MIDI message 'type': {:?}",
                                args[0].s()
                            )))
                        }
                    };

                    m.inject_midi_event(event);

                    Ok(VVal::None)
                }
                "cell_edge_labels" => {
                    arg_chk!(args, 1, "matrix.cell_edge_labels[$i(x, y)]");

                    let mut buf: [u8; 30] = [0; 30];

                    if let Some(cell) =
                        m.get(env.arg(0).v_i(0) as usize, env.arg(0).v_i(1) as usize)
                    {
                        let v_t =
                            if let Some((lbl, _)) = m.edge_label(cell, CellDir::T, &mut buf[..]) {
                                VVal::new_str(lbl)
                            } else {
                                VVal::None
                            };
                        let v_tl =
                            if let Some((lbl, _)) = m.edge_label(cell, CellDir::TL, &mut buf[..]) {
                                VVal::new_str(lbl)
                            } else {
                                VVal::None
                            };
                        let v_tr =
                            if let Some((lbl, _)) = m.edge_label(cell, CellDir::TR, &mut buf[..]) {
                                VVal::new_str(lbl)
                            } else {
                                VVal::None
                            };

                        let v_b =
                            if let Some((lbl, _)) = m.edge_label(cell, CellDir::B, &mut buf[..]) {
                                VVal::new_str(lbl)
                            } else {
                                VVal::None
                            };
                        let v_bl =
                            if let Some((lbl, _)) = m.edge_label(cell, CellDir::BL, &mut buf[..]) {
                                VVal::new_str(lbl)
                            } else {
                                VVal::None
                            };
                        let v_br =
                            if let Some((lbl, _)) = m.edge_label(cell, CellDir::BR, &mut buf[..]) {
                                VVal::new_str(lbl)
                            } else {
                                VVal::None
                            };

                        let out = VVal::map();
                        let _ = out.set_key_str("t", v_t);
                        let _ = out.set_key_str("tl", v_tl);
                        let _ = out.set_key_str("tr", v_tr);
                        let _ = out.set_key_str("b", v_b);
                        let _ = out.set_key_str("bl", v_bl);
                        let _ = out.set_key_str("br", v_br);
                        Ok(out)
                    } else {
                        Ok(VVal::None)
                    }
                }
                "param_input_is_used" => {
                    arg_chk!(args, 1, "matrix.param_input_is_used[param_id]");
                    if let Some(pid) = vv2param_id(env.arg(0)) {
                        Ok(VVal::Bol(m.param_input_is_used(pid)))
                    } else {
                        Ok(VVal::None)
                    }
                }
                "get_filled_positions" => {
                    arg_chk!(args, 0, "matrix.get_filled_positions[]");
                    let ret = VVal::vec();

                    m.for_each(|x, y, cell| {
                        if cell.node_id() != NodeId::Nop {
                            ret.push(VVal::ivec2(x as i64, y as i64));
                        }
                    });

                    Ok(ret)
                }
                "find_unused_inputs" => {
                    arg_chk!(args, 1, "matrix.param_input_is_used[node_id]");

                    let node_id = vv2node_id(&args[0]);
                    let unused = VVal::vec();

                    let mut inp_idx = 0;
                    while let Some(param_id) = node_id.inp_param_by_idx(inp_idx) {
                        if !m.param_input_is_used(param_id) {
                            unused.push(param_id2vv(param_id));
                        }
                        inp_idx += 1;
                    }

                    Ok(unused)
                }
                "find_first_adjacent_free" => {
                    arg_chk!(args, 2, "matrix.find_first_adjacent_free[$i(x, y), cell_dir]");

                    if let Some(cell) = m.get(args[0].v_i(0) as usize, args[0].v_i(1) as usize) {
                        if let Some((free_dir, _)) =
                            cell.find_first_adjacent_free(&m, vv2cell_dir(&args[1]))
                        {
                            Ok(cell_dir2vv(free_dir))
                        } else {
                            Ok(VVal::None)
                        }
                    } else {
                        Ok(VVal::None)
                    }
                }
                "find_all_adjacent_free" => {
                    arg_chk!(args, 2, "matrix.find_all_adjacent_free[$i(x, y), cell_dir]");

                    if let Some(cell) = m.get_copy(args[0].v_i(0) as usize, args[0].v_i(1) as usize)
                    {
                        let free_pos = cell.find_all_adjacent_free(&mut *m, vv2cell_dir(&args[1]));

                        let ret = VVal::vec();
                        for (dir, (x, y)) in free_pos {
                            ret.push(VVal::map2(
                                "dir",
                                cell_dir2vv(dir),
                                "pos",
                                VVal::ivec2(x as i64, y as i64),
                            ));
                        }

                        Ok(ret)
                    } else {
                        Ok(VVal::None)
                    }
                }
                "find_unconnected_ports" => {
                    arg_chk!(args, 2, "matrix.find_unconnected_ports[$i(x, y), cell_dir]");

                    if let Some(cell) = m.get_copy(args[0].v_i(0) as usize, args[0].v_i(1) as usize)
                    {
                        let free_pos = cell.find_unconnected_ports(&mut *m, vv2cell_dir(&args[1]));

                        let ret = VVal::vec();
                        for dir in free_pos {
                            ret.push(cell_dir2vv(dir));
                        }

                        Ok(ret)
                    } else {
                        Ok(VVal::None)
                    }
                }
                "get_connections" => {
                    arg_chk!(args, 1, "matrix.get_connections[$i(x, y)]");

                    if let Some(conn) =
                        m.get_connections(args[0].v_i(0) as usize, args[0].v_i(1) as usize)
                    {
                        let ret = VVal::vec();

                        for (
                            (center_cell, center_dir, _center_io_idx),
                            (other_cell, other_dir, _other_io_idx, (ox, oy)),
                        ) in conn.iter()
                        {
                            ret.push(VVal::map2(
                                "center",
                                VVal::map2(
                                    "dir",
                                    cell_dir2vv(*center_dir),
                                    "port",
                                    cell_port2vval(center_cell, *center_dir),
                                ),
                                "other",
                                VVal::map3(
                                    "dir",
                                    cell_dir2vv(*other_dir),
                                    "pos",
                                    VVal::ivec2(*ox as i64, *oy as i64),
                                    "port",
                                    cell_port2vval(other_cell, *other_dir),
                                ),
                            ));
                        }

                        Ok(ret)
                    } else {
                        Ok(VVal::None)
                    }
                }
                "restore_snapshot" => {
                    arg_chk!(args, 0, "matrix.restore_snapshot[]");
                    m.restore_matrix();
                    Ok(VVal::Bol(true))
                }
                "save_snapshot" => {
                    arg_chk!(args, 0, "matrix.save_snapshot[]");
                    m.save_matrix();
                    Ok(VVal::Bol(true))
                }
                "check" => {
                    arg_chk!(args, 0, "matrix.check[]");

                    match m.check() {
                        Ok(_) => Ok(VVal::Bol(true)),
                        Err(e) => Ok(matrix_error2vval_err(e)),
                    }
                }
                "load_init_patch" => {
                    arg_chk!(args, 0, "matrix.load_init_patch[]");

                    use hexodsp::matrix_repr::load_patch_from_mem;

                    match load_patch_from_mem(
                        &mut m,
                        include_bytes!("../../res/init_patch_2022_1.hxy"),
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            return Ok(VVal::err_msg(&format!("{:?}", e)));
                        }
                    }

                    match m.sync() {
                        Ok(_) => Ok(VVal::Bol(true)),
                        Err(e) => Ok(matrix_error2vval_err(e)),
                    }
                }
                "load_patch" => {
                    arg_chk!(args, 1, "matrix.load_patch[filepath]");

                    use hexodsp::matrix_repr::load_patch_from_file;

                    match load_patch_from_file(&mut m, &env.arg(0).s_raw()) {
                        Ok(_) => {}
                        Err(e) => {
                            return Ok(VVal::err_msg(&format!("{:?}", e)));
                        }
                    }

                    match m.sync() {
                        Ok(_) => Ok(VVal::Bol(true)),
                        Err(e) => Ok(matrix_error2vval_err(e)),
                    }
                }
                "save_patch" => {
                    arg_chk!(args, 1, "matrix.save_patch[filepath]");

                    use hexodsp::matrix_repr::save_patch_to_file;

                    match save_patch_to_file(&mut m, &env.arg(0).s_raw()) {
                        Ok(_) => {
                            let cwd = std::env::current_dir()
                                .unwrap_or_else(|_| std::path::PathBuf::from("."));

                            Ok(VVal::new_str(cwd.to_str().unwrap_or("?")))
                        }
                        Err(e) => Ok(VVal::err_msg(&format!("{}", e))),
                    }
                }
                "sync" => {
                    arg_chk!(args, 0, "matrix.sync[]");

                    match m.sync() {
                        Ok(_) => Ok(VVal::Bol(true)),
                        Err(e) => Ok(matrix_error2vval_err(e)),
                    }
                }
                "clear" => {
                    arg_chk!(args, 0, "matrix.clear[]");

                    m.clear();
                    Ok(VVal::Bol(true))
                }
                "monitored_cell" => {
                    arg_chk!(args, 0, "matrix.monitored_cell[]");

                    Ok(cell2vval(m.monitored_cell()))
                }
                "monitor_cell" => {
                    arg_chk!(args, 1, "matrix.monitor_cell[cell]");

                    let cell = vv2cell(&args[0]);
                    m.monitor_cell(cell);
                    Ok(VVal::Bol(true))
                }
                "pop_error" => {
                    arg_chk!(args, 0, "matrix.pop_error[]");

                    Ok(m.pop_error().map(|s| VVal::new_str_mv(s)).unwrap_or_else(|| VVal::None))
                }
                "get_unused_instance_node_id" => {
                    arg_chk!(args, 1, "matrix.get_unused_instance_node_id[node_id]");

                    let node_id = vv2node_id(&args[0]);
                    let node_id = m.get_unused_instance_node_id(node_id);
                    Ok(node_id2vv(node_id))
                }
                "create_pattern_data_model" => {
                    arg_chk!(args, 1, "matrix.create_pattern_data_model[tracker_id]");

                    if let Some(model) = m.get_pattern_data(args[0].i() as usize) {
                        return Ok(VVPatModel::new_vv(model));
                    } else {
                        return Ok(VVal::None);
                    }
                }
                "get_block_function" => {
                    arg_chk!(args, 1, "matrix.get_block_function[block_fun_id]");

                    if let Some(fun) = m.get_block_function(args[0].i() as usize) {
                        return Ok(VValBlockFun::from(fun));
                    } else {
                        return Ok(VVal::None);
                    }
                }
                "check_pattern_data" => {
                    arg_chk!(args, 1, "matrix.check_pattern_data[tracker_id]");

                    m.check_pattern_data(args[0].i() as usize);
                    Ok(VVal::None)
                }
                "check_block_function" => {
                    arg_chk!(args, 1, "matrix.check_block_function[block_fun_id]");

                    match m.check_block_function(args[0].i() as usize) {
                        Err(e) => {
                            Ok(VVal::err_msg(&format!("Block Function Compile Error: {:?}", e)))
                        }
                        Ok(()) => Ok(VVal::Bol(true)),
                    }
                }
                "handle_graph_events" => {
                    arg_chk!(args, 0, "matrix.handle_graph_events[]");
                    m.handle_graph_events();
                    Ok(VVal::Bol(true))
                }
                "create_pattern_feedback_model" => {
                    arg_chk!(args, 1, "matrix.create_pattern_feedback_model[node_id]");

                    Ok(VVPatEditFb::new_vv(Arc::new(Mutex::new(MatrixPatEditFb {
                        matrix: m_clone.clone(),
                        node_id: vv2node_id(&args[0]),
                    }))))
                }
                _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
            }
        } else {
            Ok(VVal::err_msg("Can't lock matrix!"))
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct VValCellDir {
    dir: CellDir,
}

pub fn vv2cell_dir(v: &VVal) -> CellDir {
    v.with_s_ref(|s| match s {
        "c" | "C" => CellDir::C,
        "t" | "T" => CellDir::T,
        "b" | "B" => CellDir::B,
        "tl" | "TL" => CellDir::TL,
        "bl" | "BL" => CellDir::BL,
        "tr" | "TR" => CellDir::TR,
        "br" | "BR" => CellDir::BR,
        _ => CellDir::C,
    })
}

impl VValCellDir {
    pub fn from_vval(v: &VVal) -> Self {
        Self { dir: vv2cell_dir(v) }
    }

    pub fn from_vval_edge(v: &VVal) -> Self {
        Self { dir: CellDir::from(v.i() as u8) }
    }
}

pub fn cell_dir2vv(dir: CellDir) -> VVal {
    VVal::new_usr(VValCellDir { dir })
}

impl vval::VValUserData for VValCellDir {
    fn s(&self) -> String {
        format!("$<CellDir::{:?}>", self.dir)
    }

    fn call_method(&self, key: &str, env: &mut Env) -> Result<VVal, StackAction> {
        let args = env.argv_ref();

        match key {
            "as_edge" => {
                arg_chk!(args, 0, "cell_dir.as_edge[]");

                Ok(VVal::Int(self.dir.as_edge() as i64))
            }
            "flip" => {
                arg_chk!(args, 0, "cell_dir.flip[]");

                Ok(cell_dir2vv(self.dir.flip()))
            }
            "is_input" => {
                arg_chk!(args, 0, "cell_dir.is_input[]");

                Ok(VVal::Bol(self.dir.is_input()))
            }
            "is_output" => {
                arg_chk!(args, 0, "cell_dir.is_output[]");

                Ok(VVal::Bol(self.dir.is_output()))
            }
            "offs_pos" => {
                arg_chk!(args, 1, "cell_dir.offs_pos[$i(x, y)]");

                let p = env.arg(0);

                let pos = (p.v_i(0) as usize, p.v_i(1) as usize);

                if let Some(opos) = self.dir.offs_pos(pos) {
                    Ok(VVal::ivec2(opos.0 as i64, opos.1 as i64))
                } else {
                    Ok(VVal::None)
                }
            }
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> {
        Box::new(self.clone())
    }
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
        VVal::ivec2(cell.pos().0 as i64, cell.pos().1 as i64),
        "ports",
        ports,
    )
}

pub fn vv2cell(v: &VVal) -> Cell {
    let node_id = vv2node_id(&v.v_k("node_id"));

    let pos = v.v_k("pos");

    let mut m_cell = Cell::empty_at(node_id, pos.v_i(0) as u8, pos.v_i(1) as u8);

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
        Self { cluster: Rc::new(RefCell::new(crate::cluster::Cluster::new())) }
    }
}

impl vval::VValUserData for VValCluster {
    fn s(&self) -> String {
        format!("$<HexoDSP::Cluster>")
    }

    fn call_method(&self, key: &str, env: &mut Env) -> Result<VVal, StackAction> {
        let args = env.argv_ref();

        match key {
            "add_cluster_at" => {
                arg_chk!(args, 2, "cluster.add_cluster_at[matrix, $i(x, y)]");

                let mut m = env.arg(0);

                if let Some(matrix) = m.with_usr_ref(|m: &mut VValMatrix| m.matrix.clone()) {
                    if let Ok(mut m) = matrix.lock() {
                        let v = env.arg(1);

                        self.cluster
                            .borrow_mut()
                            .add_cluster_at(&mut m, (v.v_i(0) as usize, v.v_i(1) as usize));
                    }
                }

                Ok(VVal::None)
            }
            "ignore_pos" => {
                arg_chk!(args, 1, "cluster.ignore_pos[$i(x, y)]");

                let v = env.arg(0);

                self.cluster.borrow_mut().ignore_pos((v.v_i(0) as usize, v.v_i(1) as usize));

                Ok(VVal::None)
            }
            "position_list" => {
                arg_chk!(args, 0, "cluster.position_list[]");

                let v = VVal::vec();

                self.cluster.borrow().for_poses(|pos| {
                    v.push(VVal::ivec2(pos.0 as i64, pos.1 as i64));
                });

                Ok(v)
            }
            "cell_list" => {
                arg_chk!(args, 0, "cluster.cell_list[]");

                let v = VVal::vec();

                self.cluster.borrow().for_cells(|cell| {
                    v.push(cell2vval(cell));
                });

                Ok(v)
            }
            "remove_cells" => {
                arg_chk!(args, 1, "cluster.remove_cells[matrix]");

                let mut m = env.arg(0);

                if let Some(matrix) = m.with_usr_ref(|m: &mut VValMatrix| m.matrix.clone()) {
                    if let Ok(mut m) = matrix.lock() {
                        self.cluster.borrow_mut().remove_cells(&mut m);
                    }
                }

                Ok(VVal::None)
            }
            "place" => {
                arg_chk!(args, 1, "cluster.place[matrix]");

                let mut m = env.arg(0);

                if let Some(matrix) = m.with_usr_ref(|m: &mut VValMatrix| m.matrix.clone()) {
                    if let Ok(mut m) = matrix.lock() {
                        return match self.cluster.borrow_mut().place(&mut m) {
                            Ok(_) => Ok(VVal::Bol(true)),
                            Err(e) => Ok(matrix_error2vval_err(e)),
                        };
                    }
                }

                Ok(VVal::None)
            }
            "move_cluster_cells_dir_path" => {
                arg_chk!(args, 1, "cluster.move_cluster_cells_dir_path[$[CellDir, ...]]");

                let path = env.arg(0);
                let mut cd_path = vec![];

                path.for_each(|v| {
                    let mut v = v.clone();

                    if let Some(dir) = v.with_usr_ref(|v: &mut VValCellDir| v.dir) {
                        cd_path.push(dir);
                    } else {
                        cd_path.push(vv2cell_dir(&v));
                    }
                });

                match self.cluster.borrow_mut().move_cluster_cells_dir_path(&cd_path[..]) {
                    Ok(_) => Ok(VVal::Bol(true)),
                    Err(e) => Ok(matrix_error2vval_err(e)),
                }
            }
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> {
        Box::new(self.clone())
    }
}

pub fn matrix2vv(matrix: Arc<Mutex<Matrix>>) -> VVal {
    VVal::new_usr(VValMatrix { matrix })
}
