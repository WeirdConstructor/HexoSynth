// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::arg_chk;
use hexodsp::wblockdsp::{BlockFun, BlockLanguage, BlockType, BlockUserInput};
use wlambda::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct VValBlockLanguage {
    lang: Rc<RefCell<BlockLanguage>>,
}

impl VValBlockLanguage {
    pub fn create() -> VVal {
        VVal::new_usr(VValBlockLanguage { lang: Rc::new(RefCell::new(BlockLanguage::new())) })
    }
}

impl VValUserData for VValBlockLanguage {
    fn s(&self) -> String {
        format!("$<BlockDSP::Language>")
    }

    fn call_method(&self, key: &str, env: &mut Env) -> Result<VVal, StackAction> {
        let args = env.argv_ref();

        match key {
            "get_type_list" => {
                arg_chk!(args, 0, "block_code_language.get_type_list[]");
                let ret = VVal::vec();
                for (category, name, input) in self.lang.borrow().get_type_list() {
                    ret.push(VVal::map3(
                        "category",
                        VVal::new_str_mv(category),
                        "name",
                        VVal::new_str_mv(name),
                        "user_input",
                        match input {
                            BlockUserInput::Integer => VVal::new_sym("integer"),
                            BlockUserInput::Float => VVal::new_sym("float"),
                            BlockUserInput::Identifier => VVal::new_sym("identifier"),
                            BlockUserInput::ClientDecision => VVal::new_sym("client"),
                            BlockUserInput::None => VVal::None,
                        },
                    ));
                }
                Ok(ret)
            }
            "define" => {
                arg_chk!(args, 1, "block_code_language.define[block_type]");

                let mut bt = BlockType::default();

                bt.category = env.arg(0).v_s_rawk("category");
                bt.name = env.arg(0).v_s_rawk("name");
                bt.description = env.arg(0).v_s_rawk("description");
                bt.rows = env.arg(0).v_ik("rows") as usize;
                bt.area_count = env.arg(0).v_ik("area_count") as usize;
                bt.color = env.arg(0).v_ik("color") as usize;
                bt.inputs = vec![];
                bt.outputs = vec![];
                env.arg(0).v_k("user_input").with_s_ref(|s| {
                    bt.user_input = match &s[..] {
                        "integer" => BlockUserInput::Integer,
                        "float" => BlockUserInput::Float,
                        "identifier" => BlockUserInput::Identifier,
                        "client" => BlockUserInput::ClientDecision,
                        _ => BlockUserInput::None,
                    }
                });

                env.arg(0).v_k("inputs").with_iter(|it| {
                    for (i, _) in it {
                        if i.is_some() {
                            bt.inputs.push(Some(i.s_raw()));
                        } else {
                            bt.inputs.push(None);
                        }
                    }
                });

                env.arg(0).v_k("outputs").with_iter(|it| {
                    for (i, _) in it {
                        if i.is_some() {
                            bt.outputs.push(Some(i.s_raw()));
                        } else {
                            bt.outputs.push(None);
                        }
                    }
                });

                //d// println!("DEFINE {:?}", bt);

                self.lang.borrow_mut().define(bt);

                Ok(VVal::None)
            }
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> {
        Box::new(self.clone())
    }
}

pub fn vv2block_code_language(mut v: VVal) -> Option<Rc<RefCell<BlockLanguage>>> {
    v.with_usr_ref(|model: &mut VValBlockLanguage| model.lang.clone())
}

#[derive(Clone)]
pub struct VValBlockFun {
    code: Arc<Mutex<BlockFun>>,
}

impl VValBlockFun {
    pub fn from(code: Arc<Mutex<BlockFun>>) -> VVal {
        VVal::new_usr(VValBlockFun { code })
    }
}

impl VValUserData for VValBlockFun {
    fn s(&self) -> String {
        format!("$<BlockDSP::Code>")
    }

    fn call_method(&self, key: &str, env: &mut Env) -> Result<VVal, StackAction> {
        let args = env.argv_ref();

        let mut code = self.code.lock().expect("BlockFun lockable");

        match key {
            "language" => {
                arg_chk!(args, 0, "blockcode.block_language[]");

                let bl = code.block_language();
                Ok(VVal::new_usr(VValBlockLanguage { lang: bl }))
            }
            "instanciate_at" => {
                arg_chk!(args, 4, "blockcode.instanciate_at[area_id, $i(x, y), typ, user_input]");

                let id = env.arg(0).i() as usize;
                let x = env.arg(1).v_i(0);
                let y = env.arg(1).v_i(1);
                let input = env.arg(3);

                let input = if input.is_some() { Some(input.s_raw()) } else { None };

                //d// println!("INSTANCIATE {}", env.arg(1).s());

                env.arg(2).with_s_ref(|typ| {
                    code.instanciate_at(id, x, y, &typ, input);
                });

                Ok(VVal::None)
            }
            "recalculate_area_sizes" => {
                arg_chk!(args, 0, "blockcode.recalculate_area_sizes[]");
                code.recalculate_area_sizes();
                Ok(VVal::None)
            }
            "clone_block_from_to" => {
                arg_chk!(args, 6, "blockcode.clone_block_from_to[id2, x2, y2, id, x, y]");
                code.clone_block_from_to(
                    args[0].i() as usize,
                    args[1].i(),
                    args[2].i(),
                    args[3].i() as usize,
                    args[4].i(),
                    args[5].i());

                Ok(VVal::None)
            }
            "split_block_chain_after" => {
                arg_chk!(args, 4, "blockcode.split_block_chain_after[id, x, y, $n | insert_node_name]");
                let name = args[3].s_raw();
                code.split_block_chain_after(
                    args[0].i() as usize,
                    args[1].i(),
                    args[2].i(),
                    if args[3].is_some() {
                        Some(&name)
                    } else {
                        None
                    });

                Ok(VVal::None)
            }
            "shift_port" => {
                arg_chk!(args, 5, "blockcode.split_block_chain_after[id, x, y, row, is_output]");

                code.shift_port(
                    args[0].i() as usize,
                    args[1].i(),
                    args[2].i(),
                    args[3].i() as usize,
                    args[4].b());

                Ok(VVal::None)
            }
            "move_block_from_to" => {
                arg_chk!(args, 6, "blockcode.move_block_from_to[id2, x2, y2, id, x, y]");
                code.move_block_from_to(
                    args[0].i() as usize,
                    args[1].i(),
                    args[2].i(),
                    args[3].i() as usize,
                    args[4].i(),
                    args[5].i());

                Ok(VVal::None)
            }
            "remove_at" => {
                arg_chk!(args, 3, "blockcode.move_block_from_to[id, x, y]");
                code.remove_at(
                    args[0].i() as usize,
                    args[1].i(),
                    args[2].i());

                Ok(VVal::None)
            }
            "move_block_chain_from_to" => {
                arg_chk!(args, 6, "blockcode.move_block_chain_from_to[id2, x2, y2, id, x, y]");
                code.move_block_chain_from_to(
                    args[0].i() as usize,
                    args[1].i(),
                    args[2].i(),
                    args[3].i() as usize,
                    args[4].i(),
                    args[5].i());

                Ok(VVal::None)
            }
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> {
        Box::new(self.clone())
    }
}

pub fn vv2block_fun(mut v: VVal) -> Option<Arc<Mutex<BlockFun>>> {
    v.with_usr_ref(|model: &mut VValBlockFun| {
        let r: Arc<Mutex<BlockFun>> = model.code.clone();
        r
    })
}

//pub fn vv2block_code_model(mut v: VVal) -> Option<Rc<RefCell<dyn BlockCodeView>>> {
//    v.with_usr_ref(|model: &mut VValBlockFun| {
//        let r: Rc<RefCell<dyn BlockCodeView>> = model.code.clone();
//        r
//    })
//}
