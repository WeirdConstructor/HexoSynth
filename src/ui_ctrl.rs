// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of HexoSynth. Released under (A)GPLv3 or any later.
// See README.md and COPYING for details.

use hexodsp::*;
use hexodsp::matrix::MatrixError;

use hexotk::widgets::DialogModel;

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

/// This structure holds global information for the UI,
/// such as a reference to the [Matrix] and other stateful
/// informaiton.
///
/// It also provides helper functions for manipulating
/// the [Matrix] and other state.
pub struct UIControl {
    matrix:         Arc<Mutex<Matrix>>,
    dialog_model:   Rc<RefCell<DialogModel>>,
}

#[derive(Clone)]
pub struct UICtrlRef(Rc<RefCell<UIControl>>);

impl UICtrlRef {
    pub fn new(matrix: Arc<Mutex<Matrix>>,
               dialog_model: Rc<RefCell<DialogModel>>)
        -> UICtrlRef
    {
        UICtrlRef(
            Rc::new(RefCell::new(UIControl {
                matrix,
                dialog_model,
            })))
    }

//    pub fn lock_matrix(&self) -> std::sync::MutexGuard<'_, Matrix> {
//        self.0.borrow().matrix.lock().unwrap()
//    }
//
    pub fn with_matrix<F, R>(&self, fun: F) -> R
        where F: FnOnce(&mut Matrix) -> R
    {
        let ctrl = self.0.borrow();
        let mut lock = ctrl.matrix.lock().unwrap();
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

        handle_matrix_change(&this.dialog_model, || {
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

            handle_matrix_change(&this.dialog_model, || {
                m.change_matrix(|matrix| {
                    matrix.place(pos.0, pos.1, cell);
                })?;

                m.sync()?;
                Ok(())
            });
        });
    }
}

pub enum DialogMessage {
    MatrixError(MatrixError),
}

impl From<MatrixError> for DialogMessage {
    fn from(error: MatrixError) -> Self {
        DialogMessage::MatrixError(error)
    }
}

pub fn handle_matrix_change<F>(dialog: &Rc<RefCell<DialogModel>>, mut f: F)
    where F: FnMut() -> Result<(), DialogMessage>
{
    match f() {
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
