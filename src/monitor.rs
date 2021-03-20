use crate::dsp::MAX_BLOCK_SIZE;
use ringbuf::{RingBuffer, Producer, Consumer};

/// 3 inputs, 3 outputs of signal monitors.
pub const FB_SIG_CNT : usize = 6;

/// Just some base to determine the monitor buffer sizes.
const IMAGINARY_MAX_SAMPLE_RATE : usize = 48000;

/// The number of minmax samples to hold.
const MONITOR_MINMAX_SAMPLES : usize = 128;

/// The length in seconds of the MONITOR_MINMAX_SAMPLES
const MONITOR_MINMAX_LEN_S   : usize = 2;

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

pub struct BackendMonitorProvider {
    rb_mon_prod:              Producer<MonitorBufPtr>,
    rb_recycle_con:          Consumer<MonitorBufPtr>,

    /// Holds enough monitor buffers to hold about 1-2 seconds
    /// of data. The [MonitorBuf] boxes are written in the
    /// backend and then sent via [monitor_prod] to the frontend.
    /// The frontend then sends the used [MonitorBufPtr] back
    /// via quick_update_con.
    unused_monitor_buffers: Vec<MonitorBufPtr>,
}

impl BackendMonitorProvider {
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

    pub fn process(&mut self, mon_buf: &mut MonitorBufPtr) {
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

                self.buf_write_ptr = (self.buf_write_ptr + 1) % self.buf.len();

                self.cur_min_max.0 =  100.0;
                self.cur_min_max.1 = -100.0;
                self.cur_min_max.2 = 0;
            }
        }
    }
}

/// Coordinates the processing of incoming MonitorBufs.
pub struct MonitorProcessor {
    rb_mon_con:              Consumer<MonitorBufPtr>,
    rb_recycle_prod:        Producer<MonitorBufPtr>,

    procs: Vec<MonitorMinMax>,
}

impl MonitorProcessor {
    pub fn new(rb_mon_con: Consumer<MonitorBufPtr>,
               rb_recycle_prod: Producer<MonitorBufPtr>)
        -> Self
    {
        let mut procs = vec![];
        for i in 0..FB_SIG_CNT {
            procs.push(MonitorMinMax::new(i));
        }

        Self {
            rb_mon_con,
            rb_recycle_prod,
            procs,
        }
    }

    /// Helper function for tests, to access the current state of
    /// the min/max buffers.
    pub fn minmax_slice_for_signal(&self, idx: usize) -> &[(f32, f32)] {
        &self.procs[idx].buf[..]
    }

    /// Internal helper function for `process`.
    fn process_mon_buf(&mut self, mon_buf: &mut MonitorBufPtr) {
        for proc in self.procs.iter_mut() {
            proc.process(mon_buf);
        }
    }

    /// Processes all queued [MonitorBuf] instances and sends
    /// then back to the [BackendMonitorProvider] thread after
    /// used for recycling.
    pub fn process(&mut self) {
        while let Some(mut buf) = self.rb_mon_con.pop() {
            self.process_mon_buf(&mut buf);
            buf.reset();
            let _ = self.rb_recycle_prod.push(buf);
        }
    }
}

/// Creates a pair of interconnected BackendMonitorProvider and MonitorProcessor
/// instances, to be sent to different threads.
pub fn new_monitor_processor() -> (BackendMonitorProvider, MonitorProcessor) {
    let rb_monitor  = RingBuffer::new(MONITOR_BUF_COUNT);
    let rb_recycle   = RingBuffer::new(MONITOR_BUF_COUNT);

    let (rb_mon_prod,     rb_mon_con)     = rb_monitor.split();
    let (rb_recycle_prod, rb_recycle_con) = rb_recycle.split();

    let mut unused_monitor_buffers = Vec::with_capacity(MONITOR_BUF_COUNT);

    for _ in 0..MONITOR_BUF_COUNT {
        unused_monitor_buffers.push(MonitorBuf::alloc());
    }

    let backend = BackendMonitorProvider {
        rb_mon_prod,
        rb_recycle_con,
        unused_monitor_buffers,
    };

    let frontend = MonitorProcessor::new(rb_mon_con, rb_recycle_prod);

    (backend, frontend)
}

/// This structure holds the output of the 6 cell inputs and outputs
/// that is currently being monitored by the frontend.
pub struct MonitorBuf {
    /// Holds the data of the signals. Each signal has it's
    /// own length. The lengths of the individual elements is
    /// reflected in the `len` attribute.
    sig_blocks: [f32; FB_SIG_CNT * MAX_BLOCK_SIZE],

    /// Holds the lengths of the individual signal data blocks in `sig_blocks`.
    len:        [usize; FB_SIG_CNT],

    /// Holds the lengths of the individual signal data blocks in `sig_blocks`.
    read_idx:   [usize; FB_SIG_CNT],
}

impl MonitorBuf {
    /// Allocates a monitor buffer that holds up to 6 signals.
    pub fn alloc() -> MonitorBufPtr {
        Box::new(Self {
            sig_blocks: [0.0; FB_SIG_CNT * MAX_BLOCK_SIZE],
            len:        [0; FB_SIG_CNT],
            read_idx:   [0; FB_SIG_CNT],
        })
    }

    pub fn reset(&mut self) {
        self.len      = [0; FB_SIG_CNT];
        self.read_idx = [0; FB_SIG_CNT];
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

    pub fn feed(&mut self, idx: usize, len: usize, slice: &[f32]) {
        let sb_idx = idx * MAX_BLOCK_SIZE;
        self.sig_blocks[sb_idx..(sb_idx + len)]
            .copy_from_slice(slice);

        self.len[idx] = len;
    }
}

/// Pointer type for the [MonitorBuf]
pub type MonitorBufPtr = Box<MonitorBuf>;

#[cfg(test)]
mod tests {
    use super::*;

    fn send_n_monitor_bufs(backend: &mut BackendMonitorProvider,
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

    #[test]
    fn check_monitor_proc() {
        let (mut backend, mut frontend) = new_monitor_processor();

        let count1 =
            (MONITOR_INPUT_LEN_PER_SAMPLE / MAX_BLOCK_SIZE) + 1;
        let count2 =
            2 * ((MONITOR_INPUT_LEN_PER_SAMPLE / MAX_BLOCK_SIZE) + 1);

        send_n_monitor_bufs(&mut backend, -0.9, 0.8, count1);

        send_n_monitor_bufs(&mut backend, -0.7, 0.6, count2);

        frontend.process();

        let sl = frontend.minmax_slice_for_signal(0);
        println!("{:?}", sl);

        assert_eq!(sl[0], (-0.9, 0.8));
        assert_eq!(sl[1], (-0.7, 0.8));
        assert_eq!(sl[2], (-0.7, 0.6));

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
        frontend.process();

        let sl = frontend.minmax_slice_for_signal(0);
        assert_eq!(sl[0], (0.0, 0.0));

        send_n_monitor_bufs(&mut backend, -0.9, 0.8, 1);
        frontend.process();

        let sl = frontend.minmax_slice_for_signal(0);
        assert_eq!(sl[0], (-0.9, 0.8));
    }

    #[test]
    fn check_monitor_fragment() {
        let (mut backend, mut frontend) = new_monitor_processor();

        let count1 = MONITOR_INPUT_LEN_PER_SAMPLE / MAX_BLOCK_SIZE;

        let rest = MONITOR_INPUT_LEN_PER_SAMPLE - count1 * MAX_BLOCK_SIZE;

        send_n_monitor_bufs(&mut backend, -0.9, 0.8, count1);
        frontend.process();

        let sl = frontend.minmax_slice_for_signal(0);
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

        frontend.process();

        let sl = frontend.minmax_slice_for_signal(0);
        assert_eq!(sl[0], (0.0, 0.0));

        let mut mon = backend.get_unused_mon_buf().unwrap();
        mon.feed(0, 1, &[0.86]);
        backend.send_mon_buf(mon);

        frontend.process();
        let sl = frontend.minmax_slice_for_signal(0);
        assert_eq!(sl[0], (-0.95, 0.86));
    }

    #[test]
    fn check_monitor_wrap_buf() {
        let (mut backend, mut frontend) = new_monitor_processor();

        let count1 =
            (MONITOR_INPUT_LEN_PER_SAMPLE / MAX_BLOCK_SIZE) + 1;

        for i in 0..MONITOR_MINMAX_SAMPLES {
            let v = i as f32 / MONITOR_MINMAX_SAMPLES as f32;
            send_n_monitor_bufs(&mut backend, -0.9, v, count1);
            frontend.process();
            backend.check_recycle();
        }

        let sl = frontend.minmax_slice_for_signal(0);

        assert_eq!((sl[0].1 * 10000.0).floor() as u32, 9765);

        assert_eq!(
            backend.count_unused_mon_bufs(),
            MONITOR_BUF_COUNT);
    }
}
