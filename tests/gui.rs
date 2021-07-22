use hexosynth::*;
use hexosynth::dsp::*;
use hexosynth::dsp::tracker::UIPatternModel;

use hexodsp::dsp::tracker::PatternData;

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

fn start_backend(shared: Arc<HexoSynthShared>) {
    let node_exec = shared.node_exec.borrow_mut().take().unwrap();
    let ne        = Arc::new(Mutex::new(node_exec));
    let ne2       = ne.clone();

    let mut in_a  = [0.0; hexodsp::dsp::MAX_BLOCK_SIZE];
    let mut in_b  = [0.0; hexodsp::dsp::MAX_BLOCK_SIZE];
    let mut out_a = [0.0; hexodsp::dsp::MAX_BLOCK_SIZE];
    let mut out_b = [0.0; hexodsp::dsp::MAX_BLOCK_SIZE];

    let us_per_frame =
        (1000000.0 * (hexodsp::dsp::MAX_BLOCK_SIZE as f32)) / 44100.0;

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
            let mut us_remaining = us_per_frame as u128 - us_passed;

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
        ZoneType::HexFieldClick { pos, ..  } => {
            VVal::vec2(
                VVal::new_str("hex_field_click"),
                VVal::ivec2(pos.0 as i64, pos.1 as i64))
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


fn setup_hx_module() -> wlambda::SymbolTable {
    let mut st = wlambda::SymbolTable::new();

    st.fun(
        "query_state", |env: &mut Env, argc: usize| {
        env.with_user_do(|ctx: &mut Ctx| {
            ctx.drv.query_state();
//            println!("ZONES: {:#?}", ctx.drv.zones);
//            println!("TEXTS: {:?}", ctx.drv.texts);
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
        "mouse_pos", |env: &mut Env, argc: usize| {
        env.with_user_do(|ctx: &mut Ctx| {
            Ok(VVal::pair(
                VVal::Flt(ctx.drv.mouse_pos.0),
                VVal::Flt(ctx.drv.mouse_pos.1)))
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

        let mut files : Vec<String> =
            std::fs::read_dir(path.to_string() + "/tests/gui/").unwrap().map(|e| {
                e.unwrap().path().as_path().to_str().unwrap().to_string()
            }).collect();

        files.sort();

        for f in files.iter() {
            let path = std::path::Path::new(f);
            let name = path.file_name().unwrap().to_str().unwrap();

            match ctx.eval_file(&f) {
                Ok(v) => {
                    println!("*** OK: {}", name);
                },
                Err(e) => {
                    println!("*** ERROR: {}\n    {}", name, e);
                },
            }
        }

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

        drvctx.borrow_mut().drv.exit();

//            {
//                let mut m = matrix.lock().unwrap();
//                m.place(3, 3, Cell::empty(NodeId::TSeq(0)));
//                m.sync();
//            }
//            std::thread::sleep(
//                std::time::Duration::from_millis(1000));

//                println!("{:#?}", drv_frontend.get_text_dump());

//                    println!("ZONES: {:#?}",
//                        drv_frontend.query_zones(
//                            6.into()).unwrap());

//                    let pos =
//                        drv_frontend.get_zone_pos(
//                            6.into(), DBGID_KNOB_FINE)
//                        .unwrap();

//            drv_frontend.move_mouse(142.0, 49.0);
//            drv_frontend.query_state();
//            println!("mp: {:?}", drv_frontend.mouse_pos);

//                let z = drv_frontend.query_hover().unwrap().unwrap();
//                println!("z: {:#?}", z);

//                assert_eq!(
//                    drv_frontend.get_text(
//                        z.id, DBGID_KNOB_NAME).unwrap(),
//                    "det");
//        }
    });

    driver
}

#[test]
fn main_gui() {
    use hexotk::widgets::{Dialog, DialogData, DialogModel};

    println!("INIT");
    let shared = Arc::new(HexoSynthShared::new());

    start_backend(shared.clone());
    let matrix = shared.matrix.clone();

    let drv = start_driver(matrix.clone());

    open_hexosynth(None, Some(drv), matrix);
}
