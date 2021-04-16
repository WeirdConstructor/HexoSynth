mod pattern;
mod sequencer;

pub const MAX_PATTERN_LEN : usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PatternColType {
    Note,
    Step,
    Value,
    Gate,
}

pub use pattern::PatternData;
pub use sequencer::PatternSequencer;
