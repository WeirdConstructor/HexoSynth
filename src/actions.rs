// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::uimsg_queue::{Msg};
use crate::state::{State, ItemType, MenuState, RandSpecifier, ATNID_HELP_BUTTON};
use crate::UIParams;
use crate::dsp::{SAtom, get_rand_node_id, RandNodeSelector};
use crate::dsp::helpers::SplitMix64;

use hexotk::{MButton, AtomId, AtomDataModel};
use hexotk::widgets::{
    DialogModel,
};
use hexodsp::{Matrix, CellDir, NodeId, Cell, NodeInfo};
use keyboard_types::Key;
use hexodsp::matrix::MatrixError;

use std::rc::Rc;
use std::cell::RefCell;

pub trait ActionHandler {
    fn init(&mut self, _actions: &mut ActionState) { }
    fn step(&mut self, _actions: &mut ActionState, _msg: &Msg) -> bool { false }
    fn menu_select(
        &mut self, _actions: &mut ActionState, _ms: MenuState,
        _item_type: ItemType)
    { }
    fn is_finished(&self) -> bool { false }
    fn get_followup_action(&mut self, _actions: &mut ActionState)
        -> Option<Box<dyn ActionHandler>>
    {
        None
    }
}

pub struct ActionState<'a, 'b, 'c> {
    pub state:          &'a mut State,
    pub dialog:         Rc<RefCell<DialogModel>>,
    pub matrix:         &'b mut Matrix,
    pub ui_params:      &'c mut UIParams,
    pub action_handler: Option<Box<dyn ActionHandler>>,
}

impl ActionState<'_, '_, '_> {
    pub fn save_patch(&mut self) {
        use hexodsp::matrix_repr::save_patch_to_file;

        let diag = self.dialog.clone();

        if catch_err_dialog(self.dialog.clone(), || {
            match save_patch_to_file(self.matrix, "init.hxy") {
                Ok(_) => Ok(()),
                Err(e) => Err(PatchSaveError {
                    path:  "init.hxy".to_string(),
                    error: e
                }.into()),
            }
        }) {
            let cwd =
                std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."));

            diag.borrow_mut().open(
                &format!("Patch saved!\nPatch saved successful to 'init.hxy'!\nTo the directory: {}",
                    cwd.to_str().unwrap_or("?")),
                Box::new(|_| ()));
        }
    }

    pub fn toggle_help(&mut self) {
        self.state.toggle_help();
    }

    pub fn escape_dialogs(&mut self) {
        if self.state.show_help {
            self.state.toggle_help();
        }
    }

    pub fn swap_cells(&mut self, pos_a: (usize, usize), pos_b: (usize, usize)) {
        let (src_cell, dst_cell) = (
            self.matrix.get_copy(pos_a.0, pos_a.1),
            self.matrix.get_copy(pos_b.0, pos_b.1)
        );

        let src_cell =
            if let Some(src_cell) = src_cell { src_cell }
            else { return; };
        let dst_cell =
            if let Some(dst_cell) = dst_cell { dst_cell }
            else { return; };

        catch_err_dialog(self.dialog.clone(), || {
            self.matrix.change_matrix(|m| {
                m.place(pos_b.0, pos_b.1, src_cell);
                m.place(pos_a.0, pos_a.1, dst_cell);
            })?;
            self.matrix.sync()?;
            Ok(())
        });
    }

    pub fn make_copy_at(&mut self, pos: (usize, usize), node_id: NodeId) {
        catch_err_dialog(self.dialog.clone(), || {
            if let Some(mut cell) = self.matrix.get_copy(pos.0, pos.1) {
                cell.set_node_id(node_id);
                self.matrix.change_matrix(|m| {
                    m.place(pos.0, pos.1, cell);
                })?;
                self.matrix.sync()?;
            }
            Ok(())
        });
    }

    pub fn instanciate_node_at(
        &mut self, pos: (usize, usize), node_id: NodeId
    ) {
        catch_err_dialog(self.dialog.clone(), || {
            if let Some(mut cell) = self.matrix.get_copy(pos.0, pos.1) {
                let unused_id =
                    self.matrix.get_unused_instance_node_id(node_id);
                cell.set_node_id(unused_id);
                self.matrix.change_matrix(|m| {
                    m.place(pos.0, pos.1, cell);
                })?;
                self.matrix.sync()?;
            }
            Ok(())
        });
    }

    pub fn clear_cell_at(&mut self, pos: (usize, usize)) {
        catch_err_dialog(self.dialog.clone(), || {
            self.matrix.change_matrix(|m| {
                m.place(pos.0, pos.1, Cell::empty(NodeId::Nop));
            })?;
            self.matrix.sync()?;
            Ok(())
        });
    }

    pub fn clear_unused_at(&mut self, pos: (usize, usize)) {
        catch_err_dialog(self.dialog.clone(), || {
            self.matrix.change_matrix(|m| {
                if let Some(mut cell) = m.get_copy(pos.0, pos.1) {
                    for e in 0..6 {
                        let dir = CellDir::from(e);
                        if cell.is_port_dir_connected(m, dir).is_none() {
                            cell.clear_io_dir(dir);
                        }
                    }
                    m.place_cell(cell);
                }
            })?;
            self.matrix.sync()?;
            Ok(())
        });
    }

    pub fn set_connection(
        &mut self,
        dir: CellDir, mut cell_a: Cell, cell_a_io: Option<usize>,
        mut cell_b: Cell, cell_b_io: Option<usize>)
    {
        catch_err_dialog(self.dialog.clone(), || {
            if let Some(cell_a_io) = cell_a_io {
                cell_a.set_io_dir(dir, cell_a_io);
            }

            if let Some(cell_b_io) = cell_b_io {
                cell_b.set_io_dir(dir.flip(), cell_b_io);
            }

            self.matrix.change_matrix(|m| {
                m.place_cell(cell_a);
                m.place_cell(cell_b);
            })?;

            self.matrix.sync()?;
            Ok(())
        });
    }

    pub fn instanciate_two_nodes_with_connection(
        &mut self, pos_a: (usize, usize), pos_b: (usize, usize),
        node_a: NodeId, node_b: NodeId,
        io_a: Option<usize>, io_b: Option<usize>,
        dir: CellDir
    ) {
        catch_err_dialog(self.dialog.clone(), || {
            self.matrix.change_matrix(move |m| {
                let cell_a = m.get_copy(pos_a.0, pos_a.1);
                let cell_b = m.get_copy(pos_b.0, pos_b.1);

                if let (Some(mut cell_a), Some(mut cell_b)) = (cell_a, cell_b) {
                    let node_a = m.get_unused_instance_node_id(node_a);
                    let node_b = m.get_unused_instance_node_id(node_b);

                    cell_a.set_node_id(node_a);
                    cell_b.set_node_id(node_b);

                    if let Some(io_a) = io_a {
                        cell_a.set_io_dir(dir, io_a);
                    }

                    if let Some(io_b) = io_b {
                        cell_b.set_io_dir(dir.flip(), io_b);
                    }

                    m.place_cell(cell_a);
                    m.place_cell(cell_b);
                }
            })?;

            self.matrix.sync()?;
            Ok(())
        });
    }

    /// Instanciates a node and tries to connect it to the adjacent cell.
    /// If this fails due to the Two-Output-to-One-Input error case,
    /// a false value is returned. Then the caller needs to find alternative
    /// ways (eg. open the menu für explicit connection).
    pub fn instanciate_node_at_with_connection(
        &mut self, pos: (usize, usize), node_id: NodeId, copy: bool,
        dir: CellDir, new_idx: Option<usize>, mut adj_cell: Cell, adj_idx: Option<usize>,
    ) -> bool {
        //d// println!("Instance New {:?}, {:?}, {:?}",
        //d//     node_id, new_idx, adj_idx);
        let mut ret = true;

        catch_err_dialog(self.dialog.clone(), || {
            if let Some(mut cell) = self.matrix.get_copy(pos.0, pos.1) {
                let node_id =
                    if copy { node_id }
                    else { self.matrix.get_unused_instance_node_id(node_id) };

                cell.set_node_id(node_id);
                self.matrix.change_matrix(|m| {
                    m.place(pos.0, pos.1, cell);
                })?;
                self.matrix.sync()?;

                if let Some(new_idx) = new_idx {
                    cell.set_io_dir(dir, new_idx);
                }
                if let Some(adj_idx) = adj_idx {
                    adj_cell.set_io_dir(dir.flip(), adj_idx);
                }

                let edge_res =
                    self.matrix.change_matrix(|m| {
                        m.place(pos.0, pos.1, cell);
                        m.place_cell(adj_cell);
                    });

                match edge_res {
                    Err(MatrixError::DuplicatedInput { .. }) => { ret = false; },
                    Err(e) => { return Err(DialogMessage::MatrixError(e)); },
                    Ok(_)  => { ret = true; },
                }

                if ret {
                    let res = self.matrix.sync();
                    match res {
                        Err(MatrixError::DuplicatedInput { .. }) => { ret = false; },
                        Err(e) => { return Err(DialogMessage::MatrixError(e)); },
                        Ok(_)  => { ret = true; },
                    }
                }
            }
            Ok(())
        });

        ret
    }

    pub fn move_cluster_from_to(
        &mut self, pos_a: (usize, usize), pos_b: (usize, usize))
    {
        let path = CellDir::path_from_to(pos_a, pos_b);

        catch_err_dialog(self.dialog.clone(), || {
            let mut cluster = crate::cluster::Cluster::new();

            self.matrix.change_matrix_err(|m| {
                cluster.add_cluster_at(m, pos_a);

                cluster.remove_cells(m);
                cluster.move_cluster_cells_dir_path(&path)?;
                cluster.place(m)?;

                Ok(())
            })?;

            self.matrix.sync()?;

            Ok(())
        });
    }

    pub fn split_cluster_at(
        &mut self, pos_a: (usize, usize), pos_b: (usize, usize))
    {
        if let Some(dir) = CellDir::are_adjacent(pos_a, pos_b) {
            catch_err_dialog(self.dialog.clone(), || {
                let mut cluster = crate::cluster::Cluster::new();

                self.matrix.change_matrix_err(|m| {
                    cluster.ignore_pos(pos_b);
                    cluster.add_cluster_at(m, pos_a);

                    cluster.remove_cells(m);
                    cluster.move_cluster_cells_dir_path(&[dir.flip()])?;
                    cluster.place(m)?;

                    Ok(())
                })?;

                self.matrix.sync()?;

                Ok(())
            });
        }
    }

    pub fn set_focus_at(&mut self, x: usize, y: usize) {
        if let Some(cell) = self.matrix.get_copy(x, y) {
            self.set_focus(cell);
        }
    }

    fn update_sample_load_id(&mut self, node_id: NodeId) {
        let mut idx = 0;
        while let Some(param_id) = node_id.param_by_idx(idx) {
            if let SAtom::AudioSample((_filename, _)) =
                param_id.as_atom_def()
            {
                self.state.sample_load_id =
                    AtomId::new(self.state.focus_uniq_node_idx, idx as u32);
            }

            idx += 1;
        }
    }

    pub fn set_node_color(&mut self, node_id: NodeId, clr: i64) {
        if clr > 255 {
            self.state.node_colors.remove(&node_id);
        } else {
            self.state.node_colors.insert(node_id, clr as u8);
        }
        self.state.sync_to_matrix(self.matrix);
    }

    pub fn clear_menu_history(&mut self) {
        self.state.menu_history.clear();
    }

    pub fn push_menu_history(&mut self, ms: MenuState, title: String) {
        self.state.menu_history.push((ms, title));
    }

    pub fn menu_back(&mut self) {
        // Clicking on "Back" creates a new history entry, so we
        // skip that one here:
        self.state.menu_history.pop();
        if let Some((ms, title)) = self.state.menu_history.pop() {
            self.next_menu_state(ms, title);
        }
    }

    pub fn next_menu_state(&mut self, ms: MenuState, title: String) {
        self.state.menu_state = ms.clone();
        *self.state.menu_title.borrow_mut() = title;
        self.state.menu_items = self.state.menu_state.to_items();
    }

    fn update_pattern_edit(&mut self) {
        let patedit_ui = self.state.widgets.patedit_ui.clone();
        let mut pe = patedit_ui.borrow_mut();
        *pe = Some(crate::ui::util_panel::create_pattern_edit(self));
    }

    pub fn set_focus(&mut self, cell: Cell) {
        let node_id = cell.node_id();

        self.state.focus_cell      = cell;
        self.state.focus_node_info = NodeInfo::from_node_id(node_id);

        let help_txt = self.state.focus_node_info.help();
        self.state.help_text_src.set(help_txt);

        self.matrix.monitor_cell(cell);

        self.state.focus_uniq_node_idx =
            self.matrix
                .unique_index_for(&node_id)
                .unwrap_or(0) as u32;

        self.update_sample_load_id(node_id);

        if node_id.to_instance(0) == NodeId::Sampl(0) {
            let uniq_id = self.state.focus_uniq_node_idx;

            if let Some(pid) = node_id.inp_param("sample") {
                self.state.sample_dir_from =
                    Some(AtomId::new(uniq_id, pid.inp().into()));
            }

        } else if node_id.to_instance(0) == NodeId::TSeq(0) {
            self.state.current_tracker_idx = node_id.instance();
            self.update_pattern_edit();
        }

        self.ui_params.set_var(
            hexotk::AtomId::new(crate::state::ATNID_CLR_SELECT, 0),
            hexotk::Atom::Setting(self.state.color_for_node(node_id) as i64));

        let node_ui = self.state.widgets.node_ui.clone();
        node_ui.borrow_mut().set_target(
            cell.node_id(),
            self.state.focus_uniq_node_idx,
            self);
    }

    pub fn init(&mut self) {
        self.update_pattern_edit();
    }

    pub fn exec(&mut self, msg: &Msg) -> bool {
        let ah = self.action_handler.take();

        let mut handled = false;

        if let Some(mut ah) = ah {
            handled = ah.step(self, msg);

            if !ah.is_finished() {
                self.action_handler = Some(ah);
            }
        }

        handled
    }
}

struct ActionNewNodeAtCell {
    x: usize,
    y: usize,
}

impl ActionNewNodeAtCell {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl ActionHandler for ActionNewNodeAtCell {
    fn init(&mut self, a: &mut ActionState) {
        a.next_menu_state(
            MenuState::SelectCategory { user_state: 0 },
            "Category for new node".to_string());
    }

    fn menu_select(&mut self, a: &mut ActionState, ms: MenuState, item_type: ItemType) {
        match item_type {
            ItemType::Back => { a.menu_back(); },
            ItemType::Category(category) => {
                a.next_menu_state(
                    MenuState::SelectNodeIdFromCat { category, user_state: 0 },
                    "Select new node".to_string());
            },
            ItemType::NodeId(node_id) => {
                if let MenuState::SelectNodeIdFromCat { .. } = ms {
                    a.instanciate_node_at((self.x, self.y), node_id);
                    a.set_focus_at(self.x, self.y);
                }
            },
            _ => ()
        }
    }
}

struct ActionContextMenu {
    x: usize,
    y: usize,
    empty_cell: bool,
}

impl ActionContextMenu {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y, empty_cell: false }
    }

    pub fn new_empty_cell(x: usize, y: usize) -> Self {
        Self { x, y, empty_cell: true }
    }

    pub fn get_cell(&self, a: &mut ActionState) -> Cell {
        a.matrix.get_copy(self.x, self.y)
            .unwrap_or_else(|| Cell::empty(NodeId::Nop))
    }
}

impl ActionHandler for ActionContextMenu {
    fn init(&mut self, a: &mut ActionState) {
        let cell      = self.get_cell(a);
        let node_id   = cell.node_id();
        let node_info = NodeInfo::from_node_id(node_id);

        if self.empty_cell {
            a.next_menu_state(
                MenuState::ContextActionPos { pos: (self.x, self.y) },
                format!("Context Action for cell at {},{}",
                    cell.pos().0,
                    cell.pos().1));

        } else {
            a.next_menu_state(
                MenuState::ContextAction { cell, node_id, node_info },
                format!("Context Action for {} at {},{}",
                    node_id.label(),
                    cell.pos().0,
                    cell.pos().1));
        }
    }

    fn menu_select(&mut self, a: &mut ActionState, ms: MenuState, item_type: ItemType) {
        match item_type {
            ItemType::Back => { a.menu_back(); },
            ItemType::SubMenu { menu_state, title } => {
                a.next_menu_state(*menu_state, title);
            },
            ItemType::Delete => {
                a.clear_cell_at((self.x, self.y));
                a.set_focus_at(self.x, self.y);
            },
            ItemType::Help(node_id) => {
                let info = NodeInfo::from_node_id(node_id);
                a.state.help_text_src.set(info.help());

                use hexotk::AtomDataModel;
                a.ui_params.set(
                    hexotk::AtomId::new(crate::HELP_TABS_ID, 0),
                    hexotk::Atom::Setting(2));

                a.state.show_help();
            },
            ItemType::ClearPorts => {
                a.clear_unused_at((self.x, self.y));
                a.set_focus_at(self.x, self.y);
            },
            ItemType::RandomNode(spec) => {
                if let MenuState::ContextRandomSubMenu { cell } = ms {
                    let ret = cell.find_all_adjacent_free(a.matrix, CellDir::C);
                    if ret.len() > 0 {
                        let mut sm = SplitMix64::new_time_seed();
                        let sel    = ret[sm.next_u64() as usize % ret.len()];
                        a.instanciate_node_at(
                            sel.1,
                            get_rand_node_id(1, RandNodeSelector::OnlyUseful)[0]);
                    }

                } else if let MenuState::ContextRandomPosSubMenu { pos } = ms {
                    match spec {
                        RandSpecifier::One => {
                            let node_id =
                                get_rand_node_id(
                                    1, RandNodeSelector::OnlyUseful)[0];
                            a.instanciate_node_at(pos, node_id);
                        },
                        RandSpecifier::Six => {
                            let node_ids =
                                get_rand_node_id(
                                    6, RandNodeSelector::OnlyUseful);

                            for e in 0..6 {
                                let dir = CellDir::from(e);

                                if let Some(pos) = dir.offs_pos(pos) {
                                    if let Some(cell) =
                                        a.matrix.get(pos.0, pos.1)
                                    {
                                        if cell.is_empty() {
                                            a.instanciate_node_at(
                                                pos, node_ids[e as usize]);
                                        }
                                    }
                                }
                            }
                        },
                    }
                }
            },
            _ => ()
        }
    }
}


struct ActionNewNodeAndConnectionTo {
    dir:            CellDir,
    cell:           Cell,
    x:              usize,
    y:              usize,
    new_node_id:    Option<NodeId>,
    new_io:         Option<usize>,
    use_defaults:   bool,
    followup:       Option<Box<dyn ActionHandler>>,
}

impl ActionNewNodeAndConnectionTo {
    pub fn new(use_defaults: bool, x: usize, y: usize, cell: Cell, dir: CellDir) -> Self {
        Self {
            x, y, cell, dir,
            new_node_id: None,
            new_io:      None,
            followup:    None,
            use_defaults,
        }
    }
}

impl ActionNewNodeAndConnectionTo {
    fn select_old_node_io(&mut self, a: &mut ActionState) {
        let node_id = self.cell.node_id();
        let node_info = NodeInfo::from_node_id(node_id);

        if self.dir.is_input() {
            if node_info.out_count() == 0 {
                self.create_node(None, a);

            } else if node_info.out_count() == 1 || self.use_defaults {
                if self.cell.has_dir_set(self.dir.flip()) {
                    self.create_node(None, a);
                } else {
                    self.create_node(
                        node_info.default_output().map(|i| i as usize),
                        a);
                }

            } else {
                a.next_menu_state(
                    MenuState::SelectOutputParam {
                        node_id,
                        node_info,
                        user_state: 1,
                    },
                    format!("Output of {}", node_id.label()));
            }
        } else {
            if node_info.in_count() == 0 {
                self.create_node(None, a);

            } else if node_info.in_count() == 1 || self.use_defaults {
                if self.cell.has_dir_set(self.dir.flip()) {
                    self.create_node(None, a);
                } else {
                    self.create_node(
                        node_info.default_input().map(|i| i as usize),
                        a);
                }

            } else {
                a.next_menu_state(
                    MenuState::SelectInputParam {
                        node_id,
                        node_info,
                        user_state: 1,
                    },
                    format!("Input of {}", node_id.label()));
            }
        }
    }

    fn select_new_node_io(&mut self, a: &mut ActionState) {
        if let Some(node_id) = self.new_node_id {
            let node_info = NodeInfo::from_node_id(node_id);

            if self.dir.is_input() {
                if node_info.in_count() == 0 {
                    self.select_old_node_io(a);

                } else if node_info.in_count() == 1 || self.use_defaults {
                    self.new_io = node_info.default_input().map(|i| i as usize);
                    self.select_old_node_io(a);

                } else {
                    a.next_menu_state(
                        MenuState::SelectInputParam {
                            node_id,
                            node_info: NodeInfo::from_node_id(node_id),
                            user_state: 0,
                        },
                        format!("Input of {}", node_id.label()));
                }
            } else {
                if node_info.out_count() == 0 {
                    self.select_old_node_io(a);

                } else if node_info.out_count() == 1 || self.use_defaults {
                    self.new_io = node_info.default_output().map(|i| i as usize);
                    self.select_old_node_io(a);

                } else {
                    a.next_menu_state(
                        MenuState::SelectOutputParam {
                            node_id,
                            node_info: NodeInfo::from_node_id(node_id),
                            user_state: 0,
                        },
                        format!("Output of {}", node_id.label()));
                }
            }
        }
    }

    fn create_node(&mut self, old_idx: Option<usize>, a: &mut ActionState) {
        if let Some(new_node_id) = self.new_node_id {
            let ret =
                a.instanciate_node_at_with_connection(
                    (self.x, self.y), new_node_id, false,
                    self.dir, self.new_io, self.cell, old_idx);

            if ret {
                a.set_focus_at(self.x, self.y);
            } else {
                if let (Some(cell_a), cell_b) =
                    (a.matrix.get_copy(self.x, self.y), self.cell)
                {
                    let dir = self.dir;
                    if !cell_a.is_empty() && !self.cell.is_empty() {
                        self.followup =
                            Some(Box::new(
                                ActionSelectCellsIO::new(
                                    false,
                                    cell_a.node_id(), cell_b.node_id(), dir,
                                    Box::new(move |a, io_a, io_b| {
                                        a.set_connection(
                                            dir,
                                            cell_a, io_a,
                                            cell_b, io_b);
                                        a.set_focus_at(
                                            cell_a.pos().0,
                                            cell_a.pos().1);
                                    }))));
                    }
                }
            }
        }
    }
}

impl ActionHandler for ActionNewNodeAndConnectionTo {
    fn init(&mut self, a: &mut ActionState) {
        a.next_menu_state(
            MenuState::SelectCategory { user_state: 0 },
            "Category for new connected node".to_string());
    }

    fn get_followup_action(&mut self, _actions: &mut ActionState)
        -> Option<Box<dyn ActionHandler>>
    {
        self.followup.take()
    }

    fn menu_select(&mut self, a: &mut ActionState, ms: MenuState, item_type: ItemType) {
        match item_type {
            ItemType::Back => { a.menu_back(); },
            ItemType::Category(category) => {
                a.next_menu_state(
                    MenuState::SelectNodeIdFromCat { category, user_state: 0 },
                    "Select new connected node".to_string());

            },
            ItemType::NodeId(node_id) => {
                if let MenuState::SelectNodeIdFromCat { .. } = ms {
                    self.new_node_id = Some(node_id);
                    self.select_new_node_io(a);
                }
            },
            ItemType::OutputIdx(out_idx) => {
                if let MenuState::SelectOutputParam {
                    node_id: _, node_info: _, user_state
                } = ms {
                    if user_state == 1 {
                        self.create_node(Some(out_idx), a);

                    } else {
                        self.new_io = Some(out_idx);
                        self.select_old_node_io(a);
                    }
                }
            },
            ItemType::InputIdx(in_idx) => {
                if let MenuState::SelectInputParam {
                    node_id: _, node_info: _, user_state
                } = ms {
                    if user_state == 1 {
                        self.create_node(Some(in_idx), a);

                    } else {
                        self.new_io = Some(in_idx);
                        self.select_old_node_io(a);
                    }
                }
            },
            _ => ()
        }
    }
}

struct ActionTwoNewConnectedNodes {
    node_id_a:      Option<NodeId>,
    node_id_b:      Option<NodeId>,
    pos_a:          (usize, usize),
    pos_b:          (usize, usize),
    dir:            CellDir,
    connect:        Option<ActionSelectCellsIO>,
    use_defaults:   bool,
}

impl ActionTwoNewConnectedNodes {
    pub fn new(
        use_defaults: bool, pos_a: (usize, usize),
        pos_b: (usize, usize), dir: CellDir
    ) -> Self {
        Self {
            pos_a, pos_b,
            dir,
            use_defaults,
            node_id_a: None,
            node_id_b: None,
            connect: None,
        }
    }
}

impl ActionTwoNewConnectedNodes {
}

impl ActionHandler for ActionTwoNewConnectedNodes {
    fn init(&mut self, a: &mut ActionState) {
        a.next_menu_state(
            MenuState::SelectCategory { user_state: 0 },
            "Category for first new node".to_string());
    }

    fn menu_select(&mut self, a: &mut ActionState, ms: MenuState, item_type: ItemType) {
        match item_type {
            ItemType::Back => { a.menu_back(); },
            ItemType::Category(category) => {
                if let MenuState::SelectCategory { user_state } = ms {
                    a.next_menu_state(
                        MenuState::SelectNodeIdFromCat { category, user_state },
                        if user_state == 0 {
                            "Select first new connected node".to_string()
                        } else {
                            "Select second new connected node".to_string()
                        });
                }
            },
            ItemType::NodeId(node_id) => {
                if let MenuState::SelectNodeIdFromCat { category: _, user_state } = ms {
                    if user_state == 0 {
                        self.node_id_a = Some(node_id);
                        a.next_menu_state(
                            MenuState::SelectCategory { user_state: 1 },
                            "Category for second new node".to_string());

                    } else {
                        self.node_id_b = Some(node_id);
                        if let (Some(node_a), Some(node_b)) =
                            (self.node_id_a, self.node_id_b)
                        {
                            let pos_a = self.pos_a;
                            let pos_b = self.pos_b;
                            let dir   = self.dir;
                            let mut ac =
                                ActionSelectCellsIO::new(
                                    self.use_defaults,
                                    node_a, node_b, self.dir,
                                    Box::new(move |a, io_a, io_b| {
                                        a.instanciate_two_nodes_with_connection(
                                            pos_a, pos_b, node_a, node_b,
                                            io_a, io_b, dir);
                                    }));
                            ac.init(a);
                            self.connect = Some(ac);
                        }
                    }
                }
            },
            ItemType::OutputIdx(_) | ItemType::InputIdx(_) => {
                if let Some(con) = &mut self.connect {
                    con.menu_select(a, ms, item_type);
                }
            },
            _ => ()
        }
    }
}

struct ActionSelectCellsIO {
    dir:            CellDir,
    node_a:         NodeId,
    node_b:         NodeId,
    cell_a_io:      Option<usize>,
    cell_b_io:      Option<usize>,
    use_defaults:   bool,
    finish_cb:      Box<dyn FnMut(&mut ActionState, Option<usize>, Option<usize>)>,
}

impl ActionSelectCellsIO {
    pub fn new(
        use_defaults: bool,
        node_a: NodeId, node_b: NodeId, dir: CellDir,
        finish_cb: Box<dyn FnMut(&mut ActionState, Option<usize>, Option<usize>)>
    ) -> Self {
        Self {
            dir,
            node_a,
            node_b,
            finish_cb,
            use_defaults,
            cell_a_io: None,
            cell_b_io: None,
        }
    }
}

impl ActionSelectCellsIO {
    fn select_cell_b(&mut self, a: &mut ActionState) {
        let node_id   = self.node_b;
        let node_info = NodeInfo::from_node_id(node_id);

        if self.dir.is_input() {
            if node_info.out_count() == 0 {
                self.finish(a);

            } else if node_info.out_count() == 1 || self.use_defaults {
                self.cell_b_io = node_info.default_output().map(|i| i as usize);
                self.finish(a);

            } else {
                a.next_menu_state(
                    MenuState::SelectOutputParam {
                        node_id,
                        node_info,
                        user_state: 1,
                    },
                    format!("Output of {}", node_id.label()));
            }
        } else {
            if node_info.in_count() == 0 {
                self.finish(a);

            } else if node_info.in_count() == 1 || self.use_defaults {
                self.cell_b_io = node_info.default_input().map(|i| i as usize);
                self.finish(a);

            } else {
                a.next_menu_state(
                    MenuState::SelectInputParam {
                        node_id,
                        node_info,
                        user_state: 1,
                    },
                    format!("Input of {}", node_id.label()));
            }
        }
    }

    fn select_cell_a(&mut self, a: &mut ActionState) {
        let node_id   = self.node_a;
        let node_info = NodeInfo::from_node_id(node_id);

        if self.dir.is_input() {
            if node_info.in_count() == 0 {
                self.select_cell_b(a);

            } else if node_info.in_count() == 1 || self.use_defaults {
                self.cell_a_io = node_info.default_input().map(|i| i as usize);
                self.select_cell_b(a);

            } else {
                a.next_menu_state(
                    MenuState::SelectInputParam {
                        node_id,
                        node_info: NodeInfo::from_node_id(node_id),
                        user_state: 0,
                    },
                    format!("Input of {}", node_id.label()));
            }
        } else {
            if node_info.out_count() == 0 {
                self.select_cell_b(a);

            } else if node_info.out_count() == 1 || self.use_defaults {
                self.cell_a_io = node_info.default_output().map(|i| i as usize);
                self.select_cell_b(a);

            } else {
                a.next_menu_state(
                    MenuState::SelectOutputParam {
                        node_id,
                        node_info: NodeInfo::from_node_id(node_id),
                        user_state: 0,
                    },
                    format!("Output of {}", node_id.label()));
            }
        }
    }

    fn finish(&mut self, a: &mut ActionState) {
        (*self.finish_cb)(a, self.cell_a_io, self.cell_b_io);
    }
}

impl ActionHandler for ActionSelectCellsIO {
    fn init(&mut self, a: &mut ActionState) {
        self.select_cell_a(a);
    }

    fn menu_select(&mut self, a: &mut ActionState, ms: MenuState, item_type: ItemType) {
        match item_type {
            ItemType::Back => { a.menu_back(); },
            ItemType::OutputIdx(out_idx) => {
                if let MenuState::SelectOutputParam {
                    node_id: _, node_info: _, user_state
                } = ms {
                    if user_state == 1 {
                        self.cell_b_io = Some(out_idx);
                        self.finish(a);

                    } else {
                        self.cell_a_io = Some(out_idx);
                        self.select_cell_b(a);
                    }
                }
            },
            ItemType::InputIdx(in_idx) => {
                if let MenuState::SelectInputParam {
                    node_id: _, node_info: _, user_state
                } = ms {
                    if user_state == 1 {
                        self.cell_b_io = Some(in_idx);
                        self.finish(a);

                    } else {
                        self.cell_a_io = Some(in_idx);
                        self.select_cell_b(a);
                    }
                }
            },
            _ => ()
        }
    }
}



pub struct DefaultActionHandler {
    ui_action: Option<Box<dyn ActionHandler>>,
}

impl DefaultActionHandler {
    pub fn new() -> Self {
        Self {
            ui_action: None
        }
    }

    pub fn set_action_handler(
        &mut self, mut ah: Box<dyn ActionHandler>,
        a: &mut ActionState
    ) {
        a.clear_menu_history();
        ah.init(a);
        self.ui_action = Some(ah);
    }

    pub fn close_menu(&mut self, a: &mut ActionState) {
        let ms =
            std::mem::replace(
                &mut a.state.menu_state,
                MenuState::None);
        a.state.menu_items = a.state.menu_state.to_items();
    }
}

impl ActionHandler for DefaultActionHandler {
    fn step(&mut self, a: &mut ActionState, msg: &Msg) -> bool {
        if let Some(ah) = self.ui_action.take() {
            a.action_handler = Some(ah);
            let handled = a.exec(msg);

            if let Some(mut ah) = a.action_handler.take() {
                if let Some(ah) = ah.get_followup_action(a) {
                    self.set_action_handler(ah, a);
                } else {
                    self.ui_action = Some(ah);
                }
            }

            if handled {
                return true;
            }
        }

        match msg {
            Msg::CellDragged { btn, pos_a, pos_b, mouse_pos } => {
                a.state.menu_pos = *mouse_pos;

                let (src_cell, dst_cell) = (
                    a.matrix.get_copy(pos_a.0, pos_a.1),
                    a.matrix.get_copy(pos_b.0, pos_b.1)
                );

                // get_copy returns None for cells outside the matrix
                let src_cell =
                    if let Some(src_cell) = src_cell { src_cell }
                    else { return false; };
                let dst_cell =
                    if let Some(dst_cell) = dst_cell { dst_cell }
                    else { return false; };

                let adjacent = CellDir::are_adjacent(*pos_a, *pos_b);

                //d// println!("DRAG CELL! {:?} {:?}", btn, msg);

                let src_is_output =
                    if let Some(dir) = adjacent { dir.is_output() }
                    else { false };

                let src =
                    if src_cell.node_id() == NodeId::Nop { None }
                    else { Some(src_cell) };
                let dst =
                    if dst_cell.node_id() == NodeId::Nop { None }
                    else { Some(dst_cell) };

                //d// println!("ACTDRAG: {:?}",
                //d//     (*btn, src, dst, adjacent, src_is_output));
                match (*btn, src, dst, adjacent, src_is_output) {
                    // Left & pos_a exists & pos_b empty
                    //  => move/swap cell
                    (btn, None, None, Some(dir), _) => {
                        let ah =
                            Box::new(
                                ActionTwoNewConnectedNodes::new(
                                    btn == MButton::Left,
                                    *pos_a, *pos_b, dir));
                        self.set_action_handler(ah, a);
                    },
                    (MButton::Left, Some(_), None, _, _) => {
                        a.move_cluster_from_to(*pos_a, *pos_b);
                        a.set_focus_at(pos_b.0, pos_b.1);
                    },
                    (btn, Some(cell_a), Some(cell_b), None, _) => {
                        let adj_free =
                            cell_b.find_first_adjacent_free(a.matrix, CellDir::T);
                        if let Some((dir, _inp_idx)) = adj_free {
                            if let Some(pos) = dir.offs_pos(cell_b.pos()) {
                                match btn {
                                    MButton::Left =>
                                        a.make_copy_at(
                                            pos, cell_a.node_id()),
                                    MButton::Right | _ =>
                                        a.instanciate_node_at(
                                            pos, cell_a.node_id()),
                                }
                                a.set_focus_at(pos.0, pos.1);
                            }
                        }
                    },
                    (MButton::Left, Some(cell_a), Some(cell_b), Some(dir), _) => {
                        let ah =
                            Box::new(
                                ActionSelectCellsIO::new(
                                    false,
                                    cell_a.node_id(), cell_b.node_id(), dir,
                                    Box::new(move |a, io_a, io_b| {
                                        a.set_connection(
                                            dir,
                                            cell_a, io_a,
                                            cell_b, io_b);
                                        a.set_focus_at(
                                            cell_a.pos().0,
                                            cell_a.pos().1);
                                    })));

                        self.set_action_handler(ah, a);
                    },
                    (MButton::Right, Some(_), None, _, _) => {
                        a.swap_cells(*pos_a, *pos_b);
                        a.set_focus_at(pos_b.0, pos_b.1);
                    },
                    (MButton::Right, Some(_), Some(_), Some(_), _) => {
                        a.split_cluster_at(*pos_b, *pos_a);
                        a.set_focus_at(pos_a.0, pos_a.1);
                    },
                    (btn, None, Some(cell), None, _) => {
                        match btn {
                            MButton::Left => {
                                a.make_copy_at(*pos_a, cell.node_id());
                                a.set_focus_at(pos_a.0, pos_a.1);
                            },
                            MButton::Right => {
                                a.instanciate_node_at(*pos_a, cell.node_id());
                                a.set_focus_at(pos_a.0, pos_a.1);
                            },
                            _ => {},
                        }
                    },
                    (btn, None, Some(cell), Some(dir), _) => {
                        let ah =
                            Box::new(
                                ActionNewNodeAndConnectionTo::new(
                                    btn == MButton::Left,
                                    pos_a.0, pos_a.1, cell, dir));
                        self.set_action_handler(ah, a);
                    },
                    (_, _, _, _, _) => (),
                }
            },
            Msg::Key { key } => {
                match key {
                    Key::F1     => a.toggle_help(),
                    Key::F4     => a.save_patch(),
                    Key::Escape => { a.escape_dialogs(); },
                    _ => {
                        println!("UNHANDLED KEY: {:?}", key);
                    }
                }
            },
            Msg::UIBtn { id } => {
                match *id {
                    ATNID_HELP_BUTTON => a.toggle_help(),
                    ATNID_SAVE_BUTTON => a.save_patch(),
                    _ => (),
                }
            },
            Msg::ClrSelect { clr } => {
                println!("SELECT IN COLOR: {:?}", clr);
                a.set_node_color(a.state.focus_cell.node_id(), *clr);
                self.close_menu(a);
            },
            Msg::MenuHover { item_idx } => {
                if *item_idx < a.state.menu_items.len() {
                    a.state.menu_help_text.set(
                        &a.state.menu_items[*item_idx].help);
                }
            },
            Msg::MenuClick { item_idx } => {
                if *item_idx < a.state.menu_items.len() {
                    let item_type = a.state.menu_items[*item_idx].typ.clone();
                    let ms =
                        std::mem::replace(
                            &mut a.state.menu_state,
                            MenuState::None);

                    if let Some(mut ah) = self.ui_action.take() {
                        let title = a.state.menu_title.borrow().clone();
                        a.push_menu_history(ms.clone(), title);
                        ah.menu_select(a, ms, item_type);

                        if let Some(ah) = ah.get_followup_action(a) {
                            self.set_action_handler(ah, a);
                        } else {
                            self.ui_action = Some(ah);
                        }
                    }

                    a.state.menu_items = a.state.menu_state.to_items();
                }
            },
            Msg::MatrixClick { x, y, btn, modkey: _ } => {
                if let Some(cell) = a.matrix.get_copy(*x, *y) {
                    if cell.is_empty() {
                        if *btn == MButton::Left {
                            let ah = Box::new(ActionNewNodeAtCell::new(*x, *y));
                            self.set_action_handler(ah, a);
                        } else {
                            let ah =
                                Box::new(
                                    ActionContextMenu::new_empty_cell(*x, *y));
                            self.set_action_handler(ah, a);
                        }
                    } else {
                        a.set_focus(cell);

                        if *btn == MButton::Right {
                            let ah =
                                Box::new(ActionContextMenu::new(*x, *y));
                            self.set_action_handler(ah, a);
                        }
                    }
                }
            },
            Msg::MenuMouseClick { x, y, btn: _ } => {
                a.state.next_menu_pos = Some((*x, *y));
            },
            Msg::MatrixMouseClick { x, y, btn: _ } => {
                a.state.menu_pos = (*x, *y);
            },
        }

        false
    }
}

pub fn catch_err_dialog<F>(dialog: Rc<RefCell<DialogModel>>, mut f: F) -> bool
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
                            output1.0.label(),
                            output1.0.instance(),
                            output1.0.out_name_by_idx(output1.1).unwrap_or("???"),
                            output2.0.label(),
                            output2.0.instance(),
                            output2.0.out_name_by_idx(output2.1).unwrap_or("???")),
                        Box::new(|_| ()));
                },
                MatrixError::NonEmptyCell { cell } => {
                    dialog.borrow_mut().open(
                        &format!("Filled cell in the way!\n\
                            You can't move this over an existing cell:\n\
                            \n\
                            Node: {} {} at {},{}",
                            cell.node_id().label(),
                            cell.node_id().instance(),
                            cell.pos().0,
                            cell.pos().1),
                        Box::new(|_| ()));
                }
                MatrixError::PosOutOfRange => {
                    dialog.borrow_mut().open(
                        &format!("Moved out of Range!\n\
                            You can't move things outside of the matrix!"),
                        Box::new(|_| ()));
                }
            }
        },
        Ok(_) => { return true; }
    }

    false
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

