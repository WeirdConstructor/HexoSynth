// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use crate::nodes::NodeAudioContext;
use crate::dsp::helpers::TriggerClock;
use crate::dsp::{SAtom, ProcBuf, DspNode, LedPhaseVals};
use crate::dsp::tracker::TrackerBackend;

use crate::dsp::MAX_BLOCK_SIZE;
use crate::dsp::tracker::MAX_COLS;

/// A tracker based sequencer
#[derive(Debug)]
pub struct TSeq {
    backend:       Option<Box<TrackerBackend>>,
    clock:         TriggerClock,
    srate:         f64,
}

impl Clone for TSeq {
    fn clone(&self) -> Self { Self::new() }
}

impl TSeq {
    pub fn new() -> Self {
        Self {
            backend:       None,
            srate:         48000.0,
            clock:         TriggerClock::new(),
        }
    }

    pub fn set_backend(&mut self, backend: TrackerBackend) {
        self.backend = Some(Box::new(backend));
    }

    pub const clock : &'static str =
        "TSeq clock\nClock input\nRange: (0..1)\n";
    pub const cmode : &'static str =
        "TSeq cmode\n'clock' input signal mode:\n\
             - RowT: Trigger = advance row\n\
             - PatT: Trigger = pattern rate\n\
             - Phase: Phase to pattern index\n\
         \n";
    pub const trk1 : &'static str =
        "TSeq trk1\nTrack 1 signal output\nRange: (-1..1)\n";
    pub const trk2 : &'static str =
        "TSeq trk2\nTrack 2 signal output\nRange: (-1..1)\n";
    pub const trk3 : &'static str =
        "TSeq trk3\nTrack 3 signal output\nRange: (-1..1)\n";
    pub const trk4 : &'static str =
        "TSeq trk4\nTrack 4 signal output\nRange: (-1..1)\n";
    pub const trk5 : &'static str =
        "TSeq trk5\nTrack 5 signal output\nRange: (-1..1)\n";
    pub const trk6 : &'static str =
        "TSeq trk6\nTrack 6 signal output\nRange: (-1..1)\n";
}

impl DspNode for TSeq {
    fn outputs() -> usize { 1 }

    fn set_sample_rate(&mut self, srate: f32) {
        self.srate = srate as f64;
    }

    fn reset(&mut self) {
        self.backend        = None;
        self.clock.reset();
    }

    #[inline]
    fn process<T: NodeAudioContext>(
        &mut self, ctx: &mut T, atoms: &[SAtom], _params: &[ProcBuf],
        inputs: &[ProcBuf], outputs: &mut [ProcBuf], ctx_vals: LedPhaseVals)
    {
        use crate::dsp::{out, inp, denorm, denorm_v, inp_dir, at};
        let clock = inp::TSeq::clock(inputs);
        let cmode = at::TSeq::cmode(atoms);

        let mut backend =
            if let Some(backend) = &mut self.backend {
                backend
            } else { return; };

        backend.check_updates();

        let mut phase_out : [f32; MAX_BLOCK_SIZE] =
            [0.0; MAX_BLOCK_SIZE];

        let cmode = cmode.i();

        for frame in 0..ctx.nframes() {
            let mut clock_phase =
                if cmode < 2 {
                    self.clock.next_phase(clock.read(frame))
                } else {
                    clock.read(frame).abs() as f64
                };

            let phase =
                match cmode {
                    // RowT
                    0 => {
                        let plen = backend.pattern_len() as f64;
                        while clock_phase >= plen {
                            clock_phase -= plen;
                        }

                        clock_phase / plen
                    },
                    // 1 | 2 PatT, Phase
                    _ => {
                        clock_phase = clock_phase.fract();
                        clock_phase
                    },
                };

            phase_out[frame] = phase as f32;
        }

//        println!("PHASE {}", phase_out[0]);

        let mut col_out : [f32; MAX_BLOCK_SIZE] =
            [0.0; MAX_BLOCK_SIZE];
        let mut col_out_slice   = &mut col_out[0..ctx.nframes()];
        let mut phase_out_slice = &phase_out[0..ctx.nframes()];

        let out_t1     = out::TSeq::trk1(outputs);
        backend.get_col_at_phase(
            0, phase_out_slice, col_out_slice);
        out_t1.write_from(col_out_slice);

        ctx_vals[0].set(col_out_slice[col_out_slice.len() - 1]);

        let out_t2     = out::TSeq::trk2(outputs);
        backend.get_col_at_phase(
            1, phase_out_slice, col_out_slice);
        out_t2.write_from(col_out_slice);

        let out_t3     = out::TSeq::trk3(outputs);
        backend.get_col_at_phase(
            2, phase_out_slice, col_out_slice);
        out_t3.write_from(col_out_slice);

        let out_t4     = out::TSeq::trk4(outputs);
        backend.get_col_at_phase(
            3, phase_out_slice, col_out_slice);
        out_t4.write_from(col_out_slice);

        let out_t5     = out::TSeq::trk5(outputs);
        backend.get_col_at_phase(
            4, phase_out_slice, col_out_slice);
        out_t5.write_from(col_out_slice);

        let out_t6     = out::TSeq::trk6(outputs);
        backend.get_col_at_phase(
            5, phase_out_slice, col_out_slice);
        out_t6.write_from(col_out_slice);

        ctx_vals[1].set(phase_out_slice[phase_out_slice.len() - 1]);
    }
}
