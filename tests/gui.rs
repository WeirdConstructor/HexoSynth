use hexosynth::*;
use hexosynth::matrix::*;
use hexosynth::nodes::new_node_engine;
use hexosynth::dsp::*;

use hexotk::constants::{dbgid2str, str2dbgid, dbgid_unpack};

use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

use wlambda;
use wlambda::vval::VVal;

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

fn setup_environment() -> wlambda::GlobalEnvRef {
    use wlambda::{Env, VVal};
    let global_env = wlambda::GlobalEnv::new_default();

    global_env.borrow_mut().add_func(
        "query_state", |env: &mut Env, argc: usize| {
        env.with_user_do(|ctx: &mut Ctx| {
            ctx.drv.query_state();
//            println!("ZONES: {:#?}", ctx.drv.zones);
//            println!("TEXTS: {:?}", ctx.drv.texts);
            Ok(VVal::None)
        })
    }, Some(0), Some(0));

    global_env.borrow_mut().add_func(
        "hover", |env: &mut Env, argc: usize| {
        env.with_user_do(|ctx: &mut Ctx| {
            if let Some(hz) = ctx.drv.hover {
                Ok(active_zone2vval(&hz))
            } else {
                Ok(VVal::None)
            }
        })
    }, Some(0), Some(0));

    global_env.borrow_mut().add_func(
        "zones", |env: &mut Env, argc: usize| {
        env.with_user_do(|ctx: &mut Ctx| {
            let ret = VVal::vec();
            for z in ctx.drv.zones.iter() {
                ret.push(active_zone2vval(z));
            }

            Ok(ret)
        })
    }, Some(0), Some(0));


    global_env.borrow_mut().add_func(
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
    }, Some(1), Some(1));

    global_env.borrow_mut().add_func(
        "mouse_move", |env: &mut Env, argc: usize| {
        let (x, y) = (
            env.arg(0).v_f(0),
            env.arg(0).v_f(1)
        );
        env.with_user_do(|ctx: &mut Ctx| {
            ctx.drv.move_mouse(x, y);
            Ok(VVal::None)
        })
    }, Some(1), Some(1));

    global_env.borrow_mut().add_func(
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
    }, Some(3), Some(3));

    global_env.borrow_mut().add_func(
        "mouse_pos", |env: &mut Env, argc: usize| {
        env.with_user_do(|ctx: &mut Ctx| {
            Ok(VVal::pair(
                VVal::Flt(ctx.drv.mouse_pos.0),
                VVal::Flt(ctx.drv.mouse_pos.1)))
        })
    }, Some(0), Some(0));

    global_env
}

fn start_driver(matrix: Arc<Mutex<Matrix>>) -> Driver {
    let (driver, mut drv_frontend) = Driver::new();

    std::thread::spawn(move || {
        use hexotk::constants::*;

        std::thread::sleep(
            std::time::Duration::from_millis(1000));

        let drvctx = Rc::new(RefCell::new(Ctx {
            drv: drv_frontend,
            matrix,
        }));

        let mut ctx =
            wlambda::EvalContext::new_with_user(
                setup_environment(),
                drvctx.clone());


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
