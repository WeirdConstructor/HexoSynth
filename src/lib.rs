// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

#![allow(incomplete_features)]

use hexotk::{open_window, HexoTKWindowHandle, Rect, TestScript, Units, UI};
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

use wlambda::vval::VVal;
use wlambda::*;

mod matrix_param_model;

use raw_window_handle::RawWindowHandle;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

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

    matrix.place(0, 1, Cell::empty(NodeId::Sin(0)).out(Some(0), None, None));
    matrix.place(
        1,
        0,
        Cell::empty(NodeId::Amp(0)).out(Some(0), None, None).input(None, None, Some(0)),
    );
    matrix.place(2, 0, Cell::empty(NodeId::Out(0)).input(None, None, Some(0)));

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
        Self { show_cursor: false }
    }
}

pub struct HexoSynthGUIHandle {
    hexotk_hdl: Option<HexoTKWindowHandle>,
}

impl HexoSynthGUIHandle {
    pub fn close(&mut self) {
        if let Some(hdl) = &mut self.hexotk_hdl {
            hdl.close();
        }
    }
    pub fn is_open(&self) -> bool {
        self.hexotk_hdl.as_ref().map(|h| h.is_open()).unwrap_or(false)
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
    matrix: Arc<Mutex<Matrix>>,
) -> HexoSynthGUIHandle {
    open_hexosynth_with_config(parent, matrix, OpenHexoSynthConfig::default())
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
    (v.v_f(0) as f32, v.v_f(1) as f32, v.v_f(2) as f32)
}

fn set_style_from_key(style: &mut hexotk::Style, key: &str, v: &VVal) -> bool {
    match key {
        "border" => {
            style.border = v.f() as f32;
        }
        "font_size" => {
            style.font_size = v.f() as f32;
        }
        "pad_left" => {
            style.pad_left = v.f() as f32;
        }
        "pad_right" => {
            style.pad_right = v.f() as f32;
        }
        "pad_top" => {
            style.pad_top = v.f() as f32;
        }
        "pad_bottom" => {
            style.pad_bottom = v.f() as f32;
        }
        "shadow_offs" => {
            style.shadow_offs = (v.v_f(0) as f32, v.v_f(1) as f32);
        }
        "color" => {
            style.color = vv2clr(v);
        }
        "bg_color" => {
            style.bg_color = vv2clr(v);
        }
        "border_color" => {
            style.border_color = vv2clr(v);
        }
        "shadow_color" => {
            style.shadow_color = vv2clr(v);
        }
        "hover_shadow_color" => {
            style.hover_shadow_color = vv2clr(v);
        }
        "hover_border_color" => {
            style.hover_border_color = vv2clr(v);
        }
        "hover_color" => {
            style.hover_color = vv2clr(v);
        }
        "active_shadow_color" => {
            style.active_shadow_color = vv2clr(v);
        }
        "active_border_color" => {
            style.active_border_color = vv2clr(v);
        }
        "active_color" => {
            style.active_color = vv2clr(v);
        }
        "text_align" => {
            style.text_align = v.with_s_ref(|vs| match vs {
                "center" => hexotk::Align::Center,
                "left" => hexotk::Align::Left,
                "right" => hexotk::Align::Right,
                _ => hexotk::Align::Left,
            });
        }
        "border_style" => {
            style.border_style = if v.is_vec() {
                let bs = v.v_(0);
                bs.with_s_ref(|bs| match bs {
                    "rect" => hexotk::BorderStyle::Rect,
                    "hex" => hexotk::BorderStyle::Hex { offset: v.v_f(1) as f32 },
                    "bevel" => {
                        let offs = v.v_(1);
                        hexotk::BorderStyle::Bevel {
                            corner_offsets: (
                                offs.v_f(0) as f32,
                                offs.v_f(1) as f32,
                                offs.v_f(2) as f32,
                                offs.v_f(3) as f32,
                            ),
                        }
                    }
                    _ => hexotk::BorderStyle::Rect,
                })
            } else {
                v.with_s_ref(|bs| match bs {
                    "rect" => hexotk::BorderStyle::Rect,
                    "hex" => hexotk::BorderStyle::Hex { offset: 5.0 },
                    "bevel" => hexotk::BorderStyle::Bevel { corner_offsets: (5.0, 5.0, 5.0, 5.0) },
                    _ => hexotk::BorderStyle::Rect,
                })
            };
        }
        "text_valign" => {
            style.text_valign = v.with_s_ref(|vs| match vs {
                "middle" => hexotk::VAlign::Middle,
                "top" => hexotk::VAlign::Top,
                "bottom" => hexotk::VAlign::Bottom,
                _ => hexotk::VAlign::Middle,
            });
        }
        _ => {
            return false;
        }
    }

    true
}

impl VValUserData for VUIStyle {
    fn s(&self) -> String {
        format!("$<UI::Style>")
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> {
        Box::new(self.clone())
    }

    fn call_method(&self, key: &str, env: &mut Env) -> Result<VVal, StackAction> {
        let args = env.argv_ref();

        match key {
            "clone" => {
                arg_chk!(args, 0, "$<UI::TextMut>.clone[]");
                Ok(VVal::new_usr(VUIStyle::from(self.style.borrow().clone())))
            }
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
                                    k.s_raw()
                                )));
                            }
                        }
                    }

                    Ok(VVal::Bol(true))
                });

                if let Ok(v) = &ret {
                    if v.b() {
                        return Ok(VVal::new_usr(VUIStyle::from(Rc::new(new_style))));
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
                                    k.s_raw()
                                )));
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
            }
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
        Self { txtmut: Rc::new(RefCell::new(hexotk::CloneMutable::new(s))) }
    }
}

impl VValUserData for VUITextMut {
    fn s(&self) -> String {
        format!("$<UI::TextMut({})>", **self.txtmut.borrow())
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> {
        Box::new(self.clone())
    }

    fn call_method(&self, key: &str, env: &mut Env) -> Result<VVal, StackAction> {
        let args = env.argv_ref();

        match key {
            "set" => {
                arg_chk!(args, 1, "$<UI::TextMut>.set[string]");

                **self.txtmut.borrow_mut() = env.arg(0).s_raw();

                Ok(env.arg(0))
            }
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
        hexotk::MButton::Left => VVal::new_sym("left"),
        hexotk::MButton::Middle => VVal::new_sym("middle"),
        hexotk::MButton::Right => VVal::new_sym("right"),
    }
}

fn vv2mbutton(vv: &VVal) -> hexotk::MButton {
    vv.with_s_ref(|s| match s {
        "0" | "1" | "left" | "l" | "L" => hexotk::MButton::Left,
        "2" | "right" | "r" | "R" => hexotk::MButton::Right,
        "3" | "middle" | "m" | "M" => hexotk::MButton::Middle,
        _ => hexotk::MButton::Left,
    })
}

fn vv2units(v: &VVal) -> Result<Option<Units>, String> {
    if v.is_none() {
        Ok(None)
    } else if v.is_str() || v.is_sym() {
        v.with_s_ref(|s| match s {
            "auto" => Ok(Some(Units::Auto)),
            _ => Err(format!("Unknown unit: {}", s)),
        })
    } else {
        let unit_type = v.v_(0);
        let value = v.v_(1);
        unit_type.with_s_ref(|unit| match unit {
            "pixels" => Ok(Some(Units::Pixels(value.f() as f32))),
            "percent" => Ok(Some(Units::Percentage(value.f() as f32))),
            "stretch" => Ok(Some(Units::Stretch(value.f() as f32))),
            _ => Err(format!("Unknown unit: {}", unit)),
        })
    }
}

fn vv2rect(v: &VVal) -> Rect {
    Rect { x: v.v_f(0) as f32, y: v.v_f(1) as f32, w: v.v_f(2) as f32, h: v.v_f(3) as f32 }
}

fn rect2vv(r: &Rect) -> VVal {
    VVal::fvec4(r.x as f64, r.y as f64, r.h as f64, r.w as f64)
}

impl VValUserData for VUIWidget {
    fn s(&self) -> String {
        format!("$<UI::Widget({})>", self.0.unique_id())
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> {
        Box::new(self.clone())
    }

    fn call_method(&self, key: &str, env: &mut Env) -> Result<VVal, StackAction> {
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
            "enable_cache" => {
                arg_chk!(args, 0, "$<UI::Widget>.enable_cache[]");
                self.0.enable_cache();
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
            "popup_at_mouse" => {
                arg_chk!(args, 0, "$<UI::Widget>.popup_at_mouse[]");
                self.0.popup_at(hexotk::PopupPos::MousePos);
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
            "set_pos" => {
                arg_chk!(args, 1, "$<UI::Widget>.set_pos[rect]");
                self.0.set_pos(vv2rect(&env.arg(0)));
                Ok(VVal::Bol(true))
            }
            "set_tag" => {
                arg_chk!(args, 1, "$<UI::Widget>.set_tag[tag_string]");
                self.0.set_tag(args[0].s_raw());
                Ok(VVal::Bol(true))
            }
            "remove_childs" => {
                arg_chk!(args, 0, "$<UI::Widget>.remove_childs[]");

                self.0.remove_childs();
                Ok(VVal::Bol(true))
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
            "style" => {
                arg_chk!(args, 0, "$<UI::Widget>.style[]");
                Ok(VVal::new_usr(VUIStyle::from(self.0.style())))
            }
            "set_style" => {
                arg_chk!(args, 1, "$<UI::Widget>.set_style[style]");
                let style = vv2style_rc(env.arg(0));
                if let Some(style) = style {
                    self.0.set_style(style);
                    Ok(VVal::Bol(true))
                } else {
                    wl_panic!("$<UI::Widget>.set_style got no style as argument!")
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
                                            layout.position_type = match &ls[..] {
                                                "self" => Some(hexotk::PositionType::SelfDirected),
                                                "parent" => {
                                                    Some(hexotk::PositionType::ParentDirected)
                                                }
                                                _ => {
                                                    return Err(format!(
                                                        "Unknown position_type assigned: {}",
                                                        ls
                                                    ));
                                                }
                                            };
                                        }
                                    }
                                    "layout_type" => {
                                        if v.is_none() {
                                            layout.layout_type = None;
                                        } else {
                                            let ls = v.s_raw();
                                            layout.layout_type = match &ls[..] {
                                                "row" => Some(hexotk::LayoutType::Row),
                                                "column" => Some(hexotk::LayoutType::Column),
                                                "grid" => Some(hexotk::LayoutType::Grid),
                                                _ => {
                                                    return Err(format!(
                                                        "Unknown layout_type assigned: {}",
                                                        ls
                                                    ));
                                                }
                                            };
                                        }
                                    }
                                    "visible" => {
                                        layout.visible = v.b();
                                    }
                                    "width" => layout.width = vv2units(&v)?,
                                    "height" => layout.height = vv2units(&v)?,
                                    "left" => layout.left = vv2units(&v)?,
                                    "top" => layout.top = vv2units(&v)?,
                                    "right" => layout.right = vv2units(&v)?,
                                    "bottom" => layout.bottom = vv2units(&v)?,
                                    "max_height" => layout.max_height = vv2units(&v)?,
                                    "min_height" => layout.min_height = vv2units(&v)?,
                                    "max_width" => layout.max_width = vv2units(&v)?,
                                    "min_width" => layout.min_width = vv2units(&v)?,
                                    "max_left" => layout.max_left = vv2units(&v)?,
                                    "min_left" => layout.min_left = vv2units(&v)?,
                                    "max_top" => layout.max_top = vv2units(&v)?,
                                    "min_top" => layout.min_top = vv2units(&v)?,
                                    "max_right" => layout.max_right = vv2units(&v)?,
                                    "min_right" => layout.min_right = vv2units(&v)?,
                                    "max_bottom" => layout.max_bottom = vv2units(&v)?,
                                    "min_bottom" => layout.min_bottom = vv2units(&v)?,
                                    "child_left" => layout.child_left = vv2units(&v)?,
                                    "child_top" => layout.child_top = vv2units(&v)?,
                                    "child_right" => layout.child_right = vv2units(&v)?,
                                    "child_bottom" => layout.child_bottom = vv2units(&v)?,
                                    "col_between" => layout.col_between = vv2units(&v)?,
                                    "row_between" => layout.row_between = vv2units(&v)?,
                                    _ => {
                                        return Err(format!(
                                            "Unknown layout field assigned: {}",
                                            ks
                                        ));
                                    }
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
            "set_drag_widget" => {
                arg_chk!(args, 1, "$<UI::Widget>.set_drag_widget[widget]");

                if let Some(wid) = vv2widget(env.arg(0)) {
                    self.0.set_drag_widget(wid);
                    Ok(VVal::Bol(true))
                } else {
                    wl_panic!("$<UI::Widget>.set_drag_widget got no widget as argument!")
                }
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
                        "label" => {
                            self.0.set_ctrl(hexotk::Control::Label {
                                label: Box::new(
                                    vv2txt_mut(env.arg(1)).unwrap_or_else(
                                        || Rc::new(RefCell::new(
                                            hexotk::CloneMutable::new(
                                                String::from("?")))))),
                            });
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
                        "connector" => {
                            if let Some(data) = wlapi::vv2connector_data(env.arg(1)) {
                                self.0.set_ctrl(hexotk::Control::Connector {
                                    con: Box::new(hexotk::Connector::new(data)),
                                });
                                Ok(VVal::Bol(true))

                            } else {
                                Ok(VVal::err_msg(
                                    &format!(
                                        "connector has non connector data as argument: {}",
                                        env.arg(1).s())))
                            }
                        }
                        "wichtext" => {
                            if let Some(data) = wlapi::vv2wichtext_data(env.arg(1)) {
                                self.0.set_ctrl(hexotk::Control::WichText {
                                    wt: Box::new(hexotk::WichText::new(data)),
                                });
                                Ok(VVal::Bol(true))

                            } else {
                                Ok(VVal::err_msg(
                                    &format!(
                                        "wichtext has non wichtext data as argument: {}",
                                        env.arg(1).s())))
                            }
                        }
                        "octave_keys" => {
                            if let Some(data) = wlapi::vv2octave_keys_model(env.arg(1)) {
                                self.0.set_ctrl(hexotk::Control::OctaveKeys {
                                    keys: Box::new(hexotk::OctaveKeys::new(data)),
                                });
                                Ok(VVal::Bol(true))

                            } else {
                                Ok(VVal::err_msg(
                                    &format!(
                                        "octave_keys has non octave_keys_model data as argument: {}",
                                        env.arg(1).s())))
                            }
                        }
                        "graph" => {
                            // Args: sample factor, live/static
                            let samples = env.arg(1).v_i(0) as u16;
                            let live    = env.arg(1).v_b(1);
                            let graph   = env.arg(1).v_(2);
                            if let Some(data) = wlapi::vv2graph_model(graph) {
                                self.0.set_ctrl(hexotk::Control::Graph {
                                    graph: Box::new(hexotk::Graph::new(data, samples, live)),
                                });
                                Ok(VVal::Bol(true))

                            } else {
                                Ok(VVal::err_msg(
                                    &format!(
                                        "graph has non graph_model data as argument: {}",
                                        env.arg(1).s())))
                            }
                        }
                        "graph_minmax" => {
                            // Args: sample factor, live/static
                            let samples = env.arg(1).v_i(0) as usize;
                            let graph   = env.arg(1).v_(1);
                            if let Some(data) = wlapi::vv2graph_minmax_model(graph) {
                                self.0.set_ctrl(hexotk::Control::GraphMinMax {
                                    graph: Box::new(hexotk::GraphMinMax::new(data, samples)),
                                });
                                Ok(VVal::Bol(true))

                            } else {
                                Ok(VVal::err_msg(
                                    &format!(
                                        "graph has non graph_minmax_model data as argument: {}",
                                        env.arg(1).s())))
                            }
                        }
                        "pattern_editor" => {
                            let columns = env.arg(1).v_i(0) as usize;

                            if let Some(model) = wlapi::vv2pat_model(env.arg(1).v_(1)) {
                                if let Some(fb) = wlapi::vv2pat_edit_feedback(env.arg(1).v_(2)) {
                                    let mut edit =
                                        Box::new(hexotk::PatternEditor::new(columns));

                                    edit.set_data_sources(model, fb);
                                    self.0.set_ctrl(
                                        hexotk::Control::PatternEditor { edit });

                                    Ok(VVal::Bol(true))

                                } else {
                                    Ok(VVal::err_msg(
                                        &format!(
                                            "pattern_editor has non $<UI::PatEditFb> as third argument: {}",
                                            env.arg(1).s())))
                                }

                            } else {
                                Ok(VVal::err_msg(
                                    &format!(
                                        "pattern_editor has non $<UI::PatModel> as second argument: {}",
                                        env.arg(1).s())))
                            }
                        }
                        _ => Ok(VVal::err_msg(
                            &format!("Unknown control assigned: {}", typ))),
                    }
                })
            }
            "reg" => {
                arg_chk!(args, 2, "$<UI::Widget>.reg[event_name, callback_fn]");

                let cb = env.arg(1);
                let cb = cb.disable_function_arity();

                self.0.reg(&env.arg(0).s_raw(), {
                    move |ctx, wid, ev| {
                        let mut user_data_out = None;
                        let mut drop_accept = None;

                        if let Some(ctx) = ctx.downcast_mut::<EvalContext>() {
                            //d// println!("WID={:?}", wid);
                            //d// println!("EV={:?}", ev);
                            let arg = match &ev.data {
                                hexotk::EvPayload::Button(btn) => mbutton2vv(*btn),
                                hexotk::EvPayload::HexGridPos { x, y } => {
                                    VVal::map2("x", VVal::Int(*x as i64), "y", VVal::Int(*y as i64))
                                }
                                hexotk::EvPayload::HexGridClick { x, y, button } => VVal::map3(
                                    "x",
                                    VVal::Int(*x as i64),
                                    "y",
                                    VVal::Int(*y as i64),
                                    "button",
                                    mbutton2vv(*button),
                                ),
                                hexotk::EvPayload::HexGridDrag {
                                    x_src,
                                    y_src,
                                    x_dst,
                                    y_dst,
                                    button,
                                } => {
                                    let m = VVal::map2(
                                        "x_src",
                                        VVal::Int(*x_src as i64),
                                        "y_src",
                                        VVal::Int(*y_src as i64),
                                    );
                                    let _ = m.set_key_str("x_dst", VVal::Int(*x_dst as i64));
                                    let _ = m.set_key_str("y_dst", VVal::Int(*y_dst as i64));
                                    let _ = m.set_key_str("button", mbutton2vv(*button));
                                    m
                                }
                                hexotk::EvPayload::WichTextCommand { line, frag, cmd } => {
                                    let m = VVal::map3(
                                        "line",
                                        VVal::Int(*line as i64),
                                        "frag",
                                        VVal::Int(*frag as i64),
                                        "cmd",
                                        VVal::new_str(cmd),
                                    );
                                    m
                                }
                                hexotk::EvPayload::UserData(rc) => {
                                    user_data_out = Some(rc.clone());
                                    VVal::None
                                }
                                hexotk::EvPayload::DropAccept(rc) => {
                                    drop_accept = Some(rc.clone());

                                    if let Some(d) = rc.borrow().0.borrow().downcast_ref::<VVal>() {
                                        d.clone()
                                    } else {
                                        VVal::None
                                    }
                                }
                                hexotk::EvPayload::HexGridDropData { x, y, data: rc } => {
                                    let m = VVal::map2(
                                        "x",
                                        VVal::Int(*x as i64),
                                        "y",
                                        VVal::Int(*y as i64),
                                    );
                                    if let Some(d) = rc.borrow().as_ref().downcast_ref::<VVal>() {
                                        let _ = m.set_key_str("data", d.clone());
                                    }
                                    m
                                }
                                _ => VVal::None,
                            };

                            match ctx.call(&cb, &[VVal::new_usr(VUIWidget::from(wid)), arg]) {
                                Ok(v) => {
                                    if let Some(drop_acc) = drop_accept {
                                        drop_acc.borrow_mut().1 = v.b();
                                    } else if let Some(ud) = user_data_out {
                                        //                                        let data : Box<dyn std::any::Any> = Box::new(10_usize);
                                        *ud.borrow_mut() = Box::new(v);
                                    }
                                }
                                Err(e) => {
                                    println!("ERROR in widget callback: {}", e);
                                }
                            }
                        }
                    }
                });
                Ok(VVal::Bol(true))
            }
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }
}

pub fn vv2widget(mut v: VVal) -> Option<hexotk::Widget> {
    v.with_usr_ref(|w: &mut VUIWidget| w.0.clone())
}

#[derive(Clone)]
pub struct VTestScript(Rc<RefCell<TestScript>>);

impl VTestScript {
    pub fn new(name: String) -> Self {
        Self(Rc::new(RefCell::new(TestScript::new(name))))
    }
}

impl VValUserData for VTestScript {
    fn s(&self) -> String {
        format!("$<UI::TestScript({})>", self.0.borrow().name())
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> {
        Box::new(self.clone())
    }

    fn call_method(&self, key: &str, env: &mut Env) -> Result<VVal, StackAction> {
        let args = env.argv_ref();

        match key {
            "add_step" => {
                arg_chk!(args, 2, "$<UI::TestScript>.add_step[step_name, step_callback]");

                let step_name = args[0].s_raw();
                let step = args[1].disable_function_arity();

                let name = self.0.borrow().name().to_string();

                self.0.borrow_mut().push_cb(step_name.clone(), Rc::new(move |ctx, driver| {
                    let driv_rc = Rc::new(RefCell::new(driver));

                    let ret = {
                        let driver = VTestDriver(driv_rc.clone());
                        let labels = driver.list_labels();

                        let driver = VVal::new_usr(driver);
                        if let Some(ctx) = ctx.downcast_mut::<EvalContext>() {
                            match ctx.call(&step, &[driver, labels]) {
                                Ok(_) => true,
                                Err(e) => {
                                    println!(
                                        "FAIL - {} - step {}: {}", name, step_name, e);
                                    false
                                }
                            }
                        } else {
                            false
                        }
                    };

                    match Rc::try_unwrap(driv_rc) {
                        Ok(cell) => (ret, cell.into_inner()),
                        Err(_) => {
                            panic!("The test scripts MUST NOT take multiple references to the TestDriver!");
                        }
                    }
                }));

                Ok(VVal::Bol(true))
            }
            _ => Ok(VVal::err_msg(&format!("$<UI::TestScript> Unknown method called: {}", key))),
        }
    }
}

pub fn vv2test_script(mut v: VVal) -> Option<TestScript> {
    v.with_usr_ref(|w: &mut VTestScript| w.0.borrow().clone())
}

#[derive(Clone)]
pub struct VTestDriver(Rc<RefCell<Box<hexotk::TestDriver>>>);

impl VTestDriver {
    pub fn list_labels(&self) -> VVal {
        let ret = VVal::vec();
        for entry in self.0.borrow().get_all_labels() {
            let ent = VVal::map2(
                "source",
                VVal::new_str(entry.source),
                "label",
                VVal::new_str_mv(entry.text),
            );
            let _ = ent.set_key_str(
                "logic_pos",
                VVal::ivec2(entry.logic_pos.0 as i64, entry.logic_pos.1 as i64),
            );
            let _ = ent.set_key_str("pos", rect2vv(&entry.pos));
            let _ = ent.set_key_str("wid_pos", rect2vv(&entry.wid_pos));
            let _ = ent.set_key_str("id", VVal::Int(entry.wid_id as i64));
            let _ = ent.set_key_str("tag", VVal::new_str_mv(entry.tag));
            let _ = ent.set_key_str("path", VVal::new_str_mv(entry.tag_path));
            let _ = ent.set_key_str("ctrl", VVal::new_str_mv(entry.ctrl));
            ret.push(ent);
        }

        ret
    }
}

impl VValUserData for VTestDriver {
    fn s(&self) -> String {
        format!("$<UI::TestDriver>")
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> {
        Box::new(self.clone())
    }

    fn call_method(&self, key: &str, env: &mut Env) -> Result<VVal, StackAction> {
        let args = env.argv_ref();

        match key {
            "mouse_press_at" => {
                arg_chk!(args, 2, "$<UI::TestDriver>.mouse_press_at[pos, btn]");

                let btn = vv2mbutton(&args[1]);
                self.0.borrow_mut().inject_mouse_press_at(
                    args[0].v_f(0) as f32,
                    args[0].v_f(1) as f32,
                    btn,
                );
                Ok(VVal::Bol(true))
            }
            "mouse_to" => {
                arg_chk!(args, 1, "$<UI::TestDriver>.mouse_to[pos]");

                self.0.borrow_mut().inject_mouse_to(args[0].v_f(0) as f32, args[0].v_f(1) as f32);
                Ok(VVal::Bol(true))
            }
            "mouse_release_at" => {
                arg_chk!(args, 2, "$<UI::TestDriver>.mouse_release_at[pos, btn]");

                let btn = vv2mbutton(&args[1]);
                self.0.borrow_mut().inject_mouse_release_at(
                    args[0].v_f(0) as f32,
                    args[0].v_f(1) as f32,
                    btn,
                );
                Ok(VVal::Bol(true))
            }
            "list_labels" => {
                arg_chk!(args, 0, "$<UI::TestDriver>.list_labels[]");
                Ok(self.list_labels())
            }
            "key_press" => {
                arg_chk!(args, 1, "$<UI::TestDriver>.key_press[key_name]");

                use std::str::FromStr;

                args[0].with_s_ref(|k| {
                    if let Ok(key) = keyboard_types::Key::from_str(k) {
                        self.0.borrow_mut().inject_key_down(key);
                        Ok(VVal::Bol(true))
                    } else {
                        Ok(VVal::err_msg(&format!("$<UI::TestDriver> Unknown key: {}", k)))
                    }
                })
            }
            "key_release" => {
                arg_chk!(args, 1, "$<UI::TestDriver>.key_release[key_name]");

                use std::str::FromStr;

                args[0].with_s_ref(|k| {
                    if let Ok(key) = keyboard_types::Key::from_str(k) {
                        self.0.borrow_mut().inject_key_up(key);
                        Ok(VVal::Bol(true))
                    } else {
                        Ok(VVal::err_msg(&format!("$<UI::TestDriver> Unknown key: {}", k)))
                    }
                })
            }
            "char" => {
                arg_chk!(args, 1, "$<UI::TestDriver>.char[character]");

                args[0].with_s_ref(|c| self.0.borrow_mut().inject_char(c));
                Ok(VVal::Bol(true))
            }
            _ => Ok(VVal::err_msg(&format!("$<UI::TestDriver> Unknown method called: {}", key))),
        }
    }
}

/// The same as [open_hexosynth] but with more configuration options, see also
/// [OpenHexoSynthConfig].
pub fn open_hexosynth_with_config(
    parent: Option<RawWindowHandle>,
    matrix: Arc<Mutex<Matrix>>,
    _config: OpenHexoSynthConfig,
) -> HexoSynthGUIHandle {
    let hexotk_hdl = open_window(
        "HexoSynth",
        1400,
        800,
        parent,
        Box::new(move || {
            let global_env = GlobalEnv::new_default();

            let lfmr = Rc::new(RefCell::new(wlambda::compiler::LocalFileModuleResolver::new()));

            let env_path =
                std::env::var("HEXOSYNTH_WLAMBDA_PATH").unwrap_or_else(|_| "".to_string());

            if env_path.len() > 0 {
                lfmr.borrow_mut().preload(
                    "main.wl",
                    std::fs::read_to_string(env_path.to_string() + "/main.wl").unwrap().to_string(),
                );
                lfmr.borrow_mut().preload(
                    "wllib/styling.wl",
                    std::fs::read_to_string(env_path.to_string() + "/wllib/styling.wl")
                        .unwrap()
                        .to_string(),
                );
                lfmr.borrow_mut().preload(
                    "wllib/editor.wl",
                    std::fs::read_to_string(env_path.to_string() + "/wllib/editor.wl")
                        .unwrap()
                        .to_string(),
                );
                lfmr.borrow_mut().preload(
                    "wllib/tests.wl",
                    std::fs::read_to_string(env_path.to_string() + "/wllib/tests.wl")
                        .unwrap()
                        .to_string(),
                );
                lfmr.borrow_mut().preload(
                    "wllib/texts.wl",
                    std::fs::read_to_string(env_path.to_string() + "/wllib/texts.wl")
                        .unwrap()
                        .to_string(),
                );
            } else {
                lfmr.borrow_mut()
                    .preload("main.wl", include_str!("wlcode_compiletime/main.wl").to_string());
                lfmr.borrow_mut().preload(
                    "wllib/styling.wl",
                    include_str!("wlcode_compiletime/wllib/styling.wl").to_string(),
                );
                lfmr.borrow_mut().preload(
                    "wllib/editor.wl",
                    include_str!("wlcode_compiletime/wllib/editor.wl").to_string(),
                );
                lfmr.borrow_mut().preload(
                    "wllib/tests.wl",
                    include_str!("wlcode_compiletime/wllib/tests.wl").to_string(),
                );
                lfmr.borrow_mut().preload(
                    "wllib/texts.wl",
                    include_str!("wlcode_compiletime/wllib/texts.wl").to_string(),
                );
            }
            global_env.borrow_mut().set_resolver(lfmr);

            let argv = VVal::vec();
            for e in std::env::args() {
                argv.push(VVal::new_str_mv(e.to_string()));
            }
            global_env.borrow_mut().set_var("ARGV", &argv);

            let mut ctx = wlambda::EvalContext::new(global_env.clone());

            let mut ui_st = wlambda::SymbolTable::new();

            ui_st.fun(
                "style",
                move |_env: &mut Env, _argc: usize| Ok(VVal::new_usr(VUIStyle::new())),
                Some(0),
                Some(0),
                false,
            );

            ui_st.fun(
                "txt",
                move |env: &mut Env, _argc: usize| {
                    Ok(VVal::new_usr(VUITextMut::new(env.arg(0).s_raw())))
                },
                Some(1),
                Some(1),
                false,
            );

            ui_st.fun(
                "connector_data",
                move |_env: &mut Env, _argc: usize| {
                    Ok(VVal::new_usr(wlapi::VValConnectorData::new()))
                },
                Some(0),
                Some(0),
                false,
            );

            ui_st.fun(
                "wichtext_simple_data_store",
                move |_env: &mut Env, _argc: usize| {
                    Ok(VVal::new_usr(wlapi::VValWichTextSimpleDataStore::new()))
                },
                Some(0),
                Some(0),
                false,
            );

            ui_st.fun(
                "create_pattern_feedback_dummy",
                |_env: &mut Env, _argc: usize| Ok(VVal::new_usr(wlapi::VVPatEditFb::new_dummy())),
                Some(0),
                Some(0),
                false,
            );

            ui_st.fun(
                "create_pattern_data_unconnected",
                |env: &mut Env, _argc: usize| {
                    Ok(VVal::new_usr(wlapi::VVPatModel::new_unconnected(env.arg(0).i() as usize)))
                },
                Some(1),
                Some(1),
                false,
            );

            ui_st.fun(
                "widget",
                move |env: &mut Env, _argc: usize| {
                    let style = vv2style_rc(env.arg(0));
                    if let Some(style) = style {
                        Ok(VVal::new_usr(VUIWidget::new(style)))
                    } else {
                        wl_panic!("ui:widget expected $<UI::Style> as first arg!")
                    }
                },
                Some(1),
                Some(1),
                false,
            );

            ui_st.fun(
                "test_script",
                move |env: &mut Env, _argc: usize| {
                    let name = env.arg(0).s_raw();
                    Ok(VVal::new_usr(VTestScript::new(name)))
                },
                Some(1),
                Some(1),
                false,
            );

            let test_scripts: Rc<RefCell<Vec<TestScript>>> = Rc::new(RefCell::new(vec![]));

            let tscr = test_scripts.clone();
            ui_st.fun(
                "install_test",
                move |env: &mut Env, _argc: usize| {
                    if let Some(script) = vv2test_script(env.arg(0)) {
                        tscr.borrow_mut().push(script);
                    } else {
                        wl_panic!("ui:install_test expected $<UI::TestScript> as first arg!")
                    }

                    Ok(VVal::None)
                },
                Some(1),
                Some(1),
                false,
            );

            for (name, clr) in hexotk::style::get_ui_colors() {
                ui_st.set(name, VVal::fvec3(clr.0 as f64, clr.1 as f64, clr.2 as f64));
            }

            let std_clrs = VVal::vec();
            for clr in hexotk::style::get_standard_colors() {
                std_clrs.push(VVal::fvec3(clr.0 as f64, clr.1 as f64, clr.2 as f64));
            }
            ui_st.set("STD_COLORS", std_clrs);

            global_env.borrow_mut().set_module("ui", ui_st);
            global_env.borrow_mut().set_module("hx", wlapi::setup_hx_module(matrix.clone()));
            global_env.borrow_mut().set_module("node_id", wlapi::setup_node_id_module());

            let matrix_obs = Arc::new(wlapi::MatrixRecorder::new());
            matrix.lock().unwrap().set_observer(matrix_obs.clone());

            let mut roots = vec![];

            match ctx.eval_string(
                "!@import main; main:init ARGV; !:global on_frame = main:on_frame; main:root",
                "top_main",
            ) {
                Ok(v) => v.with_iter(|iter| {
                    for (v, _) in iter {
                        if let Some(widget) = vv2widget(v) {
                            roots.push(widget);
                        } else {
                            println!(
                                "ERROR: Expected main.wl to return a list of UI root widgets!"
                            );
                        }
                    }
                }),
                Err(e) => {
                    println!("ERROR: {}", e);
                }
            }

            let frame_cb = ctx.get_global_var("on_frame").unwrap_or(VVal::None);

            let ctx = Rc::new(RefCell::new(ctx));
            let mut ui = Box::new(UI::new(ctx));

            ui.set_frame_callback(Box::new(move |ctx| {
                if frame_cb.is_none() {
                    return;
                }

                matrix.lock().unwrap().update_filters();

                if let Some(ctx) = ctx.downcast_mut::<EvalContext>() {
                    let recs = matrix_obs.get_records();
                    match ctx.call(&frame_cb, &[recs]) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("ERROR in frame callback: {}", e);
                        }
                    }
                }
            }));

            for test_script in test_scripts.borrow().iter() {
                ui.install_test_script(test_script.clone());
            }

            for widget in roots {
                ui.add_layer_root(widget);
            }

            ui
        }),
    );

    HexoSynthGUIHandle { hexotk_hdl }
}
