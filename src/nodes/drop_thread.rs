// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use super::DropMsg;

use ringbuf::Consumer;

/// For receiving deleted/overwritten nodes from the backend
/// thread and dropping them.
pub(crate) struct DropThread {
    terminate: std::sync::Arc<std::sync::atomic::AtomicBool>,
    th:        Option<std::thread::JoinHandle<()>>,
}

impl DropThread {
    pub(crate) fn new(mut graph_drop_con: Consumer<DropMsg>) -> Self {
        let terminate =
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let th_terminate = terminate.clone();

        let th = std::thread::spawn(move || {
            loop {
                if th_terminate.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }

                while let Some(_node) = graph_drop_con.pop() {
                    // drop it ...
                    println!("Dropped some shit...");
                }

                std::thread::sleep(std::time::Duration::from_millis(250));
            }
        });

        Self {
            th: Some(th),
            terminate,
        }
    }
}

impl Drop for DropThread {
    fn drop(&mut self) {
        self.terminate.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = self.th.take().unwrap().join();
    }
}
