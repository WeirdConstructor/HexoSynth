// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

pub mod hxdsp;
pub use hxdsp::*;

pub mod hex_knob;
pub use hex_knob::*;

pub mod sample_buf;
pub use sample_buf::*;

pub mod connector;
pub use connector::*;

pub mod wichtext;
pub use wichtext::*;

pub mod pattern_editor;
pub use pattern_editor::*;

pub mod list;
pub use list::*;

use directories::UserDirs;
use std::sync::{Arc, Mutex};
use wlambda::*;

use hexodsp::wblockdsp::setup_hxdsp_block_language;
use hexodsp::{CellDir, Matrix};

#[macro_export]
macro_rules! arg_chk {
    ($args: expr, $count: expr, $name: literal) => {
        if $args.len() != $count {
            return Err(StackAction::panic_msg(format!(
                "{} called with wrong number of arguments",
                $name
            )));
        }
    };
}

#[macro_export]
macro_rules! wl_panic {
    ($str: literal) => {
        return Err(StackAction::panic_msg($str.to_string()))
    };
}

pub fn setup_hx_module(matrix: Arc<Mutex<Matrix>>) -> wlambda::SymbolTable {
    let mut st = wlambda::SymbolTable::new();

    st.fun(
        "get_main_matrix_handle",
        move |_env: &mut Env, _argc: usize| Ok(matrix2vv(matrix.clone())),
        Some(0),
        Some(0),
        false,
    );

    st.fun(
        "new_cluster",
        move |_env: &mut Env, _argc: usize| Ok(VVal::new_usr(VValCluster::new())),
        Some(0),
        Some(0),
        false,
    );

    st.fun(
        "new_sample_buf_from",
        move |env: &mut Env, _argc: usize| {
            let mut v = vec![];
            env.arg(0).with_iter(|it| {
                for (s, _) in it {
                    v.push(s.f() as f32);
                }
            });

            Ok(VVal::new_usr(VValSampleBuf::from_vec(v)))
        },
        Some(1),
        Some(1),
        false,
    );

    st.fun(
        "dir",
        move |env: &mut Env, _argc: usize| Ok(VVal::new_usr(VValCellDir::from_vval(&env.arg(0)))),
        Some(1),
        Some(1),
        false,
    );

    st.fun(
        "dir_edge",
        move |env: &mut Env, _argc: usize| {
            Ok(VVal::new_usr(VValCellDir::from_vval_edge(&env.arg(0))))
        },
        Some(1),
        Some(1),
        false,
    );

    st.fun(
        "to_atom",
        move |env: &mut Env, _argc: usize| Ok(atom2vv(vv2atom(env.arg(0)))),
        Some(1),
        Some(1),
        false,
    );

    st.fun(
        "dir_path_from_to",
        move |env: &mut Env, _argc: usize| {
            let from = env.arg(0);
            let to = env.arg(1);

            let path = CellDir::path_from_to(
                (from.v_i(0) as usize, from.v_i(1) as usize),
                (to.v_i(0) as usize, to.v_i(1) as usize),
            );

            let pth = VVal::vec();
            for p in path.iter() {
                pth.push(cell_dir2vv(*p));
            }

            Ok(pth)
        },
        Some(2),
        Some(2),
        false,
    );

    st.fun(
        "pos_are_adjacent",
        move |env: &mut Env, _argc: usize| {
            let from = env.arg(0);
            let to = env.arg(1);

            if let Some(dir) = CellDir::are_adjacent(
                (from.v_i(0) as usize, from.v_i(1) as usize),
                (to.v_i(0) as usize, to.v_i(1) as usize),
            ) {
                Ok(cell_dir2vv(dir))
            } else {
                Ok(VVal::None)
            }
        },
        Some(2),
        Some(2),
        false,
    );

    st.set("MONITOR_MINMAX_SAMPLES", VVal::Int(hexodsp::monitor::MONITOR_MINMAX_SAMPLES as i64));

    st.fun(
        "create_test_hex_grid_model",
        |_env: &mut Env, _argc: usize| Ok(new_test_grid_model()),
        Some(0),
        Some(0),
        false,
    );

    st.fun(
        "get_directory_patches",
        |_env: &mut Env, _argc: usize| {
            if let Some(user) = UserDirs::new() {
                if let Some(doc_dir) = user.document_dir() {
                    let path = doc_dir.join("HexoSynth").join("patches");
                    let path = path.as_path();

                    if let Some(path_str) = path.to_str() {
                        if let Some(path_name) = path.file_name().map(|f| f.to_str()).flatten() {
                            Ok(VVal::pair(VVal::new_str(path_str), VVal::new_str(path_name)))
                        } else {
                            Ok(VVal::err_msg(&format!("Could not get path directory name!")))
                        }
                    } else {
                        Ok(VVal::err_msg(&format!("Could not create path string!")))
                    }
                } else {
                    Ok(VVal::err_msg(&format!("No Document dir could be found!")))
                }
            } else {
                Ok(VVal::err_msg(&format!("No valid home directory set!")))
            }
        },
        Some(0),
        Some(0),
        false,
    );

    st.fun(
        "subdir_path_of_prefix",
        |env: &mut Env, _argc: usize| {
            let path = env.arg(0).s_raw();
            let path = std::path::Path::new(&path);
            let file_path = env.arg(1).s_raw();
            let file_path = std::path::Path::new(&file_path);

            let path = match path.canonicalize() {
                Ok(path) => path,
                Err(e) => {
                    return Ok(VVal::err_msg(&format!("Can't canonicalize prefix: {}", e)));
                }
            };
            let file_path = match file_path.canonicalize() {
                Ok(path) => path,
                Err(e) => {
                    return Ok(VVal::err_msg(&format!("Can't canonicalize file path: {}", e)));
                }
            };

            let file_rel_path = match file_path.strip_prefix(path.as_path()) {
                Err(_) => return Ok(VVal::None),
                Ok(file_path) => file_path,
            };

            let out = VVal::vec();
            for anc in file_rel_path.ancestors() {
                let abs_path = path.as_path().join(anc);

                if let Some(path_str) = abs_path.to_str() {
                    if path_str.is_empty() {
                        break;
                    }

                    if let Some(path_name) = abs_path.file_name().map(|f| f.to_str()).flatten() {
                        out.push(VVal::pair(VVal::new_str(path_str), VVal::new_str(path_name)));
                    } else {
                        return Ok(VVal::err_msg(&format!("Can't get path name: {:?}", anc)));
                    }
                } else {
                    return Ok(VVal::err_msg(&format!("Can't get path: {:?}", anc)));
                }
            }

            Ok(out)
        },
        Some(2),
        Some(2),
        false,
    );

    st.fun(
        "get_directories_samples",
        |_env: &mut Env, _argc: usize| {
            let list = VVal::vec();

            if let Some(user) = UserDirs::new() {
                if let Some(doc_dir) = user.document_dir() {
                    let path = doc_dir.join("HexoSynth").join("samples");
                    let path = path.as_path();

                    if let Some(path_str) = path.to_str() {
                        if let Some(path_name) = path.file_name().map(|f| f.to_str()).flatten() {
                            list.push(VVal::pair(
                                VVal::new_str(path_str),
                                VVal::new_str(path_name),
                            ));
                        }
                    }
                }
            }

            Ok(list)
        },
        Some(0),
        Some(0),
        false,
    );

    st
}
