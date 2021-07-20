use hexosynth::*;
use hexosynth::matrix::*;
use hexosynth::nodes::new_node_engine;
use hexosynth::dsp::*;

use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

fn start_backend<F: FnMut()>(shared: Arc<HexoSynthShared>, mut f: F) {
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

    f();
}

#[test]
fn main_gui() {
    use hexotk::widgets::{Dialog, DialogData, DialogModel};

    let shared = Arc::new(HexoSynthShared::new());

    start_backend(shared.clone(), move || {
        let matrix = shared.matrix.clone();

        open_window("HexoTK Standalone", 1400, 700, None, Box::new(move || {
            use crate::ui::matrix::NodeMatrixData;

            let dialog_model = Rc::new(RefCell::new(DialogModel::new()));
            let wt_diag      = Rc::new(Dialog::new());

            let ui_ctrl = UICtrlRef::new(matrix.clone(), dialog_model.clone());

            let (drv, drv_frontend) = Driver::new();

            std::thread::spawn(move || {
                use hexotk::constants::*;
                loop {
                    std::thread::sleep(
                        std::time::Duration::from_millis(1000));

                    println!("{:#?}", drv_frontend.get_text_dump());

//                    println!("ZONES: {:#?}",
//                        drv_frontend.query_zones(
//                            6.into()).unwrap());

//                    let pos =
//                        drv_frontend.get_zone_pos(
//                            6.into(), DBGID_KNOB_FINE)
//                        .unwrap();

                    drv_frontend.move_mouse(142.0, 49.0);

                    let z = drv_frontend.query_hover().unwrap().unwrap();
                    println!("z: {:#?}", z);

                    assert_eq!(
                        drv_frontend.get_text(
                            z.id, DBGID_KNOB_NAME).unwrap(),
                        "det");
                }
            });

            (drv, Box::new(UI::new(
                Box::new(NodeMatrixData::new(
                    ui_ctrl.clone(),
                    UIPos::center(12, 12),
                    110003)),
                Box::new(wbox!(
                    wt_diag, 90000.into(), center(12, 12),
                    DialogData::new(
                        DIALOG_ID,
                        AtomId::new(DIALOG_ID, DIALOG_OK_ID),
                        dialog_model.clone()))),
                Box::new(UIParams::new(ui_ctrl)),
                (1400 as f64, 700 as f64),
            )))
        }));
    });
}
