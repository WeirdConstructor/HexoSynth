// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::UIParams;
use crate::uimsg_queue::{UIMsgQueue, Msg};
use crate::state::State;

use crate::actions::{DefaultActionHandler, ActionHandler, catch_err_dialog};

use hexodsp::*;
use hexodsp::matrix::MatrixError;

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
    log_src:            Rc<TextSourceRef>,
    log:                Vec<String>,

    sample_dir:         std::path::PathBuf,
    path_browse_list:   Vec<std::path::PathBuf>,
    sample_browse_list: ListItems,
}

impl UIControl {
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
pub struct UICtrlRef(
    Rc<RefCell<UIControl>>,
    Arc<Mutex<Matrix>>,
    Rc<RefCell<State>>,
    Rc<RefCell<Option<Box<dyn ActionHandler>>>>
);

impl UICtrlRef {
    pub fn new(matrix: Arc<Mutex<Matrix>>,
               dialog_model: Rc<RefCell<DialogModel>>)
        -> UICtrlRef
    {
        UICtrlRef(
            Rc::new(RefCell::new(UIControl {
                msg_q: UIMsgQueue::new(),
                log_src:
                    Rc::new(TextSourceRef::new(
                        crate::ui::UI_MAIN_HELP_TEXT_WIDTH)),
                sample_dir:
                    std::env::current_dir()
                        .unwrap_or_else(|_| std::path::PathBuf::from(".")),
                log:                vec![],
                path_browse_list:   vec![],
                sample_browse_list: ListItems::new(45),
                dialog_model:       dialog_model.clone(),
            })),
            matrix,
            Rc::new(RefCell::new(State::new())),
            Rc::new(RefCell::new(Some(Box::new(DefaultActionHandler::new())))))
    }

    pub fn emit(&self, msg: Msg) {
        self.0.borrow_mut().msg_q.emit(msg);
    }

    pub fn with_state<F, R>(&self, mut f: F) -> R
        where F: FnMut(&State) -> R
    {
        f(&*self.2.borrow_mut())
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

    pub fn check_atoms(&self, atoms: &dyn AtomDataModel) {
        let at_id_dir = self.2.borrow_mut().sample_dir_from.take();

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

    pub fn navigate_sample_dir(&self, path: &Path) {
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
                    let load_id = self.2.borrow().sample_load_id;
                    ui_params.set(load_id, Atom::audio_unloaded(path_str));
                }
            }

        } else if id.node_id() == crate::state::ATNID_HELP_BUTTON {
            if atom.i() == 1 {
                self.emit(Msg::ui_btn(id.node_id()));
            }
            return false;

        } else if id.node_id() == crate::state::ATNID_SAVE_BUTTON {
            if atom.i() == 1 {
                self.emit(Msg::ui_btn(id.node_id()));
            }
            return false;

        } else if id.node_id() == crate::state::ATNID_CLR_SELECT {
            self.emit(Msg::clr_sel(atom.i()));
            return true;
        }

        true
    }

    pub fn ui_start_frame(&self, ui_params: &mut UIParams) {
        let error = self.with_matrix(|m| m.pop_error());
        if let Some(error) = error {
            self.0.borrow().dialog_model.borrow_mut().open(
                &error, Box::new(|_| ()));

        }

        self.check_atoms(ui_params);

        self.with_matrix(|m| {
            m.check_pattern_data(self.2.borrow().current_tracker_idx);
            m.update_filters()
        });

        let dialog = self.0.borrow().dialog_model.clone();

        let mut action_handler = self.3.borrow_mut().take();

//        std::thread::sleep(
//            std::time::Duration::from_millis(100));

        while self.0.borrow_mut().msg_q.has_new_messages() {
            let messages = self.0.borrow_mut().msg_q.start_work();

            if let Some(messages) = messages {
                //d// println!("CHECK UI MSGS {}", messages.len());
                for msg in messages.iter() {
                    //d// println!("CHECK UI MSG {:?}", msg);
                    self.with_matrix(|matrix| {
                        let mut a = crate::actions::ActionState {
                            state:  &mut *self.2.borrow_mut(),
                            dialog: dialog.clone(),
                            matrix,
                            ui_params,
                            action_handler: action_handler.take(),
                        };
                        a.exec(msg);
                        action_handler = a.action_handler.take();
                    });
                }

                //d// println!("CHECK UI MSGS END {}", messages.len());
                self.0.borrow_mut().msg_q.end_work(messages);
            }
        }
        //d// println!("CHECK UI MSGS END REALLY");

        *self.3.borrow_mut() = action_handler;
    }

    pub fn init(&self, ui_params: &mut UIParams) {
        let dialog = self.0.borrow().dialog_model.clone();

        self.with_matrix(|matrix| {
            let mut a = crate::actions::ActionState {
                state:  &mut *self.2.borrow_mut(),
                dialog: dialog.clone(),
                matrix,
                ui_params,
                action_handler: None,
            };
            a.init();
        });
    }

}

