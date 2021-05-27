// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use crate::UIParams;

use hexodsp::*;
use hexodsp::matrix::MatrixError;
use hexodsp::matrix_repr::save_patch_to_file;

use hexotk::{AtomId, Atom, AtomDataModel};
use hexotk::widgets::{
    DialogModel,
    ListItems,
};

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

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
    dialog_model:       Rc<RefCell<DialogModel>>,

    sample_dir:         std::path::PathBuf,
    path_browse_list:   Vec<std::path::PathBuf>,
    sample_browse_list: ListItems,
    sample_load_id:     AtomId,
    focus_cell:         Cell,
}

#[derive(Clone)]
pub struct UICtrlRef(Rc<RefCell<UIControl>>, Arc<Mutex<Matrix>>);

impl UICtrlRef {
    pub const ATNID_SAMPLE_LOAD_ID : u32 = 190001;

    pub fn new(matrix: Arc<Mutex<Matrix>>,
               dialog_model: Rc<RefCell<DialogModel>>)
        -> UICtrlRef
    {
        UICtrlRef(
            Rc::new(RefCell::new(UIControl {
                dialog_model,
                path_browse_list:   vec![],
                sample_dir:
                    std::env::current_dir()
                        .unwrap_or(std::path::PathBuf::from(".")),
                sample_browse_list: ListItems::new(45),
                sample_load_id:     AtomId::from(99999),
                focus_cell:         Cell::empty(NodeId::Nop),
            })),
            matrix)
    }

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

        let mut i = 1;
        if let Ok(rd) = std::fs::read_dir(&this.sample_dir) {
            for entry in rd {
                if let Ok(dir) = entry {
                    let pb = dir.path();

                    if pb.is_dir() {
                        if let Some(Some(s)) = pb.file_name().map(|s| s.to_str()) {
                            this.sample_browse_list.push(i, s.to_string() + "/");
                            this.path_browse_list.push(pb);
                            i += 1;
                        }
                    } else {
                        if let Some(Some(ext)) = pb.extension().map(|s| s.to_str()) {
                            if ext == "wav" {
                                if let Some(Some(s)) = pb.file_name().map(|s| s.to_str()) {
                                    this.sample_browse_list.push(i, s.to_string());
                                    this.path_browse_list.push(pb);
                                    i += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_sample_dir_list(&self) -> ListItems {
        self.0.borrow_mut().sample_browse_list.clone()
    }

    pub fn with_matrix<F, R>(&self, fun: F) -> R
        where F: FnOnce(&mut Matrix) -> R
    {
        let mut lock = self.1.lock().expect("matrix lockable");
        fun(&mut *lock)
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

        let this = self.0.borrow();

        catch_err_dialog(&this.dialog_model, || {
            self.with_matrix(|m| {
                m.change_matrix(|matrix| {
                    matrix.place(pos.0, pos.1, cell);
                })?;

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

            let this = self.0.borrow();

            catch_err_dialog(&this.dialog_model, || {
                m.change_matrix(|matrix| {
                    matrix.place(pos.0, pos.1, cell);
                })?;

                m.sync()?;
                Ok(())
            });
        });
    }

    pub fn save_patch(&self) {
        let this = self.0.borrow();

        self.with_matrix(|m| {
            catch_err_dialog(&this.dialog_model, || {
                match save_patch_to_file(m, "init.hxy") {
                    Ok(_) => Ok(()),
                    Err(e) => Err(PatchSaveError {
                        path:  "init.hxy".to_string(),
                        error: e
                    })?,
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
            catch_err_dialog(&this.dialog_model, || {
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

    pub fn is_cell_focussed(&self, x: usize, y: usize) -> bool {
        let cell = self.0.borrow().focus_cell;

        if cell.node_id() == NodeId::Nop {
            return false;
        }

        let (cx, cy) = cell.pos();
        cx == x && cy == y
    }

    pub fn clear_focus(&self) {
        self.0.borrow_mut().focus_cell = Cell::empty(NodeId::Nop);
    }

    pub fn set_focus(&self, cell: Cell) {
        self.0.borrow_mut().focus_cell = cell;
    }

    pub fn set_sample_load_id(&self, id: AtomId) {
        self.0.borrow_mut().sample_load_id = id;
    }

    /// Lets the UI emit a set event for a specific [AtomId].
    /// Should return true if the value should be saved in the
    /// variables register.
    pub fn set_event(&self, ui_params: &mut UIParams, id: AtomId, atom: Atom) -> bool {
        if id.node_id() == Self::ATNID_SAMPLE_LOAD_ID {
            println!("SET SAMPLE={:?}", atom);

            let idx = atom.i() as usize;

            {
                let mut this = self.0.borrow_mut();

                let prev_dir = this.sample_dir.clone();
                this.sample_dir =
                    if let Some(pb) = this.path_browse_list.get(idx) { pb.clone() }
                    else { prev_dir };
            }

            self.reload_sample_dir_list();

            let load_id = self.0.borrow().sample_load_id;
            ui_params.set(load_id, atom);
        }

        true
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

pub fn catch_err_dialog<F>(dialog: &Rc<RefCell<DialogModel>>, mut f: F)
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
                        &format!("Cycle Detected!\n\
                            HexoSynth does not allow to create cyclic configurations.\n\
                            \n\
                            For feedback please use the nodes:\n\
                            * 'FbWr' (Feedback Writer)\n\
                            * 'FbRd' (Feedback Reader)"),
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
