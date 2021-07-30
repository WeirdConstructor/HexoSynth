use crate::uimsg_queue::{Msg};
use crate::state::{State, ItemType, MenuItem, MenuState, UICategory};
use crate::UIParams;
use crate::dsp::SAtom;

use hexotk::{MButton, AtomId};
use hexotk::widgets::{
    DialogModel,
};
use hexodsp::{Matrix, CellDir, NodeId, Cell, NodeInfo};
use keyboard_types::Key;
use hexodsp::matrix::MatrixError;
use hexodsp::matrix_repr::save_patch_to_file;

use std::rc::Rc;
use std::cell::RefCell;

pub trait ActionHandler {
    fn init(&mut self, actions: &mut ActionState) { }
    fn step(&mut self, actions: &mut ActionState, msg: &Msg) -> bool { false }
    fn menu_select(
        &mut self, actions: &mut ActionState, ms: MenuState,
        item_type: ItemType)
    { }
    fn is_finished(&self) -> bool { false }
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
            diag.borrow_mut().open(
                &format!("Patch saved!\nPatch saved successful to 'init.hxy'!"),
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
                m.place(cell_a.pos().0, cell_a.pos().1, cell_a);
                m.place(cell_b.pos().0, cell_b.pos().1, cell_b);
            })?;

            self.matrix.sync()?;
            Ok(())
        });
    }

    pub fn instanciate_node_at_with_connection(
        &mut self, pos: (usize, usize), node_id: NodeId,
        dir: CellDir, new_idx: Option<usize>, mut adj_cell: Cell, adj_idx: Option<usize>
    ) {
        //d// println!("Instance New {:?}, {:?}, {:?}",
        //d//     node_id, new_idx, adj_idx);

        catch_err_dialog(self.dialog.clone(), || {
            if let Some(mut cell) = self.matrix.get_copy(pos.0, pos.1) {
                let unused_id =
                    self.matrix.get_unused_instance_node_id(node_id);
                cell.set_node_id(unused_id);
                if let Some(new_idx) = new_idx {
                    cell.set_io_dir(dir, new_idx);
                }
                if let Some(adj_idx) = adj_idx {
                    adj_cell.set_io_dir(dir.flip(), adj_idx);
                }

                self.matrix.change_matrix(|m| {
                    m.place(pos.0, pos.1, cell);
                    m.place(adj_cell.pos().0, adj_cell.pos().1, adj_cell);
                })?;
                self.matrix.sync()?;
            }
            Ok(())
        });
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
            MenuState::SelectCategory,
            "Category for new node".to_string());
    }

    fn menu_select(&mut self, a: &mut ActionState, ms: MenuState, item_type: ItemType) {
        match item_type {
            ItemType::Back => { a.menu_back(); },
            ItemType::Category(category) => {
                a.next_menu_state(
                    MenuState::SelectNodeIdFromCat { category },
                    "Select new node".to_string());
            },
            ItemType::NodeId(node_id) => {
                if let MenuState::SelectNodeIdFromCat { category } = ms {
                    a.instanciate_node_at((self.x, self.y), node_id);
                    a.set_focus_at(self.x, self.y);
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
    category:       UICategory,
    new_io:         Option<usize>,
    use_defaults:   bool,
}

impl ActionNewNodeAndConnectionTo {
    pub fn new(use_defaults: bool, x: usize, y: usize, cell: Cell, dir: CellDir) -> Self {
        Self {
            x, y, cell, dir,
            new_node_id: None,
            category:    UICategory::None,
            new_io:      None,
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
                    self.create_node(Some(0), a);
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
                    self.create_node(Some(0), a);
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
                    self.new_io = Some(0);
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
                    self.new_io = Some(0);
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
            // XXX: Reset the node instance shenanigans from below with the 999999
            let new_node_id = new_node_id.to_instance(0);

            a.instanciate_node_at_with_connection(
                (self.x, self.y), new_node_id,
                self.dir, self.new_io, self.cell, old_idx);
            a.set_focus_at(self.x, self.y);
        }
    }
}

impl ActionHandler for ActionNewNodeAndConnectionTo {
    fn init(&mut self, a: &mut ActionState) {
        a.next_menu_state(
            MenuState::SelectCategory,
            "Category for new connected node".to_string());
    }

    fn menu_select(&mut self, a: &mut ActionState, ms: MenuState, item_type: ItemType) {
        match item_type {
            ItemType::Back => { a.menu_back(); },
            ItemType::Category(category) => {
                self.category = category;
                a.next_menu_state(
                    MenuState::SelectNodeIdFromCat { category },
                    "Select new connected node".to_string());

            },
            ItemType::NodeId(node_id) => {
                if let MenuState::SelectNodeIdFromCat { category } = ms {
                    // TODO: To determine which is the right one, we need to
                    //       look at the self.dir CellDir!
                    //       First we need to select the output node_id!
                    // XXX: We use instance 999999 here, so it does not
                    //      match any possibly existing node_id, in case we assign
                    //      the same kind of node (eg. Amp(0) => Amp(...)).
                    self.new_node_id = Some(node_id.to_instance(999999));
                    self.select_new_node_io(a);
                }
            },
            ItemType::OutputIdx(out_idx) => {
                if let MenuState::SelectOutputParam {
                    node_id, node_info, user_state
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
                    node_id, node_info, user_state
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

struct ActionConnectCells {
    dir:            CellDir,
    cell_a:         Cell,
    cell_b:         Cell,
    cell_a_io:      Option<usize>,
    cell_b_io:      Option<usize>,
}

impl ActionConnectCells {
    pub fn new(cell_a: Cell, cell_b: Cell, dir: CellDir) -> Self {
        Self {
            dir,
            cell_a,
            cell_b,
            cell_a_io: None,
            cell_b_io: None,
        }
    }
}

impl ActionConnectCells {
    fn select_cell_b(&mut self, a: &mut ActionState) {
        let node_id   = self.cell_b.node_id();
        let node_info = NodeInfo::from_node_id(node_id);

        if self.dir.is_input() {
            if node_info.out_count() == 0 {
                self.create_connection(a);

            } else if node_info.out_count() == 1 {
                self.cell_b_io = Some(0);
                self.create_connection(a);

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
                self.create_connection(a);

            } else if node_info.in_count() == 1 {
                self.cell_b_io = Some(0);
                self.create_connection(a);

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
        let node_id   = self.cell_a.node_id();
        let node_info = NodeInfo::from_node_id(node_id);

        if self.dir.is_input() {
            if node_info.in_count() == 0 {
                self.select_cell_b(a);

            } else if node_info.in_count() == 1 {
                self.cell_a_io = Some(0);
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

            } else if node_info.out_count() == 1 {
                self.cell_a_io = Some(0);
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

    fn create_connection(&mut self, a: &mut ActionState) {
        a.set_connection(
            self.dir,
            self.cell_a, self.cell_a_io,
            self.cell_b, self.cell_b_io);
        a.set_focus_at(self.cell_a.pos().0, self.cell_a.pos().1);
    }
}

impl ActionHandler for ActionConnectCells {
    fn init(&mut self, a: &mut ActionState) {
        self.select_cell_a(a);
    }

    fn menu_select(&mut self, a: &mut ActionState, ms: MenuState, item_type: ItemType) {
        match item_type {
            ItemType::Back => { a.menu_back(); },
            ItemType::OutputIdx(out_idx) => {
                if let MenuState::SelectOutputParam {
                    node_id, node_info, user_state
                } = ms {
                    if user_state == 1 {
                        self.cell_b_io = Some(out_idx);
                        self.create_connection(a);

                    } else {
                        self.cell_a_io = Some(out_idx);
                        self.select_cell_b(a);
                    }
                }
            },
            ItemType::InputIdx(in_idx) => {
                if let MenuState::SelectInputParam {
                    node_id, node_info, user_state
                } = ms {
                    if user_state == 1 {
                        self.cell_b_io = Some(in_idx);
                        self.create_connection(a);

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
}

impl ActionHandler for DefaultActionHandler {
    fn step(&mut self, a: &mut ActionState, msg: &Msg) -> bool {
        if let Some(ah) = self.ui_action.take() {
            a.action_handler = Some(ah);
            let handled = a.exec(msg);
            self.ui_action = a.action_handler.take();

            if handled {
                return true;
            }
        }

        match msg {
            Msg::CellDragged { btn, pos_a, pos_b, mouse_pos } => {
                a.state.menu_pos = *mouse_pos;

                // DONE: Left & pos_a empty & pos_b exists
                //  => open cell selection dialog for one node
                //  => connect the default input
                // Left & pos_a empty & pos_b empty & adjacent
                //  => open cell selection dialog for two NodeIds
                //  => connect the default output to default input
                //     default is always: first input, first output
                // Left & pos_a exists & pos_b exists & adjacent
                //  => open connection selection dialog for the port of the
                //     pos_a cell!
                //     (Note: you can reassign both with the right mouse button)
                // Left & pos_a exists & pos_b exists & NOT adjacent
                //  => take pos_a as output, pos_b as input
                //  => search an empty input for pos_b
                //  => copy cell at pos_a to that empty cell
                //  => open connection dialog for out => inp

                // Right & pos_a exists & pos_b empty & NOT adjacent
                //  => copy cell, but with empty ports
                // Right & pos_a exists & pos_b empty & adjacent
                //  => copy cell, but with empty ports, open port connection dialog
                // Right & pos_a exists & pos_b exists & adjacent
                //  => open connection menu for both
                // Right & pos_a empty & pos_b empty & adjacent
                //  => ????
                // Right & pos_a empty & pos_b exists
                //  => Delete pos_b
                // Right & pos_a exists & pos_b exists & NOT adjacent
                //  => take pos_a as output, pos_b as input
                //  => search an empty input for pos_b
                //  => NEW INSTANCE NodeId at pos_a to that empty cell
                //  => open connection dialog for out => inp

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

                match (*btn, src, dst, adjacent, src_is_output) {
                    // Left & pos_a exists & pos_b empty
                    //  => move/swap cell
                    (MButton::Left, Some(_), None, _, _) => {
                        a.swap_cells(*pos_a, *pos_b);
                    },
                    (MButton::Left, Some(cell_a), Some(cell_b), Some(dir), _) => {
                        let mut ah =
                            Box::new(
                                ActionConnectCells::new(cell_a, cell_b, dir));
                        self.set_action_handler(ah, a);
                    },
                    (btn, None, Some(cell), Some(dir), _) => {
                        let mut ah =
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
                    Key::F1 => a.toggle_help(),
                    Key::F4 => { a.save_patch(); },
                    Key::Escape => { a.escape_dialogs(); },
                    _ => {
                        println!("UNHANDLED KEY: {:?}", key);
                    }
                }
            },
            Msg::UIBtn { id } => {
                match *id {
                    ATNID_HELP_BUTTON => a.toggle_help(),
                    _ => {}
                }
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
//                        if let Some(new_menu_pos) =
//                            a.state.next_menu_pos.take()
//                        {
//                            a.state.menu_pos = new_menu_pos;
//                        }

                        let title = a.state.menu_title.borrow().clone();
                        a.push_menu_history(ms.clone(), title);
                        ah.menu_select(a, ms, item_type);
                        self.ui_action = Some(ah);
                    }

                    a.state.menu_items = a.state.menu_state.to_items();
                }
            },
            Msg::MatrixClick { x, y, btn, modkey } => {
                if let Some(cell) = a.matrix.get_copy(*x, *y) {
                    if cell.is_empty() {
                        if *btn == MButton::Left {
                            let mut ah = Box::new(ActionNewNodeAtCell::new(*x, *y));
                            self.set_action_handler(ah, a);
                        }
                    } else {
                        a.set_focus(cell);
                    }
                }
            },
            Msg::MenuMouseClick { x, y, btn } => {
                if *btn == MButton::Left {
                    a.state.next_menu_pos = Some((*x, *y));
                }
            },
            Msg::MatrixMouseClick { x, y, btn } => {
                if *btn == MButton::Left {
                    a.state.menu_pos = (*x, *y);
                }
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

