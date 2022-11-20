// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::arg_chk;

use hexodsp::SAtom;
use wlambda::*;

#[derive(Clone)]
struct VValAtom {
    atom: SAtom,
}

impl VValAtom {
    pub fn new(atom: SAtom) -> Self {
        Self { atom }
    }
}

impl vval::VValUserData for VValAtom {
    fn s(&self) -> String {
        format!(
            "$<HexoDSP::SAtom type={}, i={}, f={:8.4}>",
            self.atom.type_str(),
            self.atom.i(),
            self.atom.f()
        )
    }

    fn call_method(&self, key: &str, env: &mut Env) -> Result<VVal, StackAction> {
        let args = env.argv_ref();

        match key {
            "s" => {
                arg_chk!(args, 0, "atom.s[]");
                Ok(VVal::new_str_mv(self.atom.s()))
            }
            "i" => {
                arg_chk!(args, 0, "atom.i[]");
                Ok(VVal::Int(self.atom.i()))
            }
            "f" => {
                arg_chk!(args, 0, "atom.f[]");
                Ok(VVal::Flt(self.atom.f() as f64))
            }
            "audio_sample_name" => {
                arg_chk!(args, 0, "atom.audio_sample_name[]");

                if let SAtom::AudioSample((name, _vec)) = &self.atom {
                    Ok(VVal::new_str(name))
                } else {
                    Ok(VVal::None)
                }
            }
            "micro_sample" => {
                arg_chk!(args, 0, "atom.micro_sample[]");

                if let SAtom::MicroSample(ms) = &self.atom {
                    let v = VVal::vec();
                    for s in ms.iter() {
                        v.push(VVal::Flt(*s as f64));
                    }

                    Ok(v)
                } else {
                    Ok(VVal::vec1(VVal::Flt(self.atom.f() as f64)))
                }
            }
            "default_of" => {
                arg_chk!(args, 0, "atom.default_of[]");

                Ok(VVal::Usr(Box::new(VValAtom { atom: self.atom.default_of() })))
            }
            "is_continous" => {
                arg_chk!(args, 0, "atom.is_continous[]");

                Ok(VVal::Bol(self.atom.is_continous()))
            }
            "type_str" => {
                arg_chk!(args, 0, "atom.type_str[]");

                Ok(VVal::new_sym(self.atom.type_str()))
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

pub fn vv2atom(mut v: VVal) -> SAtom {
    if let Some(at) = v.with_usr_ref(|model: &mut VValAtom| model.atom.clone()) {
        return at;
    }

    match v {
        VVal::Int(i) => SAtom::setting(i),
        VVal::Flt(f) => SAtom::param(f as f32),
        VVal::Sym(_) | VVal::Str(_) | VVal::Byt(_) => v.with_s_ref(|s| SAtom::str(s)),
        VVal::Pair(_) if v.v_s_raw(0) == "audio_sample" => {
            v.v_with_s_ref(1, |s| SAtom::audio_unloaded(s))
        }
        _ => {
            let mut ms = vec![];
            v.with_iter(|iter| {
                for (v, _) in iter {
                    ms.push(v.f() as f32);
                }
            });
            SAtom::MicroSample(ms)
        }
    }
}

pub fn atom2vv(atom: SAtom) -> VVal {
    VVal::Usr(Box::new(VValAtom::new(atom)))
}
