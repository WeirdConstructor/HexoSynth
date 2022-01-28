// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use wlambda::*;
use crate::arg_chk;
use crate::ui::{
    BlockCodeLanguage,
    BlockDSPCode,
    BlockType,
    BlockCodeModel,
};

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub struct VValBlockCodeLanguage {
    lang: Rc<RefCell<BlockCodeLanguage>>,
}

impl VValBlockCodeLanguage {
    pub fn create() -> VVal {
        VVal::new_usr(VValBlockCodeLanguage {
            lang: Rc::new(RefCell::new(BlockCodeLanguage::new()))
        })
    }
}

impl VValUserData for VValBlockCodeLanguage {
    fn s(&self) -> String { format!("$<BlockDSP::Language>") }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
            "get_type_list" => {
                arg_chk!(args, 0, "block_code_language.get_type_list[]");
                Ok(VVal::None)
            },
            "define" => {
                arg_chk!(args, 1, "block_code_language.define[block_type]");

                let mut bt = BlockType::default();

                bt.category    = env.arg(0).v_s_rawk("category");
                bt.name        = env.arg(0).v_s_rawk("name");
                bt.description = env.arg(0).v_s_rawk("description");
                bt.rows        = env.arg(0).v_ik("rows") as usize;
                bt.area_count  = env.arg(0).v_ik("area_count") as usize;
                bt.user_input  = env.arg(0).v_bk("user_input");
                bt.inputs  = vec![];
                bt.outputs = vec![];
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

                self.lang.borrow_mut().define(bt);

                Ok(VVal::None)
            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

pub fn vv2block_code_language(mut v: VVal) -> Option<Rc<RefCell<BlockCodeLanguage>>> {
    v.with_usr_ref(|model: &mut VValBlockCodeLanguage| model.lang.clone())
}

#[derive(Clone)]
pub struct VValBlockDSPCode {
    code:   Rc<RefCell<BlockDSPCode>>,
}

impl VValBlockDSPCode {
    pub fn create(lang: VVal) -> VVal {

        if let Some(lang) = vv2block_code_language(lang.clone()) {
            VVal::new_usr(VValBlockDSPCode {
                code: Rc::new(RefCell::new(BlockDSPCode::new(lang)))
            })

        } else {
            VVal::err_msg(&format!("Not a $<BlockDSP:Language>: {}", lang.s()))
        }
    }
}

impl VValUserData for VValBlockDSPCode {
    fn s(&self) -> String { format!("$<BlockDSP::Code>") }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
            "instanciate_at" => {
                arg_chk!(args, 4, "block_code.instanciate_at[area_id, $i(x, y), typ, user_input]");

                let id    = env.arg(0).i() as usize;
                let x     = env.arg(1).v_i(0) as usize;
                let y     = env.arg(1).v_i(1) as usize;
                let input = env.arg(3);

                let input =
                    if input.is_some() { Some(input.s_raw()) }
                    else { None };

                println!("INSTANCIATE {}", env.arg(1).s());

                env.arg(2).with_s_ref(|typ|
                    self.code
                        .borrow_mut()
                        .instanciate_at(id, x, y, &typ, input)
                        .unwrap());

                Ok(VVal::None)
            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

pub fn vv2block_code_model(mut v: VVal) -> Option<Rc<RefCell<dyn BlockCodeModel>>> {
    v.with_usr_ref(|model: &mut VValBlockDSPCode| {
        let r : Rc<RefCell<dyn BlockCodeModel>> = model.code.clone();
        r
    })
}
