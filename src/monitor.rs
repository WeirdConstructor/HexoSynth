use crate::dsp::MAX_BLOCK_SIZE;
use ringbuf::{RingBuffer, Producer, Consumer};

use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::thread::JoinHandle;

/// 3 inputs, 3 outputs of signal monitors.
pub const MON_SIG_CNT : usize = 6;

/// Just some base to determine the monitor buffer sizes.
const IMAGINARY_MAX_SAMPLE_RATE : usize = 48000;

/// The number of minmax samples to hold.
const MONITOR_MINMAX_SAMPLES : usize = 128;

/// The length in seconds of the MONITOR_MINMAX_SAMPLES
const MONITOR_MINMAX_LEN_S   : usize = 2;

/// The sleep time of the thread that receives monitoring data
/// from the backend/audio thread.
const MONITOR_PROC_THREAD_INTERVAL_MS : u64 = 10;

// TODO / FIXME: We should recalculate this on the basis of the
// real actual sample rate, otherwise the monitor scope
// is going to be too fast.
/// The number of audio samples over which to calculate
/// one min/max sample. Typically something around 750.
const MONITOR_INPUT_LEN_PER_SAMPLE : usize =
    (MONITOR_MINMAX_LEN_S * IMAGINARY_MAX_SAMPLE_RATE)
    / MONITOR_MINMAX_SAMPLES;

/// Maximum number of monitor buffers to hold in the backend.
/// Typically there are only 16-32ms of monitor content floating
/// around, as the monitor processing thread regularily
/// processes the monitor.
const MONITOR_BUF_COUNT  : usize =
//  2 for safety margin
    2 * (IMAGINARY_MAX_SAMPLE_RATE / MAX_BLOCK_SIZE);

pub struct MonitorBackend {
    rb_mon_prod:              Producer<MonitorBufPtr>,
    rb_recycle_con:          Consumer<MonitorBufPtr>,

    /// Holds enough monitor buffers to hold about 1-2 seconds
    /// of data. The [MonitorBuf] boxes are written in the
    /// backend and then sent via [monitor_prod] to the frontend.
    /// The frontend then sends the used [MonitorBufPtr] back
    /// via quick_update_con.
    unused_monitor_buffers: Vec<MonitorBufPtr>,
}

impl MonitorBackend {
    /// Checks if there are any used monitor buffers to be
    /// collected.
    pub fn check_recycle(&mut self) {
        while let Some(buf) = self.rb_recycle_con.pop() {
            self.unused_monitor_buffers.push(buf);
        }
    }

    /// Hands out an unused [MonitorBuf] for filling and
    /// sending to the [MonitorProcessor] thread.
    pub fn get_unused_mon_buf(&mut self) -> Option<MonitorBufPtr> {
        self.unused_monitor_buffers.pop()
    }

    /// A helper function for writing tests.
    /// Returns the number of [MonitorBuf] we can hand out
    /// until there are none anymore.
    pub fn count_unused_mon_bufs(&self) -> usize {
        self.unused_monitor_buffers.len()
    }

    /// Sends a [MonitorBuf] to the [MonitorProcessor].
    pub fn send_mon_buf(&mut self, buf: MonitorBufPtr) {
        match self.rb_mon_prod.push(buf) {
            Ok(_)    => (),
            Err(buf) => self.unused_monitor_buffers.push(buf),
        }
    }
}

/// Implements the logic for min/maxing a single signal channel/line.
pub struct MonitorMinMax {
    /// Index of the signal in the [MonitorBuf]
    sig_idx:        usize,

    /// A ring buffer of min/max samples, written to by `buf_write_ptr`.
    buf:            [(f32, f32); MONITOR_MINMAX_SAMPLES],

    /// The pointer/index into `buf` to the next update to write.
    buf_write_ptr:  usize,

    /// Holds the currently accumulated min/max values and the length
    /// of so far processed audio rate samples. Once MONITOR_INPUT_LEN_PER_SAMPLE
    /// is reached, this will be written into `buf`.
    cur_min_max:    (f32, f32, usize),
}

impl MonitorMinMax {
    pub fn new(sig_idx: usize) -> Self {
        Self {
            sig_idx,
            buf:           [(0.0, 0.0); MONITOR_MINMAX_SAMPLES],
            buf_write_ptr: 0,
            cur_min_max:   (100.0, -100.0, 0),
        }
    }

    /// Processes a monitoring buffer received from the Backend.
    /// It returns `true` when a new data point was calculated.
    pub fn process(&mut self, mon_buf: &mut MonitorBufPtr) -> bool {
        let mut new_data = false;

        while let Some(sample) =
            mon_buf.next_sample_for_signal(self.sig_idx)
        {
            self.cur_min_max.0 = self.cur_min_max.0.min(sample);
            self.cur_min_max.1 = self.cur_min_max.1.max(sample);
            self.cur_min_max.2 += 1;

            if self.cur_min_max.2 >= MONITOR_INPUT_LEN_PER_SAMPLE {
                self.buf[self.buf_write_ptr] = (
                    self.cur_min_max.0,
                    self.cur_min_max.1
                );
                new_data = true;

                self.buf_write_ptr = (self.buf_write_ptr + 1) % self.buf.len();

                self.cur_min_max.0 =  100.0;
                self.cur_min_max.1 = -100.0;
                self.cur_min_max.2 = 0;
            }
        }

        new_data
    }
}

/// Represents a bunch of min/max samples.
/// Usually copied from the MonitorProcessor thread
/// to the frontend if required.
#[derive(Debug, Clone, Copy)]
pub struct MinMaxMonitorSamples {
    samples: [(f32, f32); MONITOR_MINMAX_SAMPLES],
    buf_ptr: usize,
}

impl MinMaxMonitorSamples {
    pub fn new() -> Self {
        Self {
            samples: [(0.0, 0.0); MONITOR_MINMAX_SAMPLES],
            buf_ptr: 0,
        }
    }

    fn copy_from(&mut self, min_max_slice: (usize, &[(f32, f32)])) {
        self.samples.copy_from_slice(min_max_slice.1);
        self.buf_ptr = min_max_slice.0;
    }

    fn copy_to(&self, sms: &mut MinMaxMonitorSamples) {
        sms.buf_ptr = self.buf_ptr;
        sms.samples.copy_from_slice(&self.samples[..]);
    }

    /// Gets the sample at the offset relative to the
    pub fn at(&self, offs: usize) -> &(f32, f32) {
        let idx = (self.buf_ptr + offs) % self.samples.len();
        &self.samples[idx]
    }

    pub fn len(&self) -> usize { MONITOR_MINMAX_SAMPLES }
}

impl std::ops::Index<usize> for MinMaxMonitorSamples {
    type Output = (f32, f32);

    fn index(&self, idx: usize) -> &Self::Output {
        &self.at(idx)
    }
}

/// The actual frontend API for the MonitorProcessor.
/// We start an extra thread for handling monitored signals from the
/// MonitorBackend, because we can't guarantee that the UI thread
/// is actually started or working. Also because we want to be independent
/// of whether a UI is started at all.
///
/// Just call [Monitor::get_minmax_monitor_samples] and you will always get
/// the most current data.
pub struct Monitor {
    terminate_proc:         Arc<AtomicBool>,
    proc_thread:            Option<JoinHandle<()>>,

    new_data:               Arc<AtomicBool>,
    monitor_samples:        Arc<Mutex<[MinMaxMonitorSamples; MON_SIG_CNT]>>,
    monitor_samples_copy:   [MinMaxMonitorSamples; MON_SIG_CNT],
}

impl Monitor {
    pub fn new(rb_mon_con: Consumer<MonitorBufPtr>,
               rb_recycle_prod: Producer<MonitorBufPtr>)
        -> Self
    {
        let terminate_proc = Arc::new(AtomicBool::new(false));
        let th_terminate   = terminate_proc.clone();

        let monitor_samples =
            Arc::new(Mutex::new(
                [MinMaxMonitorSamples::new(); MON_SIG_CNT]));
        let th_mon_samples = monitor_samples.clone();

        let new_data       = Arc::new(AtomicBool::new(false));
        let th_new_data    = new_data.clone();

        let th = std::thread::spawn(move || {
            let mut proc = MonitorProcessor::new(rb_mon_con, rb_recycle_prod);

            loop {
                if th_terminate.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }

                //d// let ta = std::time::Instant::now();
                proc.process();

                if proc.check_new_data() {
                    let mut ms =
                        th_mon_samples.lock()
                           .expect("Unpoisoned Lock for monitor_samples");
                    for i in 0..MON_SIG_CNT {
                        ms[i].copy_from(proc.minmax_slice_for_signal(i));
                    }

                    th_new_data.store(true, std::sync::atomic::Ordering::Relaxed);

                    //d// let ta = std::time::Instant::now().duration_since(ta);
                    //d// println!("txx Elapsed: {:?}", ta);
                }


                std::thread::sleep(
                    std::time::Duration::from_millis(
                        MONITOR_PROC_THREAD_INTERVAL_MS));
            }
        });

        Self {
            proc_thread: Some(th),
            terminate_proc,
            monitor_samples,
            monitor_samples_copy: [MinMaxMonitorSamples::new(); MON_SIG_CNT],
            new_data,
        }
    }

    pub fn get_minmax_monitor_samples(&mut self, idx: usize) -> &MinMaxMonitorSamples {
        // TODO / FIXME: We should be using a triple buffer here
        // for access to the set of MinMaxMonitorSamples. But I was
        // too lazy and think we can bear with a slightly sluggish
        // UI. Anyways, if we get a sluggish UI, we have to look here.

        if self.new_data.load(std::sync::atomic::Ordering::Relaxed) {
            let ms =
                self.monitor_samples.lock()
                   .expect("Unpoisoned Lock for monitor_samples");

            for i in 0..MON_SIG_CNT {
                ms[i].copy_to(
                    &mut self.monitor_samples_copy[i]);
            }

            self.new_data.store(false, std::sync::atomic::Ordering::Relaxed);
        }

        &self.monitor_samples_copy[idx]
    }
}

impl Drop for Monitor {
    fn drop(&mut self) {
        self.terminate_proc.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = self.proc_thread.take().unwrap().join();
    }
}

/// Coordinates the processing of incoming MonitorBufs.
pub struct MonitorProcessor {
    rb_mon_con:             Consumer<MonitorBufPtr>,
    rb_recycle_prod:        Producer<MonitorBufPtr>,

    new_data: bool,

    procs: Vec<MonitorMinMax>,
}

impl MonitorProcessor {
    pub fn new(rb_mon_con: Consumer<MonitorBufPtr>,
               rb_recycle_prod: Producer<MonitorBufPtr>)
        -> Self
    {
        let mut procs = vec![];
        for i in 0..MON_SIG_CNT {
            procs.push(MonitorMinMax::new(i));
        }

        Self {
            rb_mon_con,
            rb_recycle_prod,
            procs,
            new_data: false,
        }
    }

    /// Helper function for tests, to access the current state of
    /// the min/max buffers.
    pub fn minmax_slice_for_signal(&self, idx: usize) -> (usize, &[(f32, f32)]) {
        let buf_ptr = self.procs[idx].buf_write_ptr;
        (buf_ptr, &self.procs[idx].buf[..])
    }

    /// Internal helper function for `process`.
    fn process_mon_buf(&mut self, mon_buf: &mut MonitorBufPtr) {
        for proc in self.procs.iter_mut() {
            if proc.process(mon_buf) {
                self.new_data = true;
            }
        }
    }

    /// Processes all queued [MonitorBuf] instances and sends
    /// then back to the [MonitorBackend] thread after
    /// used for recycling.
    pub fn process(&mut self) {
        while let Some(mut buf) = self.rb_mon_con.pop() {
            self.process_mon_buf(&mut buf);
            buf.reset();
            let _ = self.rb_recycle_prod.push(buf);
        }
    }

    /// Returns true, when a new data point was received.
    /// Resets the internal flag until the next time new data is received.
    pub fn check_new_data(&mut self) -> bool {
        let new_data = self.new_data;
        self.new_data = false;
        new_data
    }
}

/// Creates a pair of interconnected MonitorBackend and MonitorProcessor
/// instances, to be sent to different threads.
pub fn new_monitor_processor() -> (MonitorBackend, Monitor) {
    let rb_monitor  = RingBuffer::new(MONITOR_BUF_COUNT);
    let rb_recycle   = RingBuffer::new(MONITOR_BUF_COUNT);

    let (rb_mon_prod,     rb_mon_con)     = rb_monitor.split();
    let (rb_recycle_prod, rb_recycle_con) = rb_recycle.split();

    let mut unused_monitor_buffers = Vec::with_capacity(MONITOR_BUF_COUNT);

    for _ in 0..MONITOR_BUF_COUNT {
        unused_monitor_buffers.push(MonitorBuf::alloc());
    }

    let backend = MonitorBackend {
        rb_mon_prod,
        rb_recycle_con,
        unused_monitor_buffers,
    };

    let frontend = Monitor::new(rb_mon_con, rb_recycle_prod);

    (backend, frontend)
}

/// This structure holds the output of the 6 cell inputs and outputs
/// that is currently being monitored by the frontend.
pub struct MonitorBuf {
    /// Holds the data of the signals. Each signal has it's
    /// own length. The lengths of the individual elements is
    /// reflected in the `len` attribute.
    sig_blocks: [f32; MON_SIG_CNT * MAX_BLOCK_SIZE],

    /// Holds the lengths of the individual signal data blocks in `sig_blocks`.
    len:        [usize; MON_SIG_CNT],

    /// Holds the lengths of the individual signal data blocks in `sig_blocks`.
    read_idx:   [usize; MON_SIG_CNT],
}

/// A trait that represents any kind of monitorable sources
/// that provides at least MAX_BLOCK_SIZE samples.
pub trait MonitorSource {
    fn copy_to(&self, len: usize, slice: &mut [f32]);
}

impl MonitorSource for &[f32] {
    fn copy_to(&self, len: usize, slice: &mut [f32]) {
        slice.copy_from_slice(&self[0..len])
    }
}

impl MonitorBuf {
    /// Allocates a monitor buffer that holds up to 6 signals.
    pub fn alloc() -> MonitorBufPtr {
        Box::new(Self {
            sig_blocks: [0.0; MON_SIG_CNT * MAX_BLOCK_SIZE],
            len:        [0; MON_SIG_CNT],
            read_idx:   [0; MON_SIG_CNT],
        })
    }

    pub fn reset(&mut self) {
        self.len      = [0; MON_SIG_CNT];
        self.read_idx = [0; MON_SIG_CNT];
    }

    pub fn next_sample_for_signal(&mut self, idx: usize) -> Option<f32> {
        let rd_idx = self.read_idx[idx];
        if rd_idx >= self.len[idx] {
            return None;
        }

        self.read_idx[idx] = rd_idx + 1;
        let sb_idx = idx * MAX_BLOCK_SIZE;

        Some(self.sig_blocks[sb_idx + rd_idx])
    }

    pub fn feed<T>(&mut self, idx: usize, len: usize, data: T)
        where T: MonitorSource
    {
        let sb_idx = idx * MAX_BLOCK_SIZE;
        data.copy_to(len, &mut self.sig_blocks[sb_idx..(sb_idx + len)]);

        self.len[idx] = len;
    }
}

/// Pointer type for the [MonitorBuf]
pub type MonitorBufPtr = Box<MonitorBuf>;

#[cfg(test)]
mod tests {
    use super::*;

    fn send_n_monitor_bufs(backend: &mut MonitorBackend,
                            first: f32, last: f32, count: usize)
    {
        for _ in 0..count {
            let mut mon = backend.get_unused_mon_buf().unwrap();

            let mut samples : Vec<f32> = vec![];
            for _ in 0..MAX_BLOCK_SIZE {
                samples.push(0.0);
            }
            samples[0] = first;
            samples[MAX_BLOCK_SIZE - 1] = last;

            mon.feed(0, MAX_BLOCK_SIZE, &samples[..]);

            backend.send_mon_buf(mon);
        }
    }

    fn wait_for_monitor_process() {
        // FIXME: This could in theory do some spin waiting for
        //        the new_data flag!
        std::thread::sleep(
            std::time::Duration::from_millis(
                3 * MONITOR_PROC_THREAD_INTERVAL_MS));
    }

    #[test]
    fn check_monitor_proc() {
        let (mut backend, mut frontend) = new_monitor_processor();

        let count1 =
            (MONITOR_INPUT_LEN_PER_SAMPLE / MAX_BLOCK_SIZE) + 1;
        let count2 =
            2 * ((MONITOR_INPUT_LEN_PER_SAMPLE / MAX_BLOCK_SIZE) + 1);

        send_n_monitor_bufs(&mut backend, -0.9, 0.8, count1);

        send_n_monitor_bufs(&mut backend, -0.7, 0.6, count2);

        wait_for_monitor_process();

        let sl = frontend.get_minmax_monitor_samples(0);

        println!("{:?}", sl);

        assert_eq!(sl[MONITOR_MINMAX_SAMPLES - 1], (-0.7, 0.6));
        assert_eq!(sl[MONITOR_MINMAX_SAMPLES - 2], (-0.7, 0.8));
        assert_eq!(sl[MONITOR_MINMAX_SAMPLES - 3], (-0.9, 0.8));

        assert_eq!(
            backend.count_unused_mon_bufs(),
            MONITOR_BUF_COUNT - count1 - count2);

        backend.check_recycle();

        assert_eq!(
            backend.count_unused_mon_bufs(),
            MONITOR_BUF_COUNT);
    }

    #[test]
    fn check_monitor_partial() {
        let (mut backend, mut frontend) = new_monitor_processor();

        let count1 = MONITOR_INPUT_LEN_PER_SAMPLE / MAX_BLOCK_SIZE;

        send_n_monitor_bufs(&mut backend, -0.9, 0.8, count1);

        wait_for_monitor_process();

        let sl = frontend.get_minmax_monitor_samples(0);
        assert_eq!(sl[MONITOR_MINMAX_SAMPLES - 1], (0.0, 0.0));

        send_n_monitor_bufs(&mut backend, -0.9, 0.8, 1);

        wait_for_monitor_process();

        let sl = frontend.get_minmax_monitor_samples(0);
        assert_eq!(sl[MONITOR_MINMAX_SAMPLES - 1], (-0.9, 0.8));
    }

    #[test]
    fn check_monitor_fragment() {
        let (mut backend, mut frontend) = new_monitor_processor();

        let count1 = MONITOR_INPUT_LEN_PER_SAMPLE / MAX_BLOCK_SIZE;

        let rest = MONITOR_INPUT_LEN_PER_SAMPLE - count1 * MAX_BLOCK_SIZE;

        send_n_monitor_bufs(&mut backend, -0.9, 0.8, count1);

        wait_for_monitor_process();

        let sl = frontend.get_minmax_monitor_samples(0);
        assert_eq!(sl[0], (0.0, 0.0));

        let mut mon = backend.get_unused_mon_buf().unwrap();

        let mut samples : Vec<f32> = vec![];
        let part1_len = rest - 1;
        for _ in 0..part1_len {
            samples.push(0.0);
        }
        samples[0]             = -0.9;
        samples[part1_len - 1] = -0.95;

        mon.feed(0, part1_len, &samples[..]);
        backend.send_mon_buf(mon);

        wait_for_monitor_process();

        let sl = frontend.get_minmax_monitor_samples(0);
        assert_eq!(sl[MONITOR_MINMAX_SAMPLES - 1], (0.0, 0.0));

        let mut mon = backend.get_unused_mon_buf().unwrap();
        mon.feed(0, 1, &[0.86][..]);
        backend.send_mon_buf(mon);

        wait_for_monitor_process();

        let sl = frontend.get_minmax_monitor_samples(0);
        assert_eq!(sl[MONITOR_MINMAX_SAMPLES - 1], (-0.95, 0.86));
    }

    #[test]
    fn check_monitor_wrap_buf() {
        let (mut backend, mut frontend) = new_monitor_processor();

        let count1 =
            (MONITOR_INPUT_LEN_PER_SAMPLE / MAX_BLOCK_SIZE) + 1;

        for i in 0..MONITOR_MINMAX_SAMPLES {
            let v = i as f32 / MONITOR_MINMAX_SAMPLES as f32;
            send_n_monitor_bufs(&mut backend, -0.9, v, count1);

            // Give the MonitorProcessor some time to work on the buffers.
            std::thread::sleep(
                std::time::Duration::from_millis(5));
            backend.check_recycle();
        }
        wait_for_monitor_process();
        backend.check_recycle();

        let sl = frontend.get_minmax_monitor_samples(0);
        println!("{:?}", sl);

        assert_eq!(
            (sl[MONITOR_MINMAX_SAMPLES - 1].1 * 10000.0).floor() as u32,
            9921);

        assert_eq!(
            backend.count_unused_mon_bufs(),
            MONITOR_BUF_COUNT);
    }
}
