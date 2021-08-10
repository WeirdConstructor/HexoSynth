// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use hexosynth::*;
use hexosynth::dsp::*;
use hexosynth::dsp::tracker::UIPatternModel;

use hexodsp::dsp::tracker::PatternData;
use hexodsp::CellDir;

use hexotk::constants::{dbgid2str, str2dbgid, dbgid_unpack};

use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

use keyboard_types::Key;

use wlambda;
use wlambda::StackAction;
use wlambda::Env;
use wlambda::vval::VVal;
use wlambda::set_vval_method;

use rustyline;

fn start_backend(node_exec: NodeExecutor) {
    let ne  = Arc::new(Mutex::new(node_exec));
    let ne2 = ne.clone();

    let mut in_a  = [0.0; hexodsp::dsp::MAX_BLOCK_SIZE];
    let mut in_b  = [0.0; hexodsp::dsp::MAX_BLOCK_SIZE];
    let mut out_a = [0.0; hexodsp::dsp::MAX_BLOCK_SIZE];
    let mut out_b = [0.0; hexodsp::dsp::MAX_BLOCK_SIZE];

    let us_per_frame =
        (1000000.0 * (hexodsp::dsp::MAX_BLOCK_SIZE as f32)) / 44100.0;
    let us_per_frame = us_per_frame as u128;

    std::thread::spawn(move || {
        let nframes = hexodsp::dsp::MAX_BLOCK_SIZE;

        let mut i2 = std::time::Instant::now();
        let mut us_last_sleep_extra = 0;
        loop {
            let i1 = std::time::Instant::now();

            let mut node_exec = ne.lock().unwrap();

            node_exec.process_graph_updates();

            let output = &mut [&mut out_a[..], &mut out_b[..]];
            let input  = &[&in_a[..], &in_b[..]];

            let mut context =
                Context {
                    nframes,
                    output,
                    input,
                };

            for i in 0..context.nframes {
                context.output[0][i] = 0.0;
                context.output[1][i] = 0.0;
            }

            node_exec.process(&mut context);

            let mut us_passed = i1.elapsed().as_micros();
            us_passed += us_last_sleep_extra;
            let mut us_remaining =
                if us_passed > us_per_frame {
                    us_per_frame - us_passed
                } else {
                    0
                };

            i2 = std::time::Instant::now();
            std::thread::sleep(
                std::time::Duration::from_micros(
                    us_remaining as u64));
            let us_sleep = i2.elapsed().as_micros();
            us_last_sleep_extra =
                if us_sleep > us_remaining {
                    us_sleep - us_remaining
                } else {
                    0
                };
        }
    });
}

struct Ctx {
    drv:    DriverFrontend,
    matrix: Arc<Mutex<Matrix>>,
}

fn zone_type2vval(zt: ZoneType) -> wlambda::VVal {
    match zt {
        ZoneType::ValueDragFine       => VVal::vec1(VVal::new_str("value_drag_fine")),
        ZoneType::ValueDragCoarse     => VVal::vec1(VVal::new_str("value_drag_coarse")),
        ZoneType::TextInput           => VVal::vec1(VVal::new_str("text_input")),
        ZoneType::Keyboard            => VVal::vec1(VVal::new_str("keyboard")),
        ZoneType::AtomClick { .. }    => VVal::vec1(VVal::new_str("atom_click")),
        ZoneType::HexFieldClick { pos, tile_size, ..  } => {
            VVal::vec3(
                VVal::new_str("hex_field_click"),
                VVal::ivec2(pos.0 as i64, pos.1 as i64),
                VVal::Flt(tile_size))
        },
        ZoneType::Click { index } => {
            VVal::vec2(VVal::new_str("click"), VVal::Int(index as i64))
        },
        ZoneType::Drag { index } => {
            VVal::vec2(VVal::new_str("click"), VVal::Int(index as i64))
        },
    }
}

fn active_zone2vval(z: &ActiveZone) -> wlambda::VVal {
    VVal::map3(
        "id",   id_idx2vval(z.id, z.dbgid),
        "pos",  VVal::fvec4(z.pos.x, z.pos.y, z.pos.w, z.pos.h),
        "zone", zone_type2vval(z.zone_type))
}

fn id_idx2vval(id: AtomId, idx: usize) -> wlambda::VVal {
    if idx > 0xFFFF {
        let (idx, x, y) = dbgid_unpack(idx);

        VVal::vec3(
            VVal::pair(
                VVal::Int(id.node_id() as i64),
                VVal::Int(id.atom_id() as i64)),
            VVal::new_str(dbgid2str(idx)),
            VVal::ivec2(x as i64, y as i64))
    } else {
        VVal::vec2(
            VVal::pair(
                VVal::Int(id.node_id() as i64),
                VVal::Int(id.atom_id() as i64)),
            VVal::new_str(dbgid2str(idx)))
    }
}

fn str2key(s: &str) -> Result<Key, StackAction> {
    if s.len() == 1 {
        Ok(Key::Character(s.to_string()))

    } else {
        match s {
            "Alt"        => Ok(Key::Alt),
            "Control"    => Ok(Key::Control),
            "Shift"      => Ok(Key::Shift),
            "Enter"      => Ok(Key::Enter),
            "Tab"        => Ok(Key::Tab),
            "Home"       => Ok(Key::Home),
            "Escape"     => Ok(Key::Escape),
            "Delete"     => Ok(Key::Delete),
            "Backspace"  => Ok(Key::Backspace),
            "PageUp"     => Ok(Key::PageUp),
            "PageDown"   => Ok(Key::PageDown),
            "ArrowUp"    => Ok(Key::ArrowUp),
            "ArrowDown"  => Ok(Key::ArrowDown),
            "ArrowLeft"  => Ok(Key::ArrowLeft),
            "ArrowRight" => Ok(Key::ArrowRight),
            "F1"         => Ok(Key::F1),
            "F2"         => Ok(Key::F2),
            "F3"         => Ok(Key::F3),
            "F4"         => Ok(Key::F4),
            "F5"         => Ok(Key::F5),
            "F6"         => Ok(Key::F6),
            "F7"         => Ok(Key::F7),
            "F8"         => Ok(Key::F8),
            "F9"         => Ok(Key::F9),
            "F10"        => Ok(Key::F10),
            "F11"        => Ok(Key::F11),
            "F12"        => Ok(Key::F12),
            _ => Err(
                StackAction::panic_msg(format!(
                    "Unknown key: '{}'", s)))
        }
    }
}

fn new_pattern_obj(pat: Arc<Mutex<PatternData>>) -> VVal {
    let obj = VVal::map();

    set_vval_method!(obj, pat, get_cell, Some(2), Some(2), env, _argc, {
        let (row, col) = (
            env.arg(0).i() as usize,
            env.arg(1).i() as usize,
        );
        if let Some(cell) = pat.lock().unwrap().get_cell(row, col) {
            Ok(VVal::new_str(cell))
        } else {
            Ok(VVal::None)
        }
    });

    set_vval_method!(obj, pat, is_col_note, Some(1), Some(1), env, _argc, {
        let col = env.arg(0).i() as usize;
        Ok(VVal::Bol(pat.lock().unwrap().is_col_note(col)))
    });

    set_vval_method!(obj, pat, is_col_step, Some(1), Some(1), env, _argc, {
        let col = env.arg(0).i() as usize;
        Ok(VVal::Bol(pat.lock().unwrap().is_col_step(col)))
    });

    set_vval_method!(obj, pat, is_col_gate, Some(1), Some(1), env, _argc, {
        let col = env.arg(0).i() as usize;
        Ok(VVal::Bol(pat.lock().unwrap().is_col_gate(col)))
    });

    set_vval_method!(obj, pat, rows, Some(0), Some(0), _env, _argc, {
        Ok(VVal::Int(pat.lock().unwrap().rows() as i64))
    });

    set_vval_method!(obj, pat, cols, Some(0), Some(0), _env, _argc, {
        Ok(VVal::Int(pat.lock().unwrap().cols() as i64))
    });

    set_vval_method!(obj, pat, set_rows, Some(1), Some(1), env, _argc, {
        pat.lock().unwrap().set_rows(env.arg(0).i() as usize);
        Ok(VVal::None)
    });

    set_vval_method!(obj, pat, clear_cell, Some(2), Some(2), env, _argc, {
        let (row, col) = (
            env.arg(0).i() as usize,
            env.arg(1).i() as usize,
        );
        pat.lock().unwrap().clear_cell(row, col);
        Ok(VVal::None)
    });

    set_vval_method!(obj, pat, set_col_note_type, Some(1), Some(1), env, _argc, {
        let col = env.arg(0).i() as usize;
        pat.lock().unwrap().set_col_note_type(col);
        Ok(VVal::None)
    });

    set_vval_method!(obj, pat, set_col_step_type, Some(1), Some(1), env, _argc, {
        let col = env.arg(0).i() as usize;
        pat.lock().unwrap().set_col_step_type(col);
        Ok(VVal::None)
    });

    set_vval_method!(obj, pat, set_col_value_type, Some(1), Some(1), env, _argc, {
        let col = env.arg(0).i() as usize;
        pat.lock().unwrap().set_col_value_type(col);
        Ok(VVal::None)
    });

    set_vval_method!(obj, pat, set_col_gate_type, Some(1), Some(1), env, _argc, {
        let col = env.arg(0).i() as usize;
        pat.lock().unwrap().set_col_gate_type(col);
        Ok(VVal::None)
    });

    set_vval_method!(obj, pat, set_cell_value, Some(3), Some(3), env, _argc, {
        let (row, col) = (
            env.arg(0).i() as usize,
            env.arg(1).i() as usize,
        );
        let value = env.arg(2).i() as u16;
        pat.lock().unwrap().set_cell_value(row, col, value);
        Ok(VVal::None)
    });

    set_vval_method!(obj, pat, get_cell_value, Some(2), Some(2), env, _argc, {
        let (row, col) = (
            env.arg(0).i() as usize,
            env.arg(1).i() as usize,
        );
        Ok(VVal::Int(pat.lock().unwrap().get_cell_value(row, col) as i64))
    });

    set_vval_method!(obj, pat, set_cursor, Some(2), Some(2), env, _argc, {
        let (row, col) = (
            env.arg(0).i() as usize,
            env.arg(1).i() as usize,
        );
        pat.lock().unwrap().set_cursor(row, col);
        Ok(VVal::None)
    });

    set_vval_method!(obj, pat, get_cursor, Some(0), Some(0), env, _argc, {
        let cur = pat.lock().unwrap().get_cursor();
        Ok(VVal::ivec2(cur.0 as i64, cur.1 as i64))
    });

    obj
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

fn cell_port2vval(cell: Cell, dir: CellDir) -> VVal {
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

fn setup_hx_module() -> wlambda::SymbolTable {
    let mut st = wlambda::SymbolTable::new();

    st.fun(
        "query_state", |env: &mut Env, argc: usize| {
        env.with_user_do(|ctx: &mut Ctx| {
            ctx.drv.query_state();
            Ok(VVal::None)
        })
    }, Some(0), Some(0), false);

    st.fun(
        "hover", |env: &mut Env, argc: usize| {
        env.with_user_do(|ctx: &mut Ctx| {
            if let Some(hz) = ctx.drv.hover {
                Ok(active_zone2vval(&hz))
            } else {
                Ok(VVal::None)
            }
        })
    }, Some(0), Some(0), false);

    st.fun(
        "zones", |env: &mut Env, argc: usize| {
        env.with_user_do(|ctx: &mut Ctx| {
            let ret = VVal::vec();
            for z in ctx.drv.zones.iter() {
                ret.push(active_zone2vval(z));
            }

            Ok(ret)
        })
    }, Some(0), Some(0), false);

    st.fun(
        "pattern_data_for_tracker", |env: &mut Env, _argc: usize| {
        let tracker_id = env.arg(0).i();

        env.with_user_do(|ctx: &mut Ctx| {
            let m = ctx.matrix.lock().unwrap();
            if let Some(pat) = m.get_pattern_data(tracker_id as usize) {
                Ok(new_pattern_obj(pat))
            } else {
                Err(StackAction::panic_msg(
                    format!("No data for tracker_id={}", tracker_id)))
            }
        })
    }, Some(1), Some(1), false);

    st.fun(
        "id_by_text", |env: &mut Env, argc: usize| {
        let needle = env.arg(0).s_raw();

        env.with_user_do(|ctx: &mut Ctx| {
            let ret = VVal::vec();

            for ((id, idx), (s, pos)) in ctx.drv.texts.iter() {
                if *s == needle {
                    ret.push(
                        VVal::vec3(
                            id_idx2vval(*id, *idx),
                            VVal::fvec4(pos.x, pos.y, pos.w, pos.h),
                            VVal::new_str(s)));
                }
            }

            Ok(if ret.len() == 0 { VVal::None } else { ret })
        })
    }, Some(1), Some(1), false);


    st.fun(
        "id_by_text_contains", |env: &mut Env, argc: usize| {
        let needle = env.arg(0).s_raw();

        env.with_user_do(|ctx: &mut Ctx| {
            let ret = VVal::vec();

            for ((id, idx), (s, pos)) in ctx.drv.texts.iter() {
                if let Some(_) = s.find(&needle) {
                    ret.push(
                        VVal::vec3(
                            id_idx2vval(*id, *idx),
                            VVal::fvec4(pos.x, pos.y, pos.w, pos.h),
                            VVal::new_str(s)));
                }
            }

            Ok(if ret.len() == 0 { VVal::None } else { ret })
        })
    }, Some(1), Some(1), false);

    st.fun(
        "mouse_move", |env: &mut Env, argc: usize| {
        let (x, y) =
            if argc == 1 { (env.arg(0).v_f(0), env.arg(0).v_f(1)) }
            else         { (env.arg(0).f(),    env.arg(1).f()) };

        env.with_user_do(|ctx: &mut Ctx| {
            match ctx.drv.move_mouse(x, y) {
                Ok(_)  => Ok(VVal::None),
                Err(e) => Err(
                    StackAction::panic_msg(format!(
                        "Driver error: {:?}", e)))
            }
        })
    }, Some(1), Some(2), false);

    st.fun(
        "mouse_down", |env: &mut Env, argc: usize| {
        let btn =
            env.arg(0).with_s_ref(|s| {
                match s {
                    "left"      => Ok(MButton::Left),
                    "right"     => Ok(MButton::Right),
                    "middle"    => Ok(MButton::Middle),
                    _ => Err(
                        StackAction::panic_msg(format!(
                            "Unknown button: '{}'", s)))
                }
            })?;
        env.with_user_do(|ctx: &mut Ctx| {
            match ctx.drv.mouse_down(btn) {
                Ok(_)  => Ok(VVal::None),
                Err(e) => Err(
                    StackAction::panic_msg(format!(
                        "Driver error: {:?}", e)))
            }
        })
    }, Some(1), Some(1), false);

    st.fun(
        "mouse_up", |env: &mut Env, argc: usize| {
        let btn =
            env.arg(0).with_s_ref(|s| {
                match s {
                    "left"      => Ok(MButton::Left),
                    "right"     => Ok(MButton::Right),
                    "middle"    => Ok(MButton::Middle),
                    _ => Err(
                        StackAction::panic_msg(format!(
                            "Unknown button: '{}'", s)))
                }
            })?;
        env.with_user_do(|ctx: &mut Ctx| {
            match ctx.drv.mouse_up(btn) {
                Ok(_)  => Ok(VVal::None),
                Err(e) => Err(
                    StackAction::panic_msg(format!(
                        "Driver error: {:?}", e)))
            }
        })
    }, Some(1), Some(1), false);

    st.fun(
        "key_down", |env: &mut Env, argc: usize| {
        let key = env.arg(0).with_s_ref(str2key)?;
        env.with_user_do(|ctx: &mut Ctx| {
            match ctx.drv.key_down(key.clone()) {
                Ok(_)  => Ok(VVal::None),
                Err(e) => Err(
                    StackAction::panic_msg(format!(
                        "Driver error: {:?}", e)))
            }
        })
    }, Some(1), Some(1), false);

    st.fun(
        "key_up", |env: &mut Env, argc: usize| {
        let key = env.arg(0).with_s_ref(str2key)?;
        env.with_user_do(|ctx: &mut Ctx| {
            match ctx.drv.key_up(key.clone()) {
                Ok(_)  => Ok(VVal::None),
                Err(e) => Err(
                    StackAction::panic_msg(format!(
                        "Driver error: {:?}", e)))
            }
        })
    }, Some(1), Some(1), false);

    st.fun(
        "param_id", |env: &mut Env, argc: usize| {
        let node_id = env.arg(0).with_s_ref(|s| NodeId::from_str(s));
        let node_id = node_id.to_instance(env.arg(1).i() as usize);

        let uniq_idx =
            env.with_user_do(|ctx: &mut Ctx| {
                let m = ctx.matrix.lock().unwrap();
                m.unique_index_for(&node_id)
            });

        if let Some(uniq_idx) = uniq_idx {
            if let Some(param_id) =
                env.arg(2).with_s_ref(|s| node_id.inp_param(s))
            {
                Ok(VVal::pair(
                    VVal::Int(uniq_idx as i64),
                    VVal::Int(param_id.inp() as i64)))
            }
            else
            {
                Ok(VVal::None)
            }
        } else {
            Ok(VVal::None)
        }
    }, Some(3), Some(3), false);

    st.fun(
        "set_cell", |env: &mut Env, argc: usize| {
        let pos  = env.arg(0);
        let cell = env.arg(1);

        let cell_node_id = cell.v_k("node_id");
        let node_id =
            cell_node_id.v_(0).with_s_ref(|s| NodeId::from_str(s));
        let node_id =
            node_id.to_instance(cell_node_id.v_i(1) as usize);

        let mut m_cell = Cell::empty(node_id);

        let x = pos.v_ik("x") as usize;
        let y = pos.v_ik("y") as usize;

        let ports = cell.v_k("ports");

        cell_set_port(&mut m_cell, ports.v_(0), CellDir::T);
        cell_set_port(&mut m_cell, ports.v_(1), CellDir::TL);
        cell_set_port(&mut m_cell, ports.v_(2), CellDir::BL);
        cell_set_port(&mut m_cell, ports.v_(3), CellDir::TR);
        cell_set_port(&mut m_cell, ports.v_(4), CellDir::BR);
        cell_set_port(&mut m_cell, ports.v_(5), CellDir::B);

        env.with_user_do(|ctx: &mut Ctx| {
            let mut m = ctx.matrix.lock().unwrap();
            m.place(x, y, m_cell);
            let _ = m.sync();
        });

        Ok(VVal::None)
    }, Some(2), Some(2), false);

    st.fun(
        "matrix_generation", |env: &mut Env, argc: usize| {
        let pos = env.arg(0);
        env.with_user_do(|ctx: &mut Ctx| {
            let m = ctx.matrix.lock().unwrap();
            Ok(VVal::Int(m.get_generation() as i64))
        })
    }, Some(0), Some(0), false);

    st.fun(
        "get_cell", |env: &mut Env, argc: usize| {
        let pos = env.arg(0);
        env.with_user_do(|ctx: &mut Ctx| {
            let m = ctx.matrix.lock().unwrap();
            let cell = m.get_copy(pos.v_i(0) as usize, pos.v_i(1) as usize);

            if let Some(cell) = cell {
                let node_id = cell.node_id();

                let ports = VVal::vec();
                ports.push(cell_port2vval(cell, CellDir::T));
                ports.push(cell_port2vval(cell, CellDir::TL));
                ports.push(cell_port2vval(cell, CellDir::BL));
                ports.push(cell_port2vval(cell, CellDir::TR));
                ports.push(cell_port2vval(cell, CellDir::BR));
                ports.push(cell_port2vval(cell, CellDir::B));

                Ok(VVal::map3(
                    "node_id",
                    VVal::pair(
                        VVal::new_str(node_id.label()),
                        VVal::Int(node_id.instance() as i64)),
                    "pos",
                    VVal::ivec2(
                        cell.pos().0 as i64,
                        cell.pos().1 as i64),
                    "ports", ports))
            } else {
                Ok(VVal::None)
            }
        })
    }, Some(1), Some(1), false);

    st.fun(
        "mouse_pos", |env: &mut Env, argc: usize| {
        env.with_user_do(|ctx: &mut Ctx| {
            Ok(VVal::fvec2(ctx.drv.mouse_pos.0, ctx.drv.mouse_pos.1))
        })
    }, Some(0), Some(0), false);

    st
}

fn start_driver(matrix: Arc<Mutex<Matrix>>) -> Driver {
    let (mut driver, mut drv_frontend) = Driver::new();

    driver.take_control();

    std::thread::spawn(move || {
        std::thread::sleep(
            std::time::Duration::from_millis(1000));

        let clear_matrix = matrix.clone();

        let drvctx = Rc::new(RefCell::new(Ctx {
            drv: drv_frontend,
            matrix,
        }));

        let global_env = wlambda::GlobalEnv::new_default();
        global_env.borrow_mut().set_module("hx", setup_hx_module());

        let mut ctx =
            wlambda::EvalContext::new_with_user(
                global_env, drvctx.clone());


        let path = env!("CARGO_MANIFEST_DIR");

        let mut a = std::env::args();
        a.next();
        let test_match = a.next();

        let mut files : Vec<String> =
            std::fs::read_dir(path.to_string() + "/test_scripts/").unwrap()
            .map(|e| {
                let pth = e.unwrap().path();
                let path = pth.as_path();
                path.to_str().unwrap().to_string()
            }).collect();

        files.sort();

        let mut error = false;

        for f in files.iter() {
            let path = std::path::Path::new(f);
            let name = path.file_name().unwrap().to_str().unwrap();

            match ctx.eval_file(&f) {
                Ok(v) => {
                    println!("[{} has {} tests]", name, v.len());
                    for (v, _) in v.iter() {
                        {
                            let mut m = clear_matrix.lock().unwrap();
                            m.clear();
                        }

                        let tname = v.v_s_raw(0);
                        let fun   = v.v_(1);
                        let mut print_name_before = false;

                        let combined = name.to_string() + "_" + &tname;

                        let exec_test =
                            if let Some(name_substr) = &test_match {
                                if name_substr == "#" {
                                    print_name_before = true;
                                    true
                                } else {
                                    combined.contains(name_substr)
                                }
                            } else {
                                true
                            };

                        if exec_test {
                            if print_name_before {
                                println!("    ({})", tname);
                            }

                            match ctx.call(&fun, &[]) {
                                Ok(v) => {
                                    println!("    - OK: {}", tname);
                                },
                                Err(e) => {
                                    println!("*** ERROR: {}\n    {}", combined, e);
                                    error = true;
                                    break;
                                }
                            }
                        }
                    }

                    if error {
                        break;
                    }
                },
                Err(e) => {
                    println!("*** ERROR: {}\n    {}", name, e);
                    error = true;
                    break;
                },
            }
        }

        if error {
            let mut rl = rustyline::Editor::<()>::new();
            if rl.load_history("gui_wlambda.history").is_ok() {
                println!("Loaded history from 'gui_wlambda.history' file.");
            }

            drvctx.borrow_mut().drv.be_quiet();

            eprintln!("HexoSynth Version {}", VERSION);
            loop {
                let readline = rl.readline(">> ");
                match readline {
                    Ok(line) => {
                        rl.add_history_entry(line.as_str());

                        match ctx.eval(&line) {
                            Ok(v)  => {
                                println!("> {}", v.s());
                                ctx.set_global_var("@@", &v);
                            },
                            Err(e) => { println!("*** {}", e); }
                        }
                    },
                    Err(_) => { break; },
                }
            }
            if rl.save_history("gui_wlambda.history").is_ok() {
                println!("Saved history to 'gui_wlambda.history'");
            }
        }

        drvctx.borrow_mut().drv.exit();
    });

    driver
}

fn main() {
    use hexotk::widgets::{Dialog, DialogData, DialogModel};

    let (matrix, node_exec) = init_hexosynth();
    let matrix = Arc::new(Mutex::new(matrix));

    start_backend(node_exec);
    let drv = start_driver(matrix.clone());

    open_hexosynth(None, Some(drv), matrix);
}
