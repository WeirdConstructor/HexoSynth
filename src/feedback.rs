use crate::dsp::MAX_BLOCK_SIZE;
use ringbuf::{RingBuffer, Producer, Consumer};

/// 3 inputs, 3 outputs of feedback.
pub const FB_SIG_CNT : usize = 6;

/// Just some base to determine the feedback buffer sizes.
const IMAGINARY_MAX_SAMPLE_RATE : usize = 48000;

/// The number of minmax samples to hold.
const FEEDBACK_MINMAX_SAMPLES : usize = 128;

/// The length in seconds of the FEEDBACK_MINMAX_SAMPLES
const FEEDBACK_MINMAX_LEN_S   : usize = 2;

// TODO / FIXME: We should recalculate this on the basis of the
// real actual sample rate, otherwise the feedback scope
// is going to be too fast.
/// The number of audio samples over which to calculate
/// one min/max sample. Typically something around 750.
const FEEDBACK_INPUT_LEN_PER_SAMPLE : usize =
    (FEEDBACK_MINMAX_LEN_S * IMAGINARY_MAX_SAMPLE_RATE)
    / FEEDBACK_MINMAX_SAMPLES;

/// Maximum number of feedback buffers to hold in the backend.
/// Typically there are only 16-32ms of feedback content floating
/// around, as the feedback processing thread regularily
/// processes the feedback.
const FEEDBACK_BUF_COUNT  : usize =
//  2 for safety margin
    2 * (IMAGINARY_MAX_SAMPLE_RATE / MAX_BLOCK_SIZE);

pub struct BackendFeedbackProvider {
    rb_fb_prod:              Producer<FeedbackBufPtr>,
    rb_recycle_con:          Consumer<FeedbackBufPtr>,

    /// Holds enough feedback buffers to hold about 1-2 seconds
    /// of data. The [FeedbackBuf] boxes are written in the
    /// backend and then sent via [feedback_prod] to the frontend.
    /// The frontend then sends the used [FeedbackBufPtr] back
    /// via quick_update_con.
    unused_feedback_buffers: Vec<FeedbackBufPtr>,
}

impl BackendFeedbackProvider {
    pub fn check_recycle(&mut self) {
        while let Some(buf) = self.rb_recycle_con.pop() {
            self.unused_feedback_buffers.push(buf);
        }
    }

    pub fn get_unused_fb_buf(&mut self) -> Option<FeedbackBufPtr> {
        self.unused_feedback_buffers.pop()
    }

    pub fn count_unused_fb_bufs(&self) -> usize {
        self.unused_feedback_buffers.len()
    }

    pub fn send_fb_buf(&mut self, buf: FeedbackBufPtr) {
        match self.rb_fb_prod.push(buf) {
            Ok(_)    => (),
            Err(buf) => self.unused_feedback_buffers.push(buf),
        }
    }
}

/// Implements the logic for min/maxing a single signal channel/line.
pub struct FeedbackMinMax {
    /// Index of the signal in the [FeedbackBuf]
    sig_idx:        usize,

    /// A ring buffer of min/max samples, written to by `buf_write_ptr`.
    buf:            [(f32, f32); FEEDBACK_MINMAX_SAMPLES],

    /// The pointer/index into `buf` to the next update to write.
    buf_write_ptr:  usize,

    /// Holds the currently accumulated min/max values and the length
    /// of so far processed audio rate samples. Once FEEDBACK_INPUT_LEN_PER_SAMPLE
    /// is reached, this will be written into `buf`.
    cur_min_max:    (f32, f32, usize),
}

impl FeedbackMinMax {
    pub fn new(sig_idx: usize) -> Self {
        Self {
            sig_idx,
            buf:           [(0.0, 0.0); FEEDBACK_MINMAX_SAMPLES],
            buf_write_ptr: 0,
            cur_min_max:   (100.0, -100.0, 0),
        }
    }

    pub fn process(&mut self, fb_buf: &mut FeedbackBufPtr) {
        while let Some(sample) =
            fb_buf.next_sample_for_signal(self.sig_idx)
        {
            self.cur_min_max.0 = self.cur_min_max.0.min(sample);
            self.cur_min_max.1 = self.cur_min_max.1.max(sample);
            self.cur_min_max.2 += 1;

            if self.cur_min_max.2 >= FEEDBACK_INPUT_LEN_PER_SAMPLE {
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

/// Coordinates the processing of incoming FeedbackBufs.
pub struct FeedbackProcessor {
    rb_fb_con:              Consumer<FeedbackBufPtr>,
    rb_recycle_prod:        Producer<FeedbackBufPtr>,

    procs: Vec<FeedbackMinMax>,
}

impl FeedbackProcessor {
    pub fn new(rb_fb_con: Consumer<FeedbackBufPtr>,
               rb_recycle_prod: Producer<FeedbackBufPtr>)
        -> Self
    {
        let mut procs = vec![];
        for i in 0..FB_SIG_CNT {
            procs.push(FeedbackMinMax::new(i));
        }

        Self {
            rb_fb_con,
            rb_recycle_prod,
            procs,
        }
    }

    pub fn minmax_slice_for_signal(&self, idx: usize) -> &[(f32, f32)] {
        &self.procs[idx].buf[..]
    }

    pub fn process_fb_buf(&mut self, fb_buf: &mut FeedbackBufPtr) {
        for proc in self.procs.iter_mut() {
            proc.process(fb_buf);
        }
    }

    pub fn process(&mut self) {
        while let Some(mut buf) = self.rb_fb_con.pop() {
            self.process_fb_buf(&mut buf);
            let _ = self.rb_recycle_prod.push(buf);
        }
    }
}

pub fn new_feedback_processor() -> (BackendFeedbackProvider, FeedbackProcessor) {
    let rb_feedback  = RingBuffer::new(FEEDBACK_BUF_COUNT);
    let rb_recycle   = RingBuffer::new(FEEDBACK_BUF_COUNT);

    let (rb_fb_prod,      rb_fb_con)      = rb_feedback.split();
    let (rb_recycle_prod, rb_recycle_con) = rb_recycle.split();

    let mut unused_feedback_buffers = Vec::with_capacity(FEEDBACK_BUF_COUNT);

    for _ in 0..FEEDBACK_BUF_COUNT {
        unused_feedback_buffers.push(FeedbackBuf::alloc());
    }

    let backend = BackendFeedbackProvider {
        rb_fb_prod,
        rb_recycle_con,
        unused_feedback_buffers,
    };

    let frontend = FeedbackProcessor::new(rb_fb_con, rb_recycle_prod);

    (backend, frontend)
}

/// This structure holds the output of the 6 cell inputs and outputs
/// that is currently being monitored by the frontend.
pub struct FeedbackBuf {
    /// Holds the data of the signals. Each signal has it's
    /// own length. The lengths of the individual elements is
    /// reflected in the `len` attribute.
    sig_blocks: [f32; FB_SIG_CNT * MAX_BLOCK_SIZE],

    /// Holds the lengths of the individual signal data blocks in `sig_blocks`.
    len:        [usize; FB_SIG_CNT],

    /// Holds the lengths of the individual signal data blocks in `sig_blocks`.
    read_idx:   [usize; FB_SIG_CNT],
}

impl FeedbackBuf {
    /// Allocates a feedback buffer that holds up to 6 signals.
    pub fn alloc() -> FeedbackBufPtr {
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

/// Pointer type for the [FeedbackBuf]
pub type FeedbackBufPtr = Box<FeedbackBuf>;

#[cfg(test)]
mod tests {
    use super::*;

    fn send_n_feedback_bufs(backend: &mut BackendFeedbackProvider,
                            first: f32, last: f32, count: usize)
    {
        for _ in 0..count {
            let mut fb = backend.get_unused_fb_buf().unwrap();

            let mut samples : Vec<f32> = vec![];
            for _ in 0..MAX_BLOCK_SIZE {
                samples.push(0.0);
            }
            samples[0] = first;
            samples[MAX_BLOCK_SIZE - 1] = last;

            fb.feed(0, MAX_BLOCK_SIZE, &samples[..]);

            backend.send_fb_buf(fb);
        }
    }

    #[test]
    fn check_feedback_proc() {
        let (mut backend, mut frontend) = new_feedback_processor();

        let count1 =
            (FEEDBACK_INPUT_LEN_PER_SAMPLE / MAX_BLOCK_SIZE) + 1;
        let count2 =
            2 * ((FEEDBACK_INPUT_LEN_PER_SAMPLE / MAX_BLOCK_SIZE) + 1);

        send_n_feedback_bufs(&mut backend, -0.9, 0.8, count1);

        send_n_feedback_bufs(&mut backend, -0.7, 0.6, count2);

        frontend.process();

        let sl = frontend.minmax_slice_for_signal(0);
        println!("{:?}", sl);

        assert_eq!(sl[0], (-0.9, 0.8));
        assert_eq!(sl[1], (-0.7, 0.8));
        assert_eq!(sl[2], (-0.7, 0.6));

        assert_eq!(
            backend.count_unused_fb_bufs(),
            FEEDBACK_BUF_COUNT - count1 - count2);

        backend.check_recycle();

        assert_eq!(
            backend.count_unused_fb_bufs(),
            FEEDBACK_BUF_COUNT);
    }

    #[test]
    fn check_feedback_partial() {
        let (mut backend, mut frontend) = new_feedback_processor();

        let count1 = FEEDBACK_INPUT_LEN_PER_SAMPLE / MAX_BLOCK_SIZE;

        send_n_feedback_bufs(&mut backend, -0.9, 0.8, count1);
        frontend.process();

        let sl = frontend.minmax_slice_for_signal(0);
        assert_eq!(sl[0], (0.0, 0.0));

        send_n_feedback_bufs(&mut backend, -0.9, 0.8, 1);
        frontend.process();

        let sl = frontend.minmax_slice_for_signal(0);
        assert_eq!(sl[0], (-0.9, 0.8));
    }

    #[test]
    fn check_feedback_fragment() {
        let (mut backend, mut frontend) = new_feedback_processor();

        let count1 = FEEDBACK_INPUT_LEN_PER_SAMPLE / MAX_BLOCK_SIZE;

        let rest = FEEDBACK_INPUT_LEN_PER_SAMPLE - count1 * MAX_BLOCK_SIZE;

        send_n_feedback_bufs(&mut backend, -0.9, 0.8, count1);
        frontend.process();

        let sl = frontend.minmax_slice_for_signal(0);
        assert_eq!(sl[0], (0.0, 0.0));

        let mut fb = backend.get_unused_fb_buf().unwrap();

        let mut samples : Vec<f32> = vec![];
        let part1_len = rest - 1;
        for _ in 0..part1_len {
            samples.push(0.0);
        }
        samples[0]             = -0.9;
        samples[part1_len - 1] = -0.95;

        fb.feed(0, part1_len, &samples[..]);
        backend.send_fb_buf(fb);

        frontend.process();

        let sl = frontend.minmax_slice_for_signal(0);
        assert_eq!(sl[0], (0.0, 0.0));

        let mut fb = backend.get_unused_fb_buf().unwrap();
        fb.feed(0, 1, &[0.86]);
        backend.send_fb_buf(fb);

        frontend.process();
        let sl = frontend.minmax_slice_for_signal(0);
        assert_eq!(sl[0], (-0.95, 0.86));
    }

    // TODO: Testcase for two incomplete buffers 32/32

    // TODO: Testcase for two incomplete buffers 1/63
}
