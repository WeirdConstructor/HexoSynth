use hexosynth::*;
use hexosynth::matrix::*;
use hexosynth::nodes::new_node_engine;
use hexosynth::dsp::*;

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

fn setup_environment() -> wlambda::GlobalEnvRef {
    use wlambda::{Env, VVal};
    let global_env = wlambda::GlobalEnv::new_default();

    global_env.borrow_mut().add_func(
        "query_state", |env: &mut Env, argc: usize| {
        env.with_user_do(|ctx: &mut Ctx| {
            ctx.drv.query_state();
            Ok(VVal::None)
        })
    }, Some(0), Some(0));

    global_env
}

fn start_driver(matrix: Arc<Mutex<Matrix>>) -> Driver {
    let (driver, mut drv_frontend) = Driver::new();

    std::thread::spawn(move || {
        use hexotk::constants::*;

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
