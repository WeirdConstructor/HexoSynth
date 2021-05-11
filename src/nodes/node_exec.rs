// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use super::{
    GraphMessage, QuickMessage, DropMsg, NodeProg,
    UNUSED_MONITOR_IDX, MAX_ALLOCATED_NODES, MAX_SMOOTHERS
};
use crate::dsp::{NodeId, Node};
use crate::util::{Smoother, AtomicFloat};
use crate::monitor::{MonitorBackend, MON_SIG_CNT};

use ringbuf::{Producer, Consumer};
use std::sync::Arc;

/// Holds the complete allocation of nodes and
/// the program. New Nodes or the program is
/// not newly allocated in the audio backend, but it is
/// copied from the input ring buffer.
/// If this turns out to be too slow, we might
/// have to push buffers of the program around.
///
pub struct NodeExecutor {
    /// Contains the nodes and their state.
    /// Is loaded from the input ring buffer when a corresponding
    /// message arrives.
    ///
    /// In case the previous node contained something that needs
    /// deallocation, the nodes are replaced and the contents
    /// is sent back using the free-ringbuffer.
    pub(crate) nodes: Vec<Node>,

    /// Contains the stand-by smoothing operators for incoming parameter changes.
    pub(crate) smoothers: Vec<(usize, Smoother)>,

    /// Contains target parameter values after a smoother finished,
    /// these will refresh the input buffers:
    pub(crate) target_refresh: Vec<(usize, f32)>,

    /// Contains the to be executed nodes and output operations.
    /// Is copied from the input ringbuffer when a corresponding
    /// message arrives.
    pub(crate) prog: NodeProg,

    /// Holds the input vector indices which are to be monitored by the frontend.
    pub(crate) monitor_signal_cur_inp_indices: [usize; MON_SIG_CNT],

    /// The sample rate
    pub(crate) sample_rate: f32,

    /// The connection with the [crate::nodes::NodeConfigurator].
    shared: SharedNodeExec,
}

/// Contains anything that connects the [NodeExecutor] with the frontend part.
pub(crate) struct SharedNodeExec {
    /// Holds two context values interleaved.
    /// The first for each node is the LED value and the second is a
    /// phase value. The LED will be displayed in the hex matrix, while the
    /// phase might be used to display an envelope's play position.
    pub(crate) node_ctx_values: Vec<Arc<AtomicFloat>>,
    /// For receiving Node and NodeProg updates
    pub(crate) graph_update_con:  Consumer<GraphMessage>,
    /// For quick updates like UI paramter changes.
    pub(crate) quick_update_con:  Consumer<QuickMessage>,
    /// For receiving deleted/overwritten nodes from the backend thread.
    pub(crate) graph_drop_prod:   Producer<DropMsg>,
    /// For sending feedback to the frontend thread.
    pub(crate) monitor_backend:   MonitorBackend,
}

pub trait NodeAudioContext {
    fn nframes(&self) -> usize;
    fn output(&mut self, channel: usize, frame: usize, v: f32);
    fn input(&mut self, channel: usize, frame: usize) -> f32;
}

impl NodeExecutor {
    pub(crate) fn new(shared: SharedNodeExec) -> Self {
        let mut nodes = Vec::new();
        nodes.resize_with(MAX_ALLOCATED_NODES, || Node::Nop);

        let mut smoothers = Vec::new();
        smoothers.resize_with(MAX_SMOOTHERS, || (0, Smoother::new()));

        let target_refresh = Vec::with_capacity(MAX_SMOOTHERS);

        NodeExecutor {
            nodes,
            smoothers,
            target_refresh,
            sample_rate:       44100.0,
            prog:              NodeProg::empty(),
            monitor_signal_cur_inp_indices: [UNUSED_MONITOR_IDX; MON_SIG_CNT],
            shared,
        }
    }

    #[inline]
    pub fn process_graph_updates(&mut self) {
        while let Some(upd) = self.shared.graph_update_con.pop() {
            match upd {
                GraphMessage::NewNode { index, mut node } => {
                    node.set_sample_rate(self.sample_rate);
                    let prev_node =
                        std::mem::replace(
                            &mut self.nodes[index as usize],
                            node);
                    let _ =
                        self.shared.graph_drop_prod.push(
                            DropMsg::Node { node: prev_node });
                },
                GraphMessage::Clear { prog } => {
                    for n in self.nodes.iter_mut() {
                        if n.to_id(0) != NodeId::Nop {
                            let prev_node = std::mem::replace(n, Node::Nop);
                            let _ =
                                self.shared.graph_drop_prod.push(
                                    DropMsg::Node { node: prev_node });
                        }
                    }

                    self.monitor_signal_cur_inp_indices =
                        [UNUSED_MONITOR_IDX; MON_SIG_CNT];

                    let prev_prog = std::mem::replace(&mut self.prog, prog);
                    let _ =
                        self.shared.graph_drop_prod.push(
                            DropMsg::Prog { prog: prev_prog });
                },
                GraphMessage::NewProg { prog, copy_old_out } => {
                    let mut prev_prog = std::mem::replace(&mut self.prog, prog);

                    self.monitor_signal_cur_inp_indices =
                        [UNUSED_MONITOR_IDX; MON_SIG_CNT];

                    // XXX: Copying from the old vector works, because we only
                    //      append nodes to the _end_ of the node instance vector.
                    //      If we do a garbage collection, we can't do this.
                    //
                    // XXX: Also, we need to initialize the input parameter
                    //      vector, because we don't know if they are updated from
                    //      the new program outputs anymore. So we need to 
                    //      copy the old paramters to the inputs.
                    //
                    //      => This does not apply to atom data, because that
                    //         is always sent with the new program and "should"
                    //         be up to date, even if we have a slight possible race
                    //         condition between GraphMessage::NewProg
                    //         and QuickMessage::AtomUpdate.

                    // First overwrite by the current input parameters,
                    // to make sure _all_ inputs have a proper value
                    // (not just those that existed before).
                    //
                    // We preserve the modulation history in the next step.
                    // This is also to make sure that new input ports
                    // have a proper value too.
                    self.prog.initialize_input_buffers();

                    if copy_old_out {
                        // XXX: The following is commented out, because presisting
                        //      the output proc buffers does not make sense anymore.
                        //      Because we don't allow cycles, so there is no
                        //      way that a node can read from the previous
                        //      iteration anyways.
                        //
                        // // Swap the old out buffers into the new NodeProg
                        // // TODO: If we toss away most of the buffers anyways,
                        // //       we could optimize this step with more
                        // //       intelligence in the matrix compiler.
                        // for (old_pb, new_pb) in
                        //     prev_prog.out.iter_mut().zip(
                        //         self.prog.out.iter_mut())
                        // {
                        //     std::mem::swap(old_pb, new_pb);
                        // }

                        // Then overwrite the inputs by the more current previous
                        // input processing buffers, so we keep any modulation
                        // (smoothed) history of the block too.
                        self.prog.swap_previous_outputs(&mut prev_prog);
                    }

                    self.prog.assign_outputs();

                    let _ =
                        self.shared.graph_drop_prod.push(
                            DropMsg::Prog { prog: prev_prog });
                },
            }
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        for n in self.nodes.iter_mut() {
            n.set_sample_rate(sample_rate);
        }

        for sm in self.smoothers.iter_mut() {
            sm.1.set_sample_rate(sample_rate);
        }
    }

    #[inline]
    pub fn get_nodes(&self) -> &Vec<Node> { &self.nodes }

    #[inline]
    pub fn get_prog(&self) -> &NodeProg { &self.prog }

    #[inline]
    fn set_param(&mut self, input_idx: usize, value: f32) {
        let prog = &mut self.prog;

        if input_idx >= prog.params.len() {
            return;
        }

        // First check if we already have a running smoother for this param:
        for (sm_inp_idx, smoother) in
            self.smoothers
                .iter_mut()
                .filter(|s| !s.1.is_done())
        {
            if *sm_inp_idx == input_idx {
                smoother.set(prog.params[input_idx], value);
                //d// println!("RE-SET SMOOTHER {} {:6.3} (old = {:6.3})",
                //d//          input_idx, value, prog.params[input_idx]);
                return;
            }
        }

        // Find unused smoother and set it:
        if let Some(sm) =
            self.smoothers
                .iter_mut()
                .filter(|s| s.1.is_done())
                .next()
        {
            sm.0 = input_idx;
            sm.1.set(prog.params[input_idx], value);
            //d// println!("SET SMOOTHER {} {:6.3} (old = {:6.3})",
            //d//          input_idx, value, prog.params[input_idx]);
        }
    }

    #[inline]
    fn process_smoothers(&mut self, nframes: usize) {
        let prog  = &mut self.prog;

        while let Some((idx, v)) = self.target_refresh.pop() {
            prog.inp[idx].fill(v);
        }

        for (idx, smoother) in
            self.smoothers
                .iter_mut()
                .filter(|s|
                    !s.1.is_done())
        {

            let inp        = &mut prog.inp[*idx];
            let mut last_v = 0.0;

            for frame in 0..nframes {
                let v = smoother.next();

                inp.write(frame, v);
                last_v = v;
            }

            prog.params[*idx] = last_v;
            self.target_refresh.push((*idx, last_v));
        }

    }

    #[inline]
    pub fn process_param_updates(&mut self, nframes: usize) {
        while let Some(upd) = self.shared.quick_update_con.pop() {
            match upd {
                QuickMessage::AtomUpdate { at_idx, value } => {
                    let prog = &mut self.prog;
                    let garbage =
                        std::mem::replace(
                            &mut prog.atoms[at_idx],
                            value);

                    let _ =
                        self.shared.graph_drop_prod.push(
                            DropMsg::Atom { atom: garbage });
                },
                QuickMessage::ParamUpdate { input_idx, value } => {
                    self.set_param(input_idx, value);
                },
                QuickMessage::SetMonitor { bufs } => {
                    self.monitor_signal_cur_inp_indices = bufs;
                },
            }
        }

        self.process_smoothers(nframes);
    }

    #[inline]
    pub fn process<T: NodeAudioContext>(&mut self, ctx: &mut T) {
        // let tb = std::time::Instant::now();

        self.process_param_updates(ctx.nframes());

        let nodes    = &mut self.nodes;
        let ctx_vals = &mut self.shared.node_ctx_values;
        let prog     = &mut self.prog;

        for op in prog.prog.iter() {
            let out = op.out_idxlen;
            let inp = op.in_idxlen;
            let at  = op.at_idxlen;

            let ctx_idx = op.idx as usize * 2;

            nodes[op.idx as usize]
                .process(
                    ctx,
                    &prog.atoms[at.0..at.1],
                    &prog.inp[inp.0..inp.1],
                    &prog.cur_inp[inp.0..inp.1],
                    &mut prog.out[out.0..out.1],
                    &ctx_vals[ctx_idx..ctx_idx + 2]);
        }

        self.shared.monitor_backend.check_recycle();

        // let ta = std::time::Instant::now();

        for (i, idx) in self.monitor_signal_cur_inp_indices.iter().enumerate() {
            if *idx == UNUSED_MONITOR_IDX {
                continue;
            }

            if let Some(mut mon) = self.shared.monitor_backend.get_unused_mon_buf() {
                if i > 2 {
                    mon.feed(i, ctx.nframes(), &prog.out[*idx]);
                } else {
                    mon.feed(i, ctx.nframes(), &prog.cur_inp[*idx]);
                }

                self.shared.monitor_backend.send_mon_buf(mon);
            }
        }

        // let ta = std::time::Instant::now().duration_since(ta);
        // let tb = std::time::Instant::now().duration_since(tb);
        // println!("ta Elapsed: {:?}", ta);
        // println!("tb Elapsed: {:?}", tb);
    }
}
