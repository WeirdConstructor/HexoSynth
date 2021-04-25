mod pattern;
mod sequencer;

use ringbuf::{RingBuffer, Producer, Consumer};

pub const MAX_COLS         : usize = 6;
pub const MAX_PATTERN_LEN  : usize = 256;
pub const MAX_RINGBUF_SIZE : usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PatternColType {
    Note,
    Step,
    Value,
    Gate,
}

pub use pattern::PatternData;
pub use sequencer::PatternSequencer;

pub enum PatternUpdateMsg {
    UpdateColumn {
        col_type:    PatternColType,
        pattern_len: usize,
        data:        [f32; MAX_PATTERN_LEN]
    },
}

struct Tracker {
    data:      PatternData,
    data_prod: Producer<PatternUpdateMsg>,
    seq:       Option<PatternSequencer>,
    seq_cons:  Option<Consumer<PatternUpdateMsg>>,
}

struct TrackerBackend {
    seq:      PatternSequencer,
    seq_cons: Consumer<PatternUpdateMsg>,
}

impl Tracker {
    pub fn new() -> Self {
        let rb = RingBuffer::new(MAX_RINGBUF_SIZE);
        let (prod, con) = rb.split();

        Self {
            data:      PatternData::new(MAX_PATTERN_LEN),
            data_prod: prod,
            seq:       Some(PatternSequencer::new(MAX_PATTERN_LEN)),
            seq_cons:  Some(con),
        }
    }

    pub fn data(&mut self) -> &mut PatternData { &mut self.data }

    pub fn send_updates(&mut self) {
    }

    pub fn fetch_backend(&mut self) -> TrackerBackend {
        if self.seq.is_none() {
            let rb = RingBuffer::new(MAX_RINGBUF_SIZE);
            let (prod, con) = rb.split();

            self.seq        = Some(PatternSequencer::new(MAX_PATTERN_LEN));
            self.data_prod  = prod;
            self.seq_cons   = Some(con);
        }

        let seq      = self.seq.take().unwrap();
        let seq_cons = self.seq_cons.take().unwrap();

        TrackerBackend {
            seq,
            seq_cons,
        }
    }
}
