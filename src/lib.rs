// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

use hexotk::{UI, open_window, Units};
//pub mod ui;
//pub mod ui_ctrl;
mod cluster;
//mod uimsg_queue;
//mod state;
//mod actions;
//mod menu;
//mod dyn_widgets;
pub mod wlapi;

//use ui_ctrl::UICtrlRef;

use wlambda::*;
use wlambda::vval::VVal;

mod matrix_param_model;

use raw_window_handle::RawWindowHandle;

use std::rc::Rc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::io::Write;

//pub use uimsg_queue::Msg;
pub use hexodsp::*;
//pub use hexotk::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initializes the default [Matrix] setup of HexoSynth.
///
/// This routine is used for example by the tests,
/// the VST2 and Jack Standalone versions to get a default
/// and commonly initialized Matrix and DSP executor ([NodeExecutor]).
///
/// It also creates a simple preset so the user won't start out
/// with an empty matrix.
pub fn init_hexosynth() -> (Matrix, NodeExecutor) {
    let (node_conf, node_exec) = nodes::new_node_engine();
    let (w, h) = (16, 16);
    let mut matrix = Matrix::new(node_conf, w, h);

    matrix.place(0, 1, Cell::empty(NodeId::Sin(0))
                       .out(Some(0), None, None));
    matrix.place(1, 0, Cell::empty(NodeId::Amp(0))
                       .out(Some(0), None, None)
                       .input(None, None, Some(0)));
    matrix.place(2, 0, Cell::empty(NodeId::Out(0))
                       .input(None, None, Some(0)));

    let gain_p = NodeId::Amp(0).inp_param("gain").unwrap();
    matrix.set_param(gain_p, gain_p.norm(0.06).into());

    if let Err(e) = load_patch_from_file(&mut matrix, "init.hxy") {
        println!("Error loading init.hxy: {:?}", e);
    }

    let _ = matrix.sync();

    (matrix, node_exec)
}

/// Configuration structure for [open_hexosynth_with_config].
#[derive(Debug, Clone, Default)]
pub struct OpenHexoSynthConfig {
    pub show_cursor: bool,
}

impl OpenHexoSynthConfig {
    pub fn new() -> Self {
        Self {
            show_cursor: false,
        }
    }
}

/// Opens the HexoSynth GUI window and initializes the UI.
///
/// * `parent` - The parent window, only used by the VST.
/// is usually used to drive the UI from the UI tests.
/// And also when out of band events need to be transmitted to
/// HexoSynth or the GUI.
/// * `matrix` - A shared thread safe reference to the
/// [Matrix]. Can be created eg. by [init_hexosynth] or directly
/// constructed.
pub fn open_hexosynth(
    parent: Option<RawWindowHandle>,
    matrix: Arc<Mutex<Matrix>>)
{
    open_hexosynth_with_config(
        parent,
        matrix,
        OpenHexoSynthConfig::default());
}

//#[macro_export]
//macro_rules! arg_chk {
//    ($args: expr, $count: expr, $name: literal) => {
//        if $args.len() != $count {
//            return Err(StackAction::panic_msg(format!(
//                "{} called with wrong number of arguments",
//                $name)));
//        }
//    }
//}
//
//#[macro_export]
//macro_rules! wl_panic {
//    ($str: literal) => {
//        return Err(StackAction::panic_msg($str.to_string()));
//    }
//}
//
#[derive(Clone)]
pub struct VUIStyle {
    pub style: Rc<RefCell<Rc<hexotk::Style>>>,
}

impl VUIStyle {
    pub fn new() -> Self {
        Self { style: Rc::new(RefCell::new(Rc::new(hexotk::Style::new()))) }
    }

    pub fn from(style: Rc<hexotk::Style>) -> Self {
        Self { style: Rc::new(RefCell::new(style)) }
    }
}

//pub struct Style {
//    pub bg_color:               (f32, f32, f32),
//    pub border_color:           (f32, f32, f32),
//    pub color:                  (f32, f32, f32),
//    pub border:                 f32,
//    pub pad_left:               f32,
//    pub pad_right:              f32,
//    pub pad_top:                f32,
//    pub pad_bottom:             f32,
//    pub shadow_offs:            (f32, f32),
//    pub shadow_color:           (f32, f32, f32),
//    pub hover_shadow_color:     (f32, f32, f32),
//    pub hover_border_color:     (f32, f32, f32),
//    pub hover_color:            (f32, f32, f32),
//    pub active_shadow_color:    (f32, f32, f32),
//    pub active_border_color:    (f32, f32, f32),
//    pub active_color:           (f32, f32, f32),
//    pub text_align:             Align,
//    pub text_valign:            VAlign,
//    pub font_size:              f32,
//    pub colors:                 Vec<(f32, f32, f32)>,
//}

fn vv2clr(v: &VVal) -> (f32, f32, f32) {
    (v.v_f(0) as f32,
     v.v_f(1) as f32,
     v.v_f(2) as f32)
}

fn set_style_from_key(style: &mut hexotk::Style, key: &str, v: &VVal) -> bool {
    match key {
        "border"              => { style.border     = v.f() as f32; }
        "font_size"           => { style.font_size  = v.f() as f32; }
        "pad_left"            => { style.pad_left   = v.f() as f32; }
        "pad_right"           => { style.pad_right  = v.f() as f32; }
        "pad_top"             => { style.pad_top    = v.f() as f32; }
        "pad_bottom"          => { style.pad_bottom = v.f() as f32; }
        "shadow_offs" => {
            style.shadow_offs = (v.v_f(0) as f32, v.v_f(1) as f32);
        }
        "color"               => { style.color               = vv2clr(v); }
        "bg_color"            => { style.bg_color            = vv2clr(v); }
        "border_color"        => { style.border_color        = vv2clr(v); }
        "shadow_color"        => { style.shadow_color        = vv2clr(v); }
        "hover_shadow_color"  => { style.hover_shadow_color  = vv2clr(v); }
        "hover_border_color"  => { style.hover_border_color  = vv2clr(v); }
        "hover_color"         => { style.hover_color         = vv2clr(v); }
        "active_shadow_color" => { style.active_shadow_color = vv2clr(v); }
        "active_border_color" => { style.active_border_color = vv2clr(v); }
        "active_color"        => { style.active_color        = vv2clr(v); }
        "text_align" => {
            style.text_align =
                v.with_s_ref(|vs| {
                    match vs {
                        "center" => hexotk::Align::Center,
                        "left"   => hexotk::Align::Left,
                        "right"  => hexotk::Align::Right,
                        _        => hexotk::Align::Left,
                    }
                });
        },
        "border_style" => {
            style.border_style =
                if v.is_vec() {
                    let bs = v.v_(0);
                    bs.with_s_ref(|bs| {
                        match bs {
                            "rect"  => hexotk::BorderStyle::Rect,
                            "hex"   => hexotk::BorderStyle::Hex {
                                offset: v.v_f(1) as f32,
                            },
                            "bevel" => {
                                let offs = v.v_(1);
                                hexotk::BorderStyle::Bevel {
                                    corner_offsets: (
                                        offs.v_f(0) as f32,
                                        offs.v_f(1) as f32,
                                        offs.v_f(2) as f32,
                                        offs.v_f(3) as f32
                                    )
                                }
                            }
                            _ => hexotk::BorderStyle::Rect
                        }
                    })
                } else {
                    v.with_s_ref(|bs| {
                        match bs {
                            "rect"  => hexotk::BorderStyle::Rect,
                            "hex"   => hexotk::BorderStyle::Hex {
                                offset: 5.0
                            },
                            "bevel" => hexotk::BorderStyle::Bevel {
                                corner_offsets: (5.0, 5.0, 5.0, 5.0)
                            },
                            _ => hexotk::BorderStyle::Rect
                        }
                    })
                };
        }
        "text_valign" => {
            style.text_valign =
                v.with_s_ref(|vs| {
                    match vs {
                        "middle" => hexotk::VAlign::Middle,
                        "top"    => hexotk::VAlign::Top,
                        "bottom" => hexotk::VAlign::Bottom,
                        _        => hexotk::VAlign::Middle,
                    }
                });
        },
        _ => { return false; }
    }

    true
}

impl VValUserData for VUIStyle {
    fn s(&self) -> String { format!("$<UI::Style>") }
    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
            "clone" => {
                arg_chk!(args, 0, "$<UI::TextMut>.clone[]");
                Ok(VVal::new_usr(VUIStyle::from(self.style.borrow().clone())))
            },
            "clone_set" => {
                arg_chk!(args, 1, "$<UI::TextMut>.clone_set[map]");
                let mut new_style = (**self.style.borrow()).clone();

                let ret = env.arg(0).with_iter(|iter| {
                    for (v, k) in iter {
                        if let Some(k) = k {
                            let mut bad_key = false;

                            k.with_s_ref(|ks| {
                                if !set_style_from_key(&mut new_style, ks, &v) {
                                    bad_key = true;
                                }
                            });

                            if bad_key {
                                return Ok(VVal::err_msg(&format!(
                                    "$<UI::TextMut>.clone_set called with unknown key: {}",
                                    k.s_raw())));
                            }
                        }
                    }

                    Ok(VVal::Bol(true))
                });

                if let Ok(v) = &ret {
                    if v.b() {
                        return Ok(VVal::new_usr(VUIStyle::from(Rc::new(new_style))))
                    }
                }

                ret
            }
            "set" => {
                arg_chk!(args, 1, "$<UI::TextMut>.set[map]");

                let mut cur_style = (**self.style.borrow()).clone();

                let ret = env.arg(0).with_iter(|iter| {
                    for (v, k) in iter {
                        if let Some(k) = k {
                            let mut bad_key = false;

                            k.with_s_ref(|ks| {
                                if !set_style_from_key(&mut cur_style, ks, &v) {
                                    bad_key = true;
                                }
                            });

                            if bad_key {
                                return Ok(VVal::err_msg(&format!(
                                    "$<UI::TextMut>.set called with unknown key: {}",
                                    k.s_raw())));
                            }
                        }
                    }

                    Ok(VVal::Bol(true))
                });

                if let Ok(v) = &ret {
                    if v.b() {
                        *self.style.borrow_mut() = Rc::new(cur_style);
                    }
                }

                ret
            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }
}

pub fn vv2style_rc(mut v: VVal) -> Option<Rc<hexotk::Style>> {
    v.with_usr_ref(|style: &mut VUIStyle| style.style.borrow().clone())
}

#[derive(Clone)]
pub struct VUITextMut {
    pub txtmut: Rc<RefCell<hexotk::CloneMutable<String>>>,
}

impl VUITextMut {
    pub fn new(s: String) -> Self {
        Self {
            txtmut: Rc::new(RefCell::new(hexotk::CloneMutable::new(s))),
        }
    }
}

impl VValUserData for VUITextMut {
    fn s(&self) -> String { format!("$<UI::TextMut({})>", **self.txtmut.borrow()) }
    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
            "set" => {
                arg_chk!(args, 1, "$<UI::TextMut>.set[string]");

                **self.txtmut.borrow_mut() = env.arg(0).s_raw();

                Ok(env.arg(0))
            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }
}

pub fn vv2txt_mut(mut v: VVal) -> Option<Rc<RefCell<hexotk::CloneMutable<String>>>> {
    v.with_usr_ref(|txtmut: &mut VUITextMut| txtmut.txtmut.clone())
}

#[derive(Clone)]
pub struct VUIWidget(hexotk::Widget);

impl VUIWidget {
    pub fn new(style: Rc<hexotk::Style>) -> Self {
        Self(hexotk::Widget::new(style))
    }

    pub fn from(widget: hexotk::Widget) -> Self {
        Self(widget)
    }
}

fn mbutton2vv(btn: hexotk::MButton) -> VVal {
    match btn {
        hexotk::MButton::Left   => VVal::new_sym("left"),
        hexotk::MButton::Middle => VVal::new_sym("middle"),
        hexotk::MButton::Right  => VVal::new_sym("right"),
    }
}

fn vv2units(v: &VVal) -> Result<Option<Units>, String> {
    if v.is_none() {
        Ok(None)
    } else if v.is_str() || v.is_sym() {
        v.with_s_ref(|s|
            match s {
                "auto" => Ok(Some(Units::Auto)),
                _      => Err(format!("Unknown unit: {}", s)),
            })
    } else {
        let unit_type = v.v_(0);
        let value     = v.v_(1);
        unit_type.with_s_ref(|unit|
            match unit {
                "pixels"  => Ok(Some(Units::Pixels(value.f() as f32))),
                "percent" => Ok(Some(Units::Percentage(value.f() as f32))),
                "stretch" => Ok(Some(Units::Stretch(value.f() as f32))),
                _         => Err(format!("Unknown unit: {}", unit)),
            })
    }
}

impl VValUserData for VUIWidget {
    fn s(&self) -> String { format!("$<UI::Widget({})>", self.0.id()) }
    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
            "show" => {
                arg_chk!(args, 0, "$<UI::Widget>.show[]");
                self.0.show();
                Ok(VVal::Bol(true))
            }
            "hide" => {
                arg_chk!(args, 0, "$<UI::Widget>.show[]");
                self.0.hide();
                Ok(VVal::Bol(true))
            }
            "is_visible" => {
                arg_chk!(args, 0, "$<UI::Widget>.is_visible[]");
                Ok(VVal::Bol(self.0.is_visible()))
            }
            "auto_hide" => {
                arg_chk!(args, 0, "$<UI::Widget>.show[]");
                self.0.auto_hide();
                Ok(VVal::Bol(true))
            }
            "add" => {
                arg_chk!(args, 1, "$<UI::Widget>.add[widget]");

                if let Some(wid) = vv2widget(env.arg(0)) {
                    self.0.add(wid);
                    Ok(VVal::Bol(true))
                } else {
                    wl_panic!("$<UI::Widget>.add got no widget as argument!")
                }
            }
            "remove_child" => {
                arg_chk!(args, 1, "$<UI::Widget>.remove_child[widget]");

                if let Some(wid) = vv2widget(env.arg(0)) {
                    self.0.remove_child(wid);
                    Ok(VVal::Bol(true))
                } else {
                    wl_panic!("$<UI::Widget>.remove_child got no widget as argument!")
                }
            }
            "change_layout" => {
                arg_chk!(args, 1, "$<UI::Widget>.change_layout[layout_set_map]");

                self.0.change_layout(|layout| {
                    env.arg(0).with_iter(|iter| {
                        for (v, k) in iter {
                            let k = k.unwrap_or(VVal::None);

                            let err = k.with_s_ref(|ks| {
                                match ks {
                                    "position_type" => {
                                        if v.is_none() {
                                            layout.position_type = None;

                                        } else {
                                            let ls = v.s_raw();
                                            layout.position_type =
                                                match &ls[..] {
                                                    "self"   => Some(hexotk::PositionType::SelfDirected),
                                                    "parent" => Some(hexotk::PositionType::ParentDirected),
                                                    _ => {
                                                        return Err(format!("Unknown position_type assigned: {}", ls));
                                                    },
                                                };
                                        }
                                    },
                                    "layout_type" => {
                                        if v.is_none() {
                                            layout.layout_type = None;

                                        } else {
                                            let ls = v.s_raw();
                                            layout.layout_type =
                                                match &ls[..] {
                                                    "row"    => Some(hexotk::LayoutType::Row),
                                                    "column" => Some(hexotk::LayoutType::Column),
                                                    "grid"   => Some(hexotk::LayoutType::Grid),
                                                    _ => {
                                                        return Err(format!("Unknown layout_type assigned: {}", ls));
                                                    },
                                                };
                                        }
                                    },
                                    "width"        => { layout.width        = vv2units(&v)? },
                                    "height"       => { layout.height       = vv2units(&v)? },
                                    "left"         => { layout.left         = vv2units(&v)? },
                                    "top"          => { layout.top          = vv2units(&v)? },
                                    "right"        => { layout.right        = vv2units(&v)? },
                                    "bottom"       => { layout.bottom       = vv2units(&v)? },
                                    "max_height"   => { layout.max_height   = vv2units(&v)? },
                                    "min_height"   => { layout.min_height   = vv2units(&v)? },
                                    "max_width"    => { layout.max_width    = vv2units(&v)? },
                                    "min_width"    => { layout.min_width    = vv2units(&v)? },
                                    "max_left"     => { layout.max_left     = vv2units(&v)? },
                                    "min_left"     => { layout.min_left     = vv2units(&v)? },
                                    "max_top"      => { layout.max_top      = vv2units(&v)? },
                                    "min_top"      => { layout.min_top      = vv2units(&v)? },
                                    "max_right"    => { layout.max_right    = vv2units(&v)? },
                                    "min_right"    => { layout.min_right    = vv2units(&v)? },
                                    "max_bottom"   => { layout.max_bottom   = vv2units(&v)? },
                                    "min_bottom"   => { layout.min_bottom   = vv2units(&v)? },
                                    "child_left"   => { layout.child_left   = vv2units(&v)? },
                                    "child_top"    => { layout.child_top    = vv2units(&v)? },
                                    "child_right"  => { layout.child_right  = vv2units(&v)? },
                                    "child_bottom" => { layout.child_bottom = vv2units(&v)? },
                                    "col_between"  => { layout.col_between  = vv2units(&v)? },
                                    "row_between"  => { layout.row_between  = vv2units(&v)? },
                                    _ => {
                                        return Err(format!("Unknown layout field assigned: {}", ks));
                                    },
                                }

                                Ok(())
                            });

                            if let Err(e) = err {
                                return Ok(VVal::err_msg(&e));
                            }
                        }

                        Ok(VVal::Bol(true))
                    })
                })
            }
            "set_ctrl" => {
                arg_chk!(args, 2, "$<UI::Widget>.set_ctrl[ctrl_type_str, init_ctrl_arg]");

                env.arg(0).with_s_ref(|typ| {
                    match typ {
                        // "entry" => {
                        //     self.0.set_ctrl(hexotk::Control::Entry {
                        //         entry: Box::new(hexotk::Entry::new(
                        //             Box::new(
                        //                 vv2txt_mut(env.arg(1))
                        //                 .unwrap_or_else(
                        //                     || Rc::new(RefCell::new(
                        //                         hexotk::CloneMutable::new(
                        //                             String::from("?")))))))),
                        //     });
                        //     Ok(VVal::Bol(true))
                        // }
                        "none" => {
                            self.0.set_ctrl(hexotk::Control::None);
                            Ok(VVal::Bol(true))
                        }
                        "rect" => {
                            self.0.set_ctrl(hexotk::Control::Rect);
                            Ok(VVal::Bol(true))
                        }
                        "button" => {
                            self.0.set_ctrl(hexotk::Control::Button {
                                label: Box::new(
                                    vv2txt_mut(env.arg(1)).unwrap_or_else(
                                        || Rc::new(RefCell::new(
                                            hexotk::CloneMutable::new(
                                                String::from("?")))))),
                            });
                            Ok(VVal::Bol(true))
                        }
                        "knob" => {
                            if let Some(param) = wlapi::vv2hex_knob_model(env.arg(1)) {
                                self.0.set_ctrl(hexotk::Control::HexKnob {
                                    knob: Box::new(hexotk::HexKnob::new(param)),
                                });
                                Ok(VVal::Bol(true))

                            } else {
                                Ok(VVal::err_msg(
                                    &format!(
                                        "knob has non parameter as argument: {}",
                                        env.arg(1).s())))
                            }
                        }
                        "grid" => {
                            if let Some(model) = wlapi::vv2hex_grid_model(env.arg(1)) {
                                self.0.set_ctrl(hexotk::Control::HexGrid {
                                    grid: Box::new(hexotk::HexGrid::new(model)),
                                });
                                Ok(VVal::Bol(true))

                            } else {
                                Ok(VVal::err_msg(
                                    &format!(
                                        "grid has no hex grid model as argument: {}",
                                        env.arg(1).s())))
                            }
                        }
                        _ => Ok(VVal::err_msg(
                            &format!("Unknown control assigned: {}", typ))),
                    }
                })
            },
            "reg" => {
                arg_chk!(args, 2, "$<UI::Widget>.reg[event_name, callback_fn]");

                let cb = env.arg(1);
                let cb = cb.disable_function_arity();

                self.0.reg(&env.arg(0).s_raw(), {
                    move |ctx, wid, ev| {
                        if let Some(ctx) = ctx.downcast_mut::<EvalContext>() {
                            println!("WID={:?}", wid);
                            println!("EV={:?}", ev);
                            let arg =
                                match ev.data {
                                    hexotk::EvPayload::Button(btn) => {
                                        mbutton2vv(btn)
                                    }
                                    hexotk::EvPayload::HexGridClick { x, y, button } => {
                                        VVal::map3(
                                            "x",      VVal::Int(x as i64),
                                            "y",      VVal::Int(y as i64),
                                            "button", mbutton2vv(button))
                                    },
                                    hexotk::EvPayload::HexGridDrag {
                                        x_src, y_src, x_dst, y_dst, button
                                    } => {
                                        let m = VVal::map2(
                                            "x_src", VVal::Int(x_src as i64),
                                            "y_src", VVal::Int(y_src as i64));
                                        m.set_key_str("x_dst", VVal::Int(x_dst as i64));
                                        m.set_key_str("y_dst", VVal::Int(y_dst as i64));
                                        m.set_key_str("button", mbutton2vv(button));
                                        m
                                    },
                                    _ => VVal::None,
                                };

                            match ctx.call(&cb, &[VVal::new_usr(VUIWidget::from(wid)), arg]) {
                                Ok(_) => {},
                                Err(e) => { println!("ERROR in widget callback: {}", e); }
                            }
                        }
                    }
                });
                Ok(VVal::Bol(true))
            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }
}

pub fn vv2widget(mut v: VVal) -> Option<hexotk::Widget> {
    v.with_usr_ref(|w: &mut VUIWidget| w.0.clone())
}

#[derive(Clone)]
pub struct VTestDriver(Rc<RefCell<Box<hexotk::TestDriver>>>);

impl VValUserData for VTestDriver {
    fn s(&self) -> String { format!("$<UI::TestDriver>") }
    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
//            "set" => {
//                arg_chk!(args, 1, "$<UI::TextMut>.set[map]");
//
//                let mut cur_style = (**self.style.borrow()).clone();
//
//                let ret = env.arg(0).with_iter(|iter| {
//                    for (v, k) in iter {
//                        if let Some(k) = k {
//                            let mut bad_key = false;
//
//                            k.with_s_ref(|ks| {
//                                if !set_style_from_key(&mut cur_style, ks, &v) {
//                                    bad_key = true;
//                                }
//                            });
//
//                            if bad_key {
//                                return Ok(VVal::err_msg(&format!(
//                                    "$<UI::TextMut>.set called with unknown key: {}",
//                                    k.s_raw())));
//                            }
//                        }
//                    }
//
//                    Ok(VVal::Bol(true))
//                });
//
//                if let Ok(v) = &ret {
//                    if v.b() {
//                        *self.style.borrow_mut() = Rc::new(cur_style);
//                    }
//                }
//
//                ret
//            },
            _ => Ok(VVal::err_msg(&format!("$<UI::TestDriver> Unknown method called: {}", key))),
        }
    }
}


/// The same as [open_hexosynth] but with more configuration options, see also
/// [OpenHexoSynthConfig].
pub fn open_hexosynth_with_config(
    parent: Option<RawWindowHandle>,
    matrix: Arc<Mutex<Matrix>>,
    config: OpenHexoSynthConfig
) {
    open_window(
        "HexoSynth", 1400, 800,
        parent,
        Box::new(move || {
            let global_env = GlobalEnv::new_default();

            let argv = VVal::vec();
            for e in std::env::args() {
                argv.push(VVal::new_str_mv(e.to_string()));
            }
            global_env.borrow_mut().set_var("ARGV", &argv);

            let mut ctx = wlambda::EvalContext::new(global_env.clone());

            let mut ui_st = wlambda::SymbolTable::new();

            ui_st.fun(
                "style", move |env: &mut Env, _argc: usize| {
                    Ok(VVal::new_usr(VUIStyle::new()))
                }, Some(0), Some(0), false);

            ui_st.fun(
                "txt", move |env: &mut Env, _argc: usize| {
                    Ok(VVal::new_usr(VUITextMut::new(env.arg(0).s_raw())))
                }, Some(1), Some(1), false);

            ui_st.fun(
                "widget", move |env: &mut Env, _argc: usize| {
                    let style = vv2style_rc(env.arg(0));
                    if let Some(style) = style {
                        Ok(VVal::new_usr(VUIWidget::new(style)))
                    } else {
                        wl_panic!("ui:widget expected $<UI::Style> as first arg!")
                    }
                }, Some(1), Some(1), false);


            for (name, clr) in hexotk::style::get_ui_colors() {
                ui_st.set(
                    name,
                    VVal::fvec3(clr.0 as f64, clr.1 as f64, clr.2 as f64));
            }

            let std_clrs = VVal::vec();
            for clr in hexotk::style::get_standard_colors() {
                std_clrs.push(
                    VVal::fvec3(clr.0 as f64, clr.1 as f64, clr.2 as f64));
            }
            ui_st.set("STD_COLORS", std_clrs);

            global_env.borrow_mut().set_module("ui", ui_st);
            global_env.borrow_mut().set_module("hx",      wlapi::setup_hx_module(matrix.clone()));
            global_env.borrow_mut().set_module("node_id", wlapi::setup_node_id_module());

            let mut roots = vec![];

            match ctx.eval_file(
                &std::env::args().nth(1).unwrap_or("main.wl".to_string()))
            {
                Ok(v) => {
                    v.with_iter(|iter| {
                        for (v, _) in iter {
                            if let Some(widget) = vv2widget(v) {
                                roots.push(widget);
                            } else {
                                println!("ERROR: Expected main.wl to return a list of UI root widgets!");
                            }
                        }
                    })
                },
                Err(e) => { println!("ERROR: {}", e); }
            }

            let tests = ctx.get_global_var("loaded_tests");

            let ctx = Rc::new(RefCell::new(ctx));
            let mut ui = Box::new(UI::new(ctx));

            if let Some(tests) = tests {
                tests.with_iter(|scripts| {
                    for (script, _) in scripts {
                        let mut fs = hexotk::FrameScript::new();
                        script.with_iter(|steps| {
                            for (step, _) in steps {
                                let step = step.disable_function_arity();

                                fs.push_cb(Box::new(move |ctx, driver| {
                                    let driv_rc = Rc::new(RefCell::new(driver));
                                    {
                                        let driver = VVal::new_usr(VTestDriver(driv_rc.clone()));
                                        if let Some(ctx) = ctx.downcast_mut::<EvalContext>() {
                                            match ctx.call(&step, &[driver]) {
                                                Ok(_) => {},
                                                Err(e) => { println!("ERROR in frame step callback: {}", e); }
                                            }
                                        }
                                    }

                                    match Rc::try_unwrap(driv_rc) {
                                        Ok(cell) => cell.into_inner(),
                                        Err(_) => {
                                            panic!("The test scripts MUST NOT take multiple references to the TestDriver!");
                                        }
                                    }
                                }));
                            }
                        });
                        ui.push_frame_script(fs);
                    }
                });
            }

            for widget in roots {
                ui.add_layer_root(widget);
            }

            ui
        }));
}
