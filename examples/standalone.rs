// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use hexotk::*;
//use hexotk::widgets::*;
use hexosynth::ui::matrix::NodeMatrixData;
use hexosynth::*;
use hexosynth::nodes::NodeExecutor;

use std::sync::Arc;
use std::sync::Mutex;

struct Notifications {
    node_exec: Arc<Mutex<NodeExecutor>>,
}

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn buffer_size(&mut self, _: &jack::Client, sz: jack::Frames) -> jack::Control {
        println!("JACK: buffer size changed to {}", sz);
        jack::Control::Continue
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", srate);
        let mut ne = self.node_exec.lock().unwrap();
        ne.set_sample_rate(srate as f32);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, client: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        if let Some(p) = client.port_by_id(port_id) {
            if let Ok(name) = p.name() {
                println!("JACK: port registered: {}", name);
//                if name == "HexoSynth:hexosynth_out1" {
//                    client.connect_ports_by_name(&name, "system:playback_1");
//                } else if name == "HexoSynth:hexosynth_out2" {
//                    client.connect_ports_by_name(&name, "system:playback_2");
//                }
            }
        }
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        println!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }

    fn latency(&mut self, _: &jack::Client, mode: jack::LatencyType) {
        println!(
            "JACK: {} latency has changed",
            match mode {
                jack::LatencyType::Capture => "capture",
                jack::LatencyType::Playback => "playback",
            }
        );
    }
}


fn start_backend<F: FnMut()>(shared: Arc<HexoSynthShared>, mut f: F) {
    let (client, _status) =
        jack::Client::new("HexoSynth", jack::ClientOptions::NO_START_SERVER)
        .unwrap();

    let in_a =
        client.register_port("hexosynth_in1", jack::AudioIn::default())
            .unwrap();
    let in_b =
        client.register_port("hexosynth_in2", jack::AudioIn::default())
            .unwrap();
    let mut out_a =
        client.register_port("hexosynth_out1", jack::AudioOut::default())
            .unwrap();
    let mut out_b =
        client.register_port("hexosynth_out2", jack::AudioOut::default())
            .unwrap();

    let node_exec = shared.node_exec.borrow_mut().take().unwrap();
    let ne        = Arc::new(Mutex::new(node_exec));
    let ne2       = ne.clone();

    let oversample_simulation =
        if let Some(arg) = std::env::args().skip(1).next() {
            arg == "4x"
        } else {
            false
        };

    let mut first = true;
    let process_callback = move |client: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let out_a_p = out_a.as_mut_slice(ps);
        let out_b_p = out_b.as_mut_slice(ps);
        let in_a_p = in_a.as_slice(ps);
        let in_b_p = in_b.as_slice(ps);

        if first {
            client.connect_ports_by_name("HexoSynth:hexosynth_out1", "system:playback_1")
                .expect("jack connect ports works");
            client.connect_ports_by_name("HexoSynth:hexosynth_out2", "system:playback_2")
                .expect("jack connect ports works");
            first = false;
        }

        let nframes = out_a_p.len();

        let mut node_exec = ne.lock().unwrap();

        node_exec.process_graph_updates();

        let mut frames_left = nframes;
        let mut offs        = 0;

        while frames_left > 0 {
            let cur_nframes =
                if frames_left >= hexosynth::dsp::MAX_BLOCK_SIZE {
                    hexosynth::dsp::MAX_BLOCK_SIZE
                } else {
                    frames_left
                };

            frames_left -= cur_nframes;

            let output = &mut [&mut out_a_p[offs..(offs + cur_nframes)],
                               &mut out_b_p[offs..(offs + cur_nframes)]];
            let input =
                &[&in_a_p[offs..(offs + cur_nframes)],
                  &in_b_p[offs..(offs + cur_nframes)]];

            let mut context =
                Context {
                    nframes: cur_nframes,
                    output,
                    input,
                };

            for i in 0..context.nframes {
                context.output[0][i] = 0.0;
                context.output[1][i] = 0.0;
            }

            node_exec.process(&mut context);

            if oversample_simulation {
                node_exec.process(&mut context);
                node_exec.process(&mut context);
                node_exec.process(&mut context);
            }

            offs += cur_nframes;
        }

        jack::Control::Continue
    };

    let process =
        jack::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client =
        client.activate_async(Notifications {
            node_exec: ne2,
        }, process).unwrap();

    f();

    active_client.deactivate().unwrap();
}

fn main() {
    let shared = Arc::new(HexoSynthShared::new());

    start_backend(shared.clone(), move || {
        let matrix = shared.matrix.clone();

        open_window("HexoTK Standalone", 1400, 700, None, Box::new(|| {
            Box::new(UI::new(
                Box::new(NodeMatrixData::new(matrix.clone(), UIPos::center(12, 12), 11)),
                Box::new(HexoSynthUIParams::new(matrix)),
                (1400 as f64, 700 as f64),
            ))
        }));
    });
}
