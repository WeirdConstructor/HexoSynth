// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use crate::UIParams;
use crate::uimsg_queue::{UIMsgQueue, Msg};
use crate::state::State;

use hexodsp::*;
use hexodsp::matrix::MatrixError;
use hexodsp::matrix_repr::save_patch_to_file;

use hexotk::{AtomId, Atom, AtomDataModel};
use hexotk::widgets::{
    DialogModel,
    ListItems,
    TextSourceRef,
};

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;
use std::path::Path;

const MAX_LOG_LINES : usize = 40;

/// Common operations that can be done with the matrix
pub enum UICellTrans {
    /// Swap two cells
    Swap,
    /// Copy source to destination cell
    CopyTo,
    /// Instanciate new cell with the same type as the source cell
    Instanciate,
}

/// This structure holds global information for the UI,
/// such as a reference to the [Matrix] and other stateful
/// informaiton.
///
/// It also provides helper functions for manipulating
/// the [Matrix] and other state.
pub struct UIControl {
    msg_q:              UIMsgQueue,

    dialog_model:       Rc<RefCell<DialogModel>>,
    help_text_src:      Rc<TextSourceRef>,
    log_src:            Rc<TextSourceRef>,
    log:                Vec<String>,

    sample_dir:         std::path::PathBuf,
    path_browse_list:   Vec<std::path::PathBuf>,
    sample_browse_list: ListItems,
    sample_load_id:     AtomId,
    focus_cell:         Cell,
    focus_node_info:    NodeInfo,

    sample_dir_from:    Option<AtomId>,
}

impl UIControl {
    pub fn is_cell_focussed(&self, x: usize, y: usize) -> bool {
        let cell = self.focus_cell;

        if cell.node_id() == NodeId::Nop {
            return false;
        }

        let (cx, cy) = cell.pos();
        cx == x && cy == y
    }

    pub fn set_focus(&mut self, cell: Cell) {
        self.focus_cell = cell;
        self.focus_node_info =
            NodeInfo::from_node_id(cell.node_id());
        let help_txt = self.focus_node_info.help();
        self.help_text_src.set(help_txt);
    }

    pub fn update_log(&mut self) {
        let mut log = self.log.clone();
        log.push("HexoSynth Log".to_string());
        log.reverse();
        self.log_src.set(&log.join("\n"));
    }

    pub fn log(&mut self, source: &str, msg: &str) {
        self.log.push(format!("[{}] {}", source, msg));

        while self.log.len() > MAX_LOG_LINES {
            self.log.remove(0);
        }

    }
}

#[derive(Clone)]
pub struct UICtrlRef(Rc<RefCell<UIControl>>, Arc<Mutex<Matrix>>, Rc<RefCell<State>>);

impl UICtrlRef {
    pub fn new(matrix: Arc<Mutex<Matrix>>,
               dialog_model: Rc<RefCell<DialogModel>>)
        -> UICtrlRef
    {
        UICtrlRef(
            Rc::new(RefCell::new(UIControl {
                msg_q: UIMsgQueue::new(),
                help_text_src:
                    Rc::new(TextSourceRef::new(
                        crate::ui::UI_MAIN_HELP_TEXT_WIDTH)),
                log_src:
                    Rc::new(TextSourceRef::new(
                        crate::ui::UI_MAIN_HELP_TEXT_WIDTH)),
                sample_dir:
                    std::env::current_dir()
                        .unwrap_or_else(|_| std::path::PathBuf::from(".")),
                log:                vec![],
                path_browse_list:   vec![],
                sample_browse_list: ListItems::new(45),
                sample_load_id:     AtomId::from(99999),
                focus_cell:         Cell::empty(NodeId::Nop),
                focus_node_info:    NodeInfo::from_node_id(NodeId::Nop),
                sample_dir_from:    None,
                dialog_model,
            })),
            matrix,
            Rc::new(RefCell::new(State::new())))
    }

    pub fn emit(&self, msg: Msg) {
        self.0.borrow_mut().msg_q.emit(msg);
    }

    pub fn with_state<F, R>(&self, mut f: F) -> R
        where F: FnMut(&State) -> R
    {
        f(&*self.2.borrow_mut())
    }

    pub fn get_help_text_src(&self) -> Rc<TextSourceRef> {
        self.0.borrow().help_text_src.clone()
    }

    pub fn get_log_src(&self) -> Rc<TextSourceRef> {
        self.0.borrow().log_src.clone()
    }

    #[allow(clippy::collapsible_else_if)]
    pub fn reload_sample_dir_list(&self) {
        let mut this = self.0.borrow_mut();
        this.sample_browse_list.clear();
        this.path_browse_list.clear();

        let (lbl, pb) =
            if let Some(parent) = this.sample_dir.parent() {
                ("..".to_string(), parent.to_path_buf())
            } else {
                (".".to_string(), this.sample_dir.clone())
            };

        this.sample_browse_list.push(0, lbl);
        this.path_browse_list.push(pb);

        let mut dir_contents = vec![];

        if let Ok(rd) = std::fs::read_dir(&this.sample_dir) {
            for dir in rd.flatten() {
                let pb = dir.path();

                if pb.is_dir() {
                    if let Some(Some(s)) = pb.file_name().map(|s| s.to_str()) {
                        dir_contents.push((true, s.to_string() + "/", pb));
                    }
                } else {
                    if let Some(Some(ext)) = pb.extension().map(|s| s.to_str()) {
                        if ext == "wav" {
                            if let Some(Some(s)) = pb.file_name().map(|s| s.to_str()) {
                                dir_contents.push((false, s.to_string(), pb));
                            }
                        }
                    }
                }
            }
        }

        dir_contents.sort_by(|a, b| a.1.cmp(&b.1));
        dir_contents.sort_by(|a, b| b.0.cmp(&a.0));

        for (i, (_is_dir, filename, pb)) in dir_contents.into_iter().enumerate() {
            this.sample_browse_list.push((i + 1) as i64, filename);
            this.path_browse_list.push(pb);
        }
    }

    pub fn get_sample_dir_list(&self) -> ListItems {
        self.0.borrow_mut().sample_browse_list.clone()
    }

    pub fn with_matrix_catch_err<F>(&self, mut fun: F)
        where F: FnMut(&mut Matrix) -> Result<(), MatrixError>
    {
        let mut lock = self.1.lock().expect("matrix lockable");
        let this = self.0.borrow_mut();
        catch_err_dialog(this.dialog_model.clone(), move || {
            Ok(fun(&mut *lock)?)
        });
    }

    pub fn with_matrix<F, R>(&self, fun: F) -> R
        where F: FnOnce(&mut Matrix) -> R
    {
        let mut lock = self.1.lock().expect("matrix lockable");
        fun(&mut *lock)
    }

    pub fn clear_cell_ports(&self, mut cell: Cell) {
        let pos      = cell.pos();
        let mut this = self.0.borrow_mut();
        cell.clear_io_dir(CellDir::C);

        catch_err_dialog(this.dialog_model.clone(), || {
            self.with_matrix(|m| {
                m.change_matrix(|matrix| {
                    matrix.place(pos.0, pos.1, cell);
                })?;

                if this.is_cell_focussed(pos.0, pos.1) {
                    if let Some(cell) = m.get_copy(pos.0, pos.1) {
                        this.set_focus(cell);
                    }
                }

                m.sync()?;

                Ok(())
            })
        });
    }

    pub fn assign_cell_port(
        &self, mut cell: Cell, cell_dir: CellDir, idx: Option<usize>)
    {
        if let Some(idx) = idx {
            cell.set_io_dir(cell_dir, idx);
        } else {
            cell.clear_io_dir(cell_dir);
        }
        let pos = cell.pos();

        let mut this = self.0.borrow_mut();

        catch_err_dialog(this.dialog_model.clone(), || {
            self.with_matrix(|m| {
                m.change_matrix(|matrix| {
                    matrix.place(pos.0, pos.1, cell);
                })?;

                if this.is_cell_focussed(pos.0, pos.1) {
                    if let Some(cell) = m.get_copy(pos.0, pos.1) {
                        this.set_focus(cell);
                    }
                }

                m.sync()?;

                Ok(())
            })
        });
    }

    pub fn assign_cell_new_node(
        &self, mut cell: Cell, node_id: NodeId)
    {
        self.with_matrix(|m| {
            let node_id = m.get_unused_instance_node_id(node_id);
            cell.set_node_id(node_id);
            let pos = cell.pos();

            let mut this = self.0.borrow_mut();

            catch_err_dialog(this.dialog_model.clone(), || {
                m.change_matrix(|matrix| {
                    matrix.place(pos.0, pos.1, cell);
                })?;

                if this.is_cell_focussed(pos.0, pos.1) {
                    if let Some(cell) = m.get_copy(pos.0, pos.1) {
                        this.set_focus(cell);
                    }
                }

                m.sync()?;
                Ok(())
            });

        });
    }

    pub fn save_patch(&self) {
        let this = self.0.borrow();

        self.with_matrix(|m| {
            catch_err_dialog(this.dialog_model.clone(), || {
                match save_patch_to_file(m, "init.hxy") {
                    Ok(_) => Ok(()),
                    Err(e) => Err(PatchSaveError {
                        path:  "init.hxy".to_string(),
                        error: e
                    }.into()),
                }
            });
        });
    }

    pub fn cell_transform(
        &self, src_pos: (usize, usize), dst_pos: (usize, usize), transform: UICellTrans) -> Option<()>
    {
        let (mut src_cell, dst_cell) =
            self.with_matrix(|m| {
                Some((
                    m.get_copy(src_pos.0, src_pos.1)?,
                    m.get_copy(dst_pos.0, dst_pos.1)?
                ))
            })?;

        if self.is_cell_focussed(src_pos.0, src_pos.1) {
            self.set_focus(src_cell.with_pos_of(dst_cell));
        }

        let this = self.0.borrow();

        self.with_matrix(|m| {
            catch_err_dialog(this.dialog_model.clone(), || {
                match transform {
                    UICellTrans::Swap => {
                        m.change_matrix(|m| {
                            m.place(dst_pos.0, dst_pos.1, src_cell);
                            m.place(src_pos.0, src_pos.1, dst_cell);
                        })?;
                        m.sync()?;
                    },
                    UICellTrans::CopyTo => {
                        m.change_matrix(|m| {
                            m.place(dst_pos.0, dst_pos.1, src_cell);
                        })?;
                        m.sync()?;
                    },
                    UICellTrans::Instanciate => {
                        let unused_id =
                            m.get_unused_instance_node_id(src_cell.node_id());
                        src_cell.set_node_id(unused_id);
                        m.change_matrix(|m| {
                            m.place(dst_pos.0, dst_pos.1, src_cell);
                        })?;
                        m.sync()?;
                    },
                }

                Ok(())
            })
        });

        Some(())
    }

    pub fn get_recent_focus(&self) -> Cell {
        self.0.borrow().focus_cell
    }

    pub fn get_focus_node_info(&self) -> NodeInfo {
        self.0.borrow_mut().focus_node_info.clone()
    }

    pub fn get_focus_id(&self) -> NodeId {
        self.0.borrow().focus_cell.node_id()
    }

    pub fn is_cell_focussed(&self, x: usize, y: usize) -> bool {
        self.0.borrow().is_cell_focussed(x, y)
    }

    pub fn clear_focus(&self) {
        self.0.borrow_mut().focus_cell = Cell::empty(NodeId::Nop);
    }

    pub fn set_focus(&self, cell: Cell) {
        let nid = cell.node_id();
        self.0.borrow_mut().set_focus(cell);

        if nid.to_instance(0) == NodeId::Sampl(0) {
            let uniq_id =
                self.with_matrix(|m|
                    m.unique_index_for(&nid).unwrap_or(0) as u32);

            if let Some(pid) = nid.inp_param("sample") {
                self.0.borrow_mut().sample_dir_from =
                    Some(AtomId::new(uniq_id, pid.inp().into()));
            }
        }
    }

    pub fn check_atoms(&mut self, atoms: &dyn AtomDataModel) {
        let at_id_dir = self.0.borrow_mut().sample_dir_from.take();

        if let Some(at_id_dir) = at_id_dir {
            let sampl = atoms.get(at_id_dir);

            if let Some(Atom::AudioSample((path, _))) = sampl {
                let path = Path::new(path);

                if let Some(path) = path.parent() {
                    self.navigate_sample_dir(path);
                }
            }
        }

        {
            use hexodsp::log::retrieve_log_messages;
            let mut new_msg = false;
            retrieve_log_messages(|name, s| {
                new_msg = true;
                self.0.borrow_mut().log(name, s)
            });

            if new_msg {
                self.0.borrow_mut().update_log();
            }
        }

    }

    pub fn set_sample_load_id(&self, id: AtomId) {
        self.0.borrow_mut().sample_load_id = id;
    }

    pub fn navigate_sample_dir(&mut self, path: &Path) {
        if path != self.0.borrow().sample_dir {
            self.0.borrow_mut().sample_dir = path.to_path_buf();
            self.reload_sample_dir_list();
        }
    }

    /// Lets the UI emit a set event for a specific [AtomId].
    /// Should return true if the value should be saved in the
    /// variables register.
    pub fn set_event(&self, ui_params: &mut UIParams, id: AtomId, atom: Atom) -> bool {
        if id.node_id() == crate::state::ATNID_SAMPLE_LOAD_ID {
            let idx = atom.i() as usize;

            let mut load_file = None;
            let mut do_reload = false;

            {
                let mut this = self.0.borrow_mut();

                let mut new_sample_dir = None;

                if let Some(pb) = this.path_browse_list.get(idx) {
                    if pb.is_dir() {
                        new_sample_dir = Some(pb.clone());
                    } else {
                        load_file = Some(pb.clone());
                    }
                }

                if let Some(pb) = new_sample_dir {
                    do_reload       = this.sample_dir != pb;
                    this.sample_dir = pb;
                }
            }

            if do_reload {
                self.reload_sample_dir_list();
            }

            if let Some(file) = load_file {
                if let Some(path_str) = file.to_str() {
                    let load_id = self.0.borrow().sample_load_id;
                    ui_params.set(load_id, Atom::audio_unloaded(path_str));
                }
            }

        } else if id.node_id() == crate::state::ATNID_HELP_BUTTON {
            if atom.i() == 1 {
                self.emit(Msg::ui_btn(id.node_id()));
            }
            return false;
        }

        true
    }

    pub fn ui_start_frame(&self, ui_params: &mut UIParams) {
        let error = self.with_matrix(|m| m.pop_error());
        if let Some(error) = error {
            self.0.borrow().dialog_model.borrow_mut().open(
                &error, Box::new(|_| ()));

        }

        self.with_matrix(|m| m.update_filters());

        while self.0.borrow_mut().msg_q.has_new_messages() {
            let events = self.0.borrow_mut().msg_q.start_work();
            if let Some(events) = events {
                for e in events.iter() {
                    self.with_matrix(|m|
                        self.2.borrow_mut().apply(ui_params, m, e));
                }
                self.0.borrow_mut().msg_q.end_work(events);
            }
        }
    }
}

pub struct PatchSaveError {
    path:   String,
    error:  std::io::Error,
}

pub enum DialogMessage {
    MatrixError(MatrixError),
    IOError(std::io::Error),
    PatchSaveError(PatchSaveError),
}

impl From<MatrixError> for DialogMessage {
    fn from(error: MatrixError) -> Self {
        DialogMessage::MatrixError(error)
    }
}

impl From<std::io::Error> for DialogMessage {
    fn from(error: std::io::Error) -> Self {
        DialogMessage::IOError(error)
    }
}

impl From<PatchSaveError> for DialogMessage {
    fn from(error: PatchSaveError) -> Self {
        DialogMessage::PatchSaveError(error)
    }
}

pub fn catch_err_dialog<F>(dialog: Rc<RefCell<DialogModel>>, mut f: F)
    where F: FnMut() -> Result<(), DialogMessage>
{
    match f() {
        Err(DialogMessage::PatchSaveError(err)) => {
            dialog.borrow_mut().open(
                &format!("Patch Saving failed!\n\
                    Path: {}\n\
                    Error: {}\n", err.path, err.error),
                Box::new(|_| ()));
        },
        Err(DialogMessage::IOError(err)) => {
            dialog.borrow_mut().open(
                &format!("An Unknown I/O Error Occured!\n\
                    Error: {}\n", err),
                Box::new(|_| ()));
        },
        Err(DialogMessage::MatrixError(err)) => {
            match err {
                MatrixError::CycleDetected => {
                    dialog.borrow_mut().open(
                        &"Cycle Detected!\n\
                            HexoSynth does not allow to create cyclic configurations.\n\
                            \n\
                            For feedback please use the nodes:\n\
                            * 'FbWr' (Feedback Writer)\n\
                            * 'FbRd' (Feedback Reader)",
                        Box::new(|_| ()));
                },
                MatrixError::DuplicatedInput { output1, output2 } => {
                    dialog.borrow_mut().open(
                        &format!("Unjoined Outputs Detected!\n\
                            It's not possible to assign to an input port twice.\n\
                            Please use a mixer or some other kind of node to join the outputs.\n\
                            \n\
                            Conflicting Outputs:\n\
                            * {} {}, port {}\n\
                            * {} {}, port {}",
                            output1.0.name(),
                            output1.0.instance(),
                            output1.0.out_name_by_idx(output1.1).unwrap_or("???"),
                            output2.0.name(),
                            output2.0.instance(),
                            output2.0.out_name_by_idx(output2.1).unwrap_or("???")),
                        Box::new(|_| ()));
                }
            }
        },
        Ok(_) => (),
    }
}
