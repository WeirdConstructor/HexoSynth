// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use wlambda::*;
#[allow(warnings)]
use tuix::*;
#[allow(warnings)]
use tuix::widgets::*;

#[allow(dead_code)]
mod ui;
#[allow(dead_code)]
mod cluster;
mod matrix_param_model;
mod wlapi;

mod jack;
mod synth;

use ui::*;

use wlapi::{
    atom2vv, vv2atom,
    vv2hex_knob_model, vv2hex_grid_model,
    matrix2vv,
    VValCluster,
    VValCellDir,
    cell_dir2vv,
    new_test_grid_model,
    MatrixRecorder,
};

use hexodsp::{Matrix, CellDir};

use std::rc::Rc;
use std::cell::RefCell;

use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum GUIAction {
    NewElem(i64, i64, VVal),
    NewLabel(i64, i64, String, VVal),
    NewRow(i64, i64, Option<String>),
    NewCol(i64, i64, VVal),
    NewHexKnob(i64, i64, VVal, VVal),
    NewHexGrid(i64, i64, VVal),
    NewTabs(Vec<(VVal, i64)>, i64, VVal),
    NewPopup(i64, VVal),
    NewPatternEditor(i64, i64, Option<String>),
    NewButton(i64, i64, String, VVal, VVal),
    NewOctaveKeys(i64, i64, VVal),
    NewCvArray(i64, i64, VVal),
    NewConnector(i64, i64, VVal),
    NewBlockCode(i64, i64, VVal),
    EmitTo(i64, i64, VVal),
    SetProp(i64, &'static str, VVal),
    SetText(i64, String),
    AddTheme(String),
    RemoveAllChilds(i64),
    Remove(i64),
    Redraw,
}

#[derive(Debug)]
enum GUIRef {
    None,
    Ent(Entity),
}

pub struct GUIActionRecorder {
    actions:    Vec<GUIAction>,
    refs:       Vec<GUIRef>,
    free_refs:  Vec<i64>,
    ref_idx:    i64,
}


pub fn exec_cb(
    self_ref: Rc<RefCell<GUIActionRecorder>>,
    wl_ctx:   Rc<RefCell<EvalContext>>,
    state:    &mut State,
    entity:   Entity,
    callback: VVal,
    args:     &[VVal])
{
    if callback.is_none() {
        return;
    }

    match wl_ctx.borrow_mut().call(&callback, args) {
        Ok(_) => {},
        Err(e) => { panic!("Error in callback: {:?}", e); }
    }

    let sr = self_ref.clone();

    self_ref.borrow_mut().run(sr, wl_ctx, state, entity);
}

fn vv2event(event: &VVal) -> Event {
    match &event.v_s_raw(0)[..] {
        "textbox:set_value"
            => Event::new(TextboxEvent::SetValue(event.v_s_raw(1))),
        "popup:open_at_cursor"
            => Event::new(PopupEvent::OpenAtCursor),
        "popup:close"
            => Event::new(PopupEvent::Close),
        "connector:set_connection" => {
            Event::new(
                ConMessage::SetConnection(
                    if event.v_(1).is_none() {
                        None
                    } else {
                        Some((
                            event.v_i(1) as usize,
                            event.v_i(2) as usize))
                    }))
        },
        "connector:set_items" => {
            let mut vin  = vec![];
            let mut vout = vec![];

            event.v_(1).with_iter(|it| {
                for (inp, _) in it {
                    vin.push((inp.v_s_raw(0), inp.v_(1).b()));
                }
            });

            event.v_(2).with_iter(|it| {
                for (out, _) in it {
                    vout.push((out.v_s_raw(0), out.v_(1).b()));
                }
            });

            Event::new(ConMessage::SetItems(Box::new((vin, vout))))
        },
        "octave_keys:set_mask"
            => Event::new(
                OctaveKeysMessage::SetMask(event.v_i(1))),
        "cv_array:set_array" => {
            if let Some(ar) = vv2sample_buf(event.v_(1)) {
                Event::new(CvArrayMessage::SetArray(ar.clone()))
            } else {
                eprintln!("Bad Event Type sent: {}, bad array arg!", event.s());
                Event::new(WindowEvent::Redraw)
            }
        },
        "hexknob:set_model" => {
            if let Some(model) = vv2hex_knob_model(event.v_(1)) {
                Event::new(HexKnobMessage::SetModel(model))
            } else {
                eprintln!("Bad Event Type sent: {}, bad model arg!", event.s());
                Event::new(WindowEvent::Redraw)
            }
        },
        "hexgrid:set_model" => {
            if let Some(model) = vv2hex_grid_model(event.v_(1)) {
                Event::new(HexGridMessage::SetModel(model))
            } else {
                eprintln!("Bad Event Type sent: {}, bad model arg!", event.s());
                Event::new(WindowEvent::Redraw)
            }
        },
        _ => {
            eprintln!("Unknown Event Type sent: {}", event.s());
            Event::new(WindowEvent::Redraw)
        },
    }
}

fn vv2class(class: VVal) -> Option<String> {
    if class.is_some() {
        Some(class.s_raw())
    } else {
        None
    }
}

fn vv2units(v: &VVal) -> Units {
    if v.is_int() {
        return Units::Pixels(v.i() as f32);
    }

    let amt = v.v_f(0) as f32;
    v.v_with_s_ref(1, |s|
        match s {
            "px"       => Units::Pixels(amt),
            "%"        => Units::Percentage(amt),
            "s"        => Units::Stretch(amt),
            "auto" | _ => Units::Auto,
        })
}

fn set_vv_prop(state: &mut State, ent: Entity, prop: &str, v: VVal) {
    match &prop[..] {
        "height" => { ent.set_height(state, vv2units(&v)); },
        _ => {},
    }
}

fn vvbuilder<'a, T>(mut builder: Builder<'a, T>, build_attribs: &VVal)
    -> Builder<'a, T>
{
    let mut attribs = vec![];

    println!("VVB: {}", build_attribs.s());

    build_attribs.for_eachk(|key, val| {
        attribs.push((key.to_string(), val.clone()));
    });


    for (k, v) in attribs {
        let ent = builder.entity();
        set_vv_prop(builder.state(), ent, &k[..], v.clone());

        builder =
            match &k[..] {
                "class" => {
                    if v.is_vec() {
                        for i in 0..v.len() {
                            builder = builder.class(&v.v_s_raw(i));
                        }
                        builder
                    } else {
                        builder.class(&v.s_raw())
                    }
                },
                "space"       => { builder.set_space(vv2units(&v)) },
                "child_space" => { builder.set_child_space(vv2units(&v)) },
                "text"        => { builder.set_text(&v.s_raw()) },
                "row"         => { builder.set_row_index(v.i() as usize) },
                "col"         => { builder.set_col_index(v.i() as usize) },
                "row_span"    => { builder.set_row_span(v.i() as usize) },
                "col_span"    => { builder.set_col_span(v.i() as usize) },
                "row_between" => { builder.set_row_between(vv2units(&v)) },
                "col_between" => { builder.set_col_between(vv2units(&v)) },
                "z_order"     => { builder.set_z_order(v.i() as i32) },
                "width"       => { builder.set_width(vv2units(&v)) },
                "height"      => { builder.set_height(vv2units(&v)) },
                "left"        => { builder.set_left(vv2units(&v)) },
                "top"         => { builder.set_top(vv2units(&v)) },
                "right"       => { builder.set_right(vv2units(&v)) },
                "bottom"      => { builder.set_bottom(vv2units(&v)) },
                "grid_rows" => {
                    let mut rows = vec![];
                    v.for_each(|v| { rows.push(vv2units(v)); });
                    builder.set_grid_rows(rows)
                },
                "grid_cols" => {
                    let mut cols = vec![];
                    v.for_each(|v| { cols.push(vv2units(v)); });
                    builder.set_grid_cols(cols)
                },
                "layout_type" => {
                    builder.set_layout_type(
                        if &v.s_raw() == "row" {
                            LayoutType::Row
                        } else if &v.s_raw() == "grid" {
                            LayoutType::Grid
                        } else if &v.s_raw() == "column" {
                            LayoutType::Column
                        } else {
                            LayoutType::Column
                        })
                },
                "position" =>
                    builder.set_position_type(
                        if &v.s_raw() == "self" { PositionType::SelfDirected }
                        else { PositionType::ParentDirected }),
                _       => builder,
            };
    }

    builder
}

#[derive(Clone)]
struct VValSampleBuf {
    buf: Arc<Mutex<Vec<f32>>>,
}

impl VValSampleBuf {
    pub fn from_vec(v: Vec<f32>) -> Self {
        Self {
            buf: Arc::new(Mutex::new(v)),
        }
    }
}

impl vval::VValUserData for VValSampleBuf {
    fn s(&self) -> String {
        let size = self.buf.lock().map_or(0, |guard| guard.len());
        format!("$<SampleBuf[{}]>", size)
    }

    fn set_key(&self, key: &VVal, val: VVal) -> Result<(), StackAction> {
        let idx = key.i() as usize;

        if let Ok(mut guard) = self.buf.lock() {
            if idx < guard.len() {
                guard[idx] = val.f() as f32;
            }
        }

        Ok(())
    }

    fn get_key(&self, key: &str) -> Option<VVal> {
        let idx = key.parse::<usize>().unwrap_or(0);
        let val =
            self.buf.lock().map_or(
                None,
                |guard| guard.get(idx).copied())?;

        Some(VVal::Flt(val as f64))
    }

    fn call_method(&self, key: &str, env: &mut Env)
        -> Result<VVal, StackAction>
    {
        let args = env.argv_ref();

        match key {
            "len" => {
                arg_chk!(args, 0, "sample_buf.len[]");

                let size = self.buf.lock().map_or(0, |guard| guard.len());
                Ok(VVal::Int(size as i64))
            },
            _ => Ok(VVal::err_msg(&format!("Unknown method called: {}", key))),
        }
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_ud(&self) -> Box<dyn vval::VValUserData> { Box::new(self.clone()) }
}

fn vv2sample_buf(mut v: VVal) -> Option<Arc<Mutex<Vec<f32>>>> {
    v.with_usr_ref(|model: &mut VValSampleBuf| model.buf.clone())
}

fn sample_buf2vv(r: Arc<Mutex<Vec<f32>>>) -> VVal {
    VVal::new_usr(VValSampleBuf { buf: r })
}

fn btn2vval(btn: tuix::MouseButton) -> VVal {
    match btn {
        tuix::MouseButton::Right    => VVal::new_sym("right"),
        tuix::MouseButton::Middle   => VVal::new_sym("middle"),
        tuix::MouseButton::Left     => VVal::new_sym("left"),
        tuix::MouseButton::Other(n) =>
            VVal::pair(VVal::new_sym("other"), VVal::Int(n as i64)),
    }
}

impl GUIActionRecorder {
    pub fn new() -> Rc<RefCell<GUIActionRecorder>> {
        Rc::new(RefCell::new(
            GUIActionRecorder {
                actions:    vec![],
                refs:       vec![],
                free_refs:  vec![],
                ref_idx:    1,
            }))
    }

    pub fn set(&mut self, id: i64, prop: &'static str, v: VVal) {
        self.actions.push(GUIAction::SetProp(id, prop, v));
    }

    pub fn add<F: FnOnce(i64) -> GUIAction>(&mut self, f: F) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(f(ret_ref));
        ret_ref
    }

    pub fn new_pattern_editor(&mut self, parent: i64, class: VVal) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(GUIAction::NewPatternEditor(parent, ret_ref, vv2class(class)));
        ret_ref
    }

    pub fn new_button(&mut self, parent: i64, label: String, on_click: VVal, build_attribs: VVal) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(GUIAction::NewButton(parent, ret_ref, label, on_click, build_attribs));
        ret_ref
    }

    pub fn new_row(&mut self, parent: i64, class: VVal) -> i64 {
        let ret_ref = self.new_ref();
        self.actions.push(GUIAction::NewRow(parent, ret_ref, vv2class(class)));
        ret_ref
    }

    pub fn new_ref(&mut self) -> i64 {
        if !self.free_refs.is_empty() {
            if let Some(id) = self.free_refs.pop() {
                return id;
            }
        }

        let idx = self.ref_idx;
        self.ref_idx += 1;
        while self.refs.len() <= (idx as usize) {
            self.refs.push(GUIRef::None);
        }
        idx
    }

    pub fn set_root(&mut self, root: Entity) {
        if self.refs.len() < 1 {
            self.refs.push(GUIRef::Ent(root));
        } else {
            self.refs[0] = GUIRef::Ent(root);
        }
    }

    pub fn run(&mut self, self_ref: Rc<RefCell<GUIActionRecorder>>,
               wl_ctx: Rc<RefCell<EvalContext>>,
               state: &mut State, _entity: Entity)
    {
        for act in self.actions.iter() {
            match act {
                GUIAction::NewRow(parent, out, class) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        self.refs[*out as usize] = GUIRef::Ent(
                            Row::new().build(state, *parent, |builder| {
                                if let Some(class) = class {
                                    builder.class(class)
                                } else {
                                    builder
                                }
                            }));
                    }
                },
                GUIAction::NewElem(parent, out, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        self.refs[*out as usize] = GUIRef::Ent(
                            Element::new().build(
                                state, *parent,
                                |builder| vvbuilder(builder, build_attribs)));
                    }
                },
                GUIAction::NewCol(parent, out, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        self.refs[*out as usize] = GUIRef::Ent(
                            Column::new().build(
                                state, *parent,
                                |builder| vvbuilder(builder, build_attribs)));
                    }
                },
                GUIAction::NewLabel(parent, out, text, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        self.refs[*out as usize] = GUIRef::Ent(
                            Label::new(&text).build(
                                state, *parent,
                                |builder| vvbuilder(builder, build_attribs)));
                    }
                },
                GUIAction::NewHexGrid(parent, out, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        let on_click     = build_attribs.v_k("on_click");
                        let on_cell_drag = build_attribs.v_k("on_cell_drag");

                        let sr1 = self_ref.clone();
                        let sr2 = self_ref.clone();
                        let wl_ctx1 = wl_ctx.clone();
                        let wl_ctx2 = wl_ctx.clone();

                        self.refs[*out as usize] = GUIRef::Ent(
                            HexGrid::new()
                                .on_click(move |_, state, button, x, y, btn| {
                                    exec_cb(
                                        sr1.clone(), wl_ctx1.clone(),
                                        state, button, on_click.clone(),
                                        &[VVal::ivec2(x as i64, y as i64), btn2vval(btn)]);
                                })
                                .on_cell_drag(move |_, state, button, x1, y1, x2, y2, btn| {
                                    exec_cb(
                                        sr2.clone(), wl_ctx2.clone(),
                                        state, button, on_cell_drag.clone(),
                                        &[VVal::ivec2(x1 as i64, y1 as i64),
                                          VVal::ivec2(x2 as i64, y2 as i64),
                                          btn2vval(btn)]);
                                })
                                .build(state, *parent,
                                    |builder| vvbuilder(builder, build_attribs)));
                    }

                },
                GUIAction::NewHexKnob(parent, out, param_model, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        // XXX: Unwrap is checked by the
                        //      creator of GUIAction::NewHexKnob!
                        let param_model =
                            vv2hex_knob_model(param_model.clone()).unwrap();

                        self.refs[*out as usize] = GUIRef::Ent(
                            HexKnob::new(param_model)
                                .build(state, *parent,
                                    |builder| vvbuilder(builder, build_attribs)));
                    }
                },
                GUIAction::NewOctaveKeys(parent, out, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        let on_change = build_attribs.v_k("on_change");
                        let sr1       = self_ref.clone();
                        let wl_ctx1   = wl_ctx.clone();

                        self.refs[*out as usize] = GUIRef::Ent(
                            OctaveKeys::new()
                                .on_change(move |_, state, button, mask| {
                                    exec_cb(
                                        sr1.clone(), wl_ctx1.clone(),
                                        state, button, on_change.clone(),
                                        &[VVal::Int(mask)]);
                                })
                                .build(state, *parent,
                                    |builder| vvbuilder(builder, build_attribs)));
                    }
                },
                GUIAction::NewCvArray(parent, out, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        let binary    = build_attribs.v_k("binary");
                        let on_change = build_attribs.v_k("on_change");
                        let sr1       = self_ref.clone();
                        let wl_ctx1   = wl_ctx.clone();

                        self.refs[*out as usize] = GUIRef::Ent(
                            CvArray::new(binary.b())
                                .on_change(move |_, state, button, arr| {
                                    exec_cb(
                                        sr1.clone(), wl_ctx1.clone(),
                                        state, button, on_change.clone(),
                                        &[sample_buf2vv(arr.clone())]);
                                })
                                .build(state, *parent,
                                    |builder| vvbuilder(builder, build_attribs)));
                    }
                },
                GUIAction::NewConnector(parent, out, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        let on_change = build_attribs.v_k("on_change");
                        let on_hover  = build_attribs.v_k("on_hover");
                        let sr1       = self_ref.clone();
                        let wl_ctx1   = wl_ctx.clone();
                        let sr2       = self_ref.clone();
                        let wl_ctx2   = wl_ctx.clone();

                        self.refs[*out as usize] = GUIRef::Ent(
                            Connector::new()
                                .on_change(move |_, state, ent, con| {
                                    exec_cb(
                                        sr1.clone(), wl_ctx1.clone(),
                                        state, ent, on_change.clone(),
                                        &[
                                            VVal::Int(con.0 as i64),
                                            VVal::Int(con.1 as i64)
                                        ]);
                                })
                                .on_hover(move |_, state, ent, inputs, idx| {
                                    exec_cb(
                                        sr2.clone(), wl_ctx2.clone(),
                                        state, ent, on_hover.clone(),
                                        &[
                                            VVal::Bol(inputs),
                                            VVal::Int(idx as i64)
                                        ]);
                                })
                                .build(state, *parent,
                                    |builder| vvbuilder(builder, build_attribs)));
                    }
                },
                GUIAction::NewBlockCode(parent, out, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        let on_change = build_attribs.v_k("on_change");
                        let on_hover  = build_attribs.v_k("on_hover");
                        let sr1       = self_ref.clone();
                        let wl_ctx1   = wl_ctx.clone();
                        let sr2       = self_ref.clone();
                        let wl_ctx2   = wl_ctx.clone();

                        self.refs[*out as usize] = GUIRef::Ent(
                            BlockCode::new()
                                .on_change(move |_, state, ent, con| {
                                    exec_cb(
                                        sr1.clone(), wl_ctx1.clone(),
                                        state, ent, on_change.clone(),
                                        &[
                                            VVal::Int(con.0 as i64),
                                            VVal::Int(con.1 as i64)
                                        ]);
                                })
                                .on_hover(move |_, state, ent, inputs, idx| {
                                    exec_cb(
                                        sr2.clone(), wl_ctx2.clone(),
                                        state, ent, on_hover.clone(),
                                        &[
                                            VVal::Bol(inputs),
                                            VVal::Int(idx as i64)
                                        ]);
                                })
                                .build(state, *parent,
                                    |builder| vvbuilder(builder, build_attribs)));
                    }
                }
                GUIAction::NewPatternEditor(parent, out, _class) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        self.refs[*out as usize] = GUIRef::Ent(
                            PatternEditor::new(
                                hexodsp::dsp::tracker::MAX_COLS)
                                .build(state, *parent, |builder| builder));
                    }
                },
                GUIAction::NewPopup(out, build_attribs) => {
                    self.refs[*out as usize] = GUIRef::Ent(
                        Popup::new()
                            .build(state, Entity::root(),
                                |builder|
                                    vvbuilder(builder, build_attribs)));
                },
                GUIAction::NewButton(parent, out, label, on_click, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        let wl_ctx   = wl_ctx.clone();
                        let wl_ctx2  = wl_ctx.clone();
                        let on_click = on_click.clone();
                        let on_press = build_attribs.v_k("on_press");
                        let sr       = self_ref.clone();
                        let sr2      = self_ref.clone();

                        self.refs[*out as usize] = GUIRef::Ent(
                            Button::with_label(label)
                                .on_release(move |_, state, button| {
                                    exec_cb(
                                        sr.clone(), wl_ctx.clone(),
                                        state, button, on_click.clone(),
                                        &[]);
                                })
                                .on_press(move |_, state, button| {
                                    exec_cb(
                                        sr2.clone(), wl_ctx2.clone(),
                                        state, button, on_press.clone(),
                                        &[]);
                                })
                                .build(state, *parent,
                                    |builder|
                                        vvbuilder(builder, build_attribs)));
                    }
                },
                GUIAction::NewTabs(tabs, parent, build_attribs) => {
                    if let Some(GUIRef::Ent(parent)) = self.refs.get(*parent as usize) {
                        let tab_build_at = build_attribs.v_k("tab");

                        let (tab_bar, tab_viewport) =
                            TabView::new().build(state, *parent,
                                |builder| vvbuilder(
                                    builder, &build_attribs.v_k("tab_view")));

                        for (i, (tab_battribs, tab_cont_id)) in tabs.iter().enumerate() {
                            let name   = tab_battribs.v_s_rawk("name");
                            let title  = tab_battribs.v_s_rawk("title");
                            let catrib = tab_battribs.v_k("cont");

                            let tab =
                                Tab::new(&name)
                                    .build(state, tab_bar, |builder| {
                                        vvbuilder(builder.set_text(&title), &tab_build_at)
                                    });

                            println!("CONT: {}", catrib.s());
                            let container =
                                TabContainer::new(&name)
                                    .build(state, tab_viewport,
                                        |builder| vvbuilder(builder, &catrib));

                            if i == 0 {
                                tab.set_checked(state, true);
                            } else {
                                container.set_display(state, Display::None);
                            }

                            self.refs[*tab_cont_id as usize] =
                                GUIRef::Ent(container);
                        }
                    }
                },
                GUIAction::AddTheme(theme) => {
                    state.add_theme(theme);
                    println!("ADDTHEME: {}", theme);
                },
                GUIAction::Remove(id) => {
                    if let Some(GUIRef::Ent(entity)) = self.refs.get(*id as usize) {
                        state.remove(*entity);
                    }

                    self.refs[*id as usize] = GUIRef::None;
                    self.free_refs.push(*id);
                },
                GUIAction::RemoveAllChilds(id) => {
                    let mut removed_entities = vec![];

                    if let Some(GUIRef::Ent(entity)) = self.refs.get(*id as usize) {
                        for i in 0..state.tree.get_num_children(*entity).unwrap_or(0) {
                            if let Some(child) =
                                state.tree.get_child(*entity, i as usize)
                            {
                                removed_entities.push(child);
                            }
                        }
                    }

                    for dead_child in removed_entities {
                        let mut remove_idx = None;

                        state.remove(dead_child);

                        for (i, r) in self.refs.iter().enumerate() {
                            if let GUIRef::Ent(entity) = r {
                                if *entity == dead_child {
                                    remove_idx = Some(i);
                                }
                            }
                        }

                        if let Some(i) = remove_idx {
                            self.refs[i] = GUIRef::None;
                            self.free_refs.push(i as i64);
                        }
                    }
                },
                GUIAction::SetText(entity, text) => {
                    if let Some(GUIRef::Ent(entity)) = self.refs.get(*entity as usize) {
                        entity.set_text(state, text);
                    }
                },
                GUIAction::EmitTo(entity, to, event) => {
                    if let Some(GUIRef::Ent(entity)) = self.refs.get(*entity as usize) {
                        if let Some(GUIRef::Ent(to)) = self.refs.get(*to as usize) {
                            state.insert_event(
                                vv2event(event).target(*to).origin(*entity));
                        }
                    }
                },
                GUIAction::SetProp(entity, prop, v) => {
                    if let Some(GUIRef::Ent(entity)) =
                        self.refs.get(*entity as usize)
                    {
                        println!("SET PROP {} = {}", prop, v.s());

                        match &prop[..] {
                            "height" => { entity.set_height(state, vv2units(&v)); },
                            _ => {},
                        }
                    }
                },
                GUIAction::Redraw => {
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                            .target(Entity::root()));
                },
            }
        }

        self.actions.clear();
    }
}

#[derive(Lens)]
pub struct UIState {
}

impl Model for UIState {
}

struct HiddenThingie;
impl Widget for HiddenThingie {
    type Ret = Entity;
    type Data = ();

    fn on_build(&mut self, _state: &mut State, entity: Entity) -> Self::Ret {
        entity
    }

    fn on_draw(&mut self, _state: &mut State, _entity: Entity, _canvas: &mut Canvas) {
    }
}

#[macro_export]
macro_rules! set_modfun {
    ($st: expr, $ref: ident, $fun: tt, $min: expr, $max: expr, $env: ident, $argc: ident, $b: block) => {
        {
            let $ref = $ref.clone();
            $st.fun(
                &stringify!($fun),
                move |$env: &mut Env, $argc: usize| $b, $min, $max, false);
        }
    }
}

fn setup_vizia_module(r: Rc<RefCell<GUIActionRecorder>>) -> wlambda::SymbolTable {
    let mut st = wlambda::SymbolTable::new();

    set_modfun!(st, r, redraw, Some(0), Some(0), _env, _argc, {
        r.borrow_mut().actions.push(GUIAction::Redraw);
        Ok(VVal::None)
    });

    set_modfun!(st, r, set_text, Some(2), Some(2), env, _argc, {
        r.borrow_mut().actions.push(
            GUIAction::SetText(
                env.arg(0).i(), env.arg(1).s_raw()));
        Ok(VVal::None)
    });

    set_modfun!(st, r, emit_to, Some(3), Some(3), env, _argc, {
        r.borrow_mut().actions.push(
            GUIAction::EmitTo(
                env.arg(0).i(), env.arg(1).i(), env.arg(2)));
        Ok(VVal::None)
    });

    set_modfun!(st, r, add_theme, Some(1), Some(1), env, _argc, {
        r.borrow_mut().actions.push(
            GUIAction::AddTheme(env.arg(0).s_raw()));
        Ok(VVal::None)
    });

    set_modfun!(st, r, remove, Some(1), Some(1), env, _argc, {
        r.borrow_mut().actions.push(GUIAction::Remove(env.arg(0).i()));
        Ok(VVal::None)
    });

    set_modfun!(st, r, remove_all_childs, Some(1), Some(1), env, _argc, {
        r.borrow_mut().actions.push(GUIAction::RemoveAllChilds(env.arg(0).i()));
        Ok(VVal::None)
    });


    set_modfun!(st, r, new_row, Some(1), Some(2), env, _argc, {
        let mut r = r.borrow_mut();
        Ok(VVal::Int(r.new_row(env.arg(0).i(), env.arg(1))))
    });

    set_modfun!(st, r, new_col, Some(1), Some(2), env, _argc, {
        Ok(VVal::Int(r.borrow_mut().add(|id|
            GUIAction::NewCol(env.arg(0).i(), id, env.arg(1)))))
    });

    set_modfun!(st, r, new_elem, Some(1), Some(2), env, _argc, {
        Ok(VVal::Int(r.borrow_mut().add(|id|
            GUIAction::NewElem(env.arg(0).i(), id, env.arg(1)))))
    });

    set_modfun!(st, r, new_label, Some(2), Some(3), env, _argc, {
        Ok(VVal::Int(r.borrow_mut().add(|id|
            GUIAction::NewLabel(env.arg(0).i(), id, env.arg(1).s_raw(), env.arg(2)))))
    });

    set_modfun!(st, r, new_hexknob, Some(2), Some(3), env, _argc, {
        if let Some(_) = vv2hex_knob_model(env.arg(1)) {
            Ok(VVal::Int(
                r.borrow_mut().add(|id|
                    GUIAction::NewHexKnob(env.arg(0).i(), id, env.arg(1), env.arg(2)))))
        } else {
            wl_panic!(
                "ui.new_hexknob[parent_id, hex_knob_model, build_attrs] \
                not called with a $<UI::HexKnobModel>!");
        }
    });

    set_modfun!(st, r, new_octave_keys, Some(1), Some(2), env, _argc, {
        Ok(VVal::Int(
            r.borrow_mut().add(|id|
                GUIAction::NewOctaveKeys(env.arg(0).i(), id, env.arg(1)))))
    });

    set_modfun!(st, r, new_cv_array, Some(1), Some(2), env, _argc, {
        Ok(VVal::Int(
            r.borrow_mut().add(|id|
                GUIAction::NewCvArray(env.arg(0).i(), id, env.arg(1)))))
    });

    set_modfun!(st, r, new_connector, Some(1), Some(2), env, _argc, {
        Ok(VVal::Int(
            r.borrow_mut().add(|id|
                GUIAction::NewConnector(env.arg(0).i(), id, env.arg(1)))))
    });

    set_modfun!(st, r, new_block_code, Some(1), Some(2), env, _argc, {
        Ok(VVal::Int(
            r.borrow_mut().add(|id|
                GUIAction::NewBlockCode(env.arg(0).i(), id, env.arg(1)))))
    });

    set_modfun!(st, r, new_hexgrid, Some(1), Some(2), env, _argc, {
        Ok(VVal::Int(
            r.borrow_mut().add(|id|
                GUIAction::NewHexGrid(env.arg(0).i(), id, env.arg(1)))))
    });

    set_modfun!(st, r, new_pattern_editor, Some(1), Some(2), env, _argc, {
        let mut r = r.borrow_mut();
        Ok(VVal::Int(r.new_pattern_editor(env.arg(0).i(), env.arg(1))))
    });

    set_modfun!(st, r, new_tabs, Some(2), Some(3), env, _argc, {
        let mut rr = r.borrow_mut();
        let mut tabs = vec![];
        let ids = VVal::vec();

        env.arg(1).for_each(|v| {
            let id = rr.new_ref();
            tabs.push((v.clone(), id));
            ids.push(VVal::Int(id));
        });

        rr.actions.push(
            GUIAction::NewTabs(tabs, env.arg(0).i(), env.arg(2)));

        Ok(ids)
    });

    set_modfun!(st, r, new_popup, Some(0), Some(1), env, _argc, {
        Ok(VVal::Int(
            r.borrow_mut().add(|id|
                GUIAction::NewPopup(id, env.arg(0)))))
    });

    set_modfun!(st, r, new_button, Some(3), Some(4), env, _argc, {
        let mut r = r.borrow_mut();
        Ok(VVal::Int(r.new_button(
            env.arg(0).i(),
            env.arg(1).s_raw(),
            env.arg(2),
            env.arg(3)
        )))
    });

    set_modfun!(st, r, set_height, Some(2), Some(2), env, _argc, {
        r.borrow_mut().set(env.arg(0).i(), "height", env.arg(1));
        Ok(VVal::None)
    });

    st
}

fn setup_hx_module(matrix: Arc<Mutex<Matrix>>) -> wlambda::SymbolTable {
    let mut st = wlambda::SymbolTable::new();

    st.set(
        "hexo_consts_rs",
        VVal::new_str(std::include_str!("ui/widgets/mod.rs")));

    st.fun(
        "get_main_matrix_handle", move |_env: &mut Env, _argc: usize| {
            Ok(matrix2vv(matrix.clone()))
        }, Some(0), Some(0), false);

    st.fun(
        "new_cluster", move |_env: &mut Env, _argc: usize| {
            Ok(VVal::new_usr(VValCluster::new()))
        }, Some(0), Some(0), false);

    st.fun(
        "new_sample_buf_from", move |env: &mut Env, _argc: usize| {
            let mut v = vec![];
            env.arg(0).with_iter(|it| {
                for (s, _) in it {
                    v.push(s.f() as f32);
                }
            });

            Ok(VVal::new_usr(VValSampleBuf::from_vec(v)))
        }, Some(1), Some(1), false);

    st.fun(
        "dir", move |env: &mut Env, _argc: usize| {
            Ok(VVal::new_usr(VValCellDir::from_vval(&env.arg(0))))
        }, Some(1), Some(1), false);

    st.fun(
        "dir_edge", move |env: &mut Env, _argc: usize| {
            Ok(VVal::new_usr(VValCellDir::from_vval_edge(&env.arg(0))))
        }, Some(1), Some(1), false);

    st.fun(
        "to_atom", move |env: &mut Env, _argc: usize| {
            Ok(atom2vv(vv2atom(env.arg(0))))
        }, Some(1), Some(1), false);

    st.fun(
        "dir_path_from_to", move |env: &mut Env, _argc: usize| {
            let from = env.arg(0);
            let to   = env.arg(1);

            let path =
                CellDir::path_from_to(
                    (from.v_i(0) as usize, from.v_i(1) as usize),
                    (to.v_i(0)   as usize, to.v_i(1)   as usize));

            let pth = VVal::vec();
            for p in path.iter() {
                pth.push(cell_dir2vv(*p));
            }

            Ok(pth)
        }, Some(2), Some(2), false);

    st.fun(
        "pos_are_adjacent", move |env: &mut Env, _argc: usize| {
            let from = env.arg(0);
            let to   = env.arg(1);

            if let Some(dir) =
                CellDir::are_adjacent(
                    (from.v_i(0) as usize, from.v_i(1) as usize),
                    (to.v_i(0)   as usize, to.v_i(1)   as usize))
            {
                Ok(cell_dir2vv(dir))
            }
            else
            {
                Ok(VVal::None)
            }
        }, Some(2), Some(2), false);

    st.fun(
        "create_test_hex_grid_model", |_env: &mut Env, _argc: usize| {
            Ok(new_test_grid_model())
        }, Some(0), Some(0), false);

    st
}

fn main() {
    synth::start(move |matrix| {
        let matrix_recorder = Arc::new(MatrixRecorder::new());
        if let Ok(mut matrix) = matrix.lock() {
            matrix.set_observer(matrix_recorder.clone());
        }

        let global_env = wlambda::GlobalEnv::new_default();
        global_env.borrow_mut().set_module("hx",        setup_hx_module(matrix));
        global_env.borrow_mut().set_module("node_id",   wlapi::setup_node_id_module());

        let wl_ctx      = wlambda::EvalContext::new(global_env.clone());
        let wl_ctx      = Rc::new(RefCell::new(wl_ctx));
        let wl_ctx_idle = wl_ctx.clone();

        let mut idle_cb = VVal::None;

        let gui_rec      = GUIActionRecorder::new();
        let gui_rec_idle = gui_rec.clone();

        let root_entity = Rc::new(RefCell::new(Entity::null()));

        let app =
            Application::new(
                WindowDescription::new().with_inner_size(900, 760),
                |state, window| {
                    let gui_rec_self = gui_rec.clone();

                    *(root_entity.borrow_mut()) =
                        (HiddenThingie { })
                        .build(state, window, |builder| builder);

                    gui_rec_self.borrow_mut().set_root(*root_entity.borrow());

                    state.add_font_mem(
                        "font_serif",
                        std::include_bytes!("ui/widgets/font.ttf"));

                    state.add_font_mem(
                        "font_mono",
                        std::include_bytes!("ui/widgets/font_mono.ttf"));

                    state.set_default_font("font_serif");

                    let vizia_st = setup_vizia_module(gui_rec.clone());
                    global_env.borrow_mut().set_module("vizia", vizia_st);

                    match wl_ctx.borrow_mut().eval_file("wllib/main.wl") {
                        Ok(_) => { },
                        Err(e) => { panic!("Error in main.wl:\n{}", e); }
                    }

                    let init_fun =
                        wl_ctx.borrow_mut().get_global_var("init")
                           .expect("global 'init' function in main.wl defined");

                    match wl_ctx.borrow_mut().call(&init_fun, &[]) {
                        Ok(_) => {},
                        Err(e) => { panic!("Error in main.wl 'init':\n{}", e); }
                    }

                    idle_cb = wl_ctx.borrow_mut().get_global_var("idle").unwrap_or(VVal::None);

                    gui_rec.borrow_mut().run(gui_rec_self, wl_ctx, state, *(root_entity.borrow()));
                })
            .on_idle(move |state| {
                let recs = matrix_recorder.get_records();

                if idle_cb.is_some() {
                    match wl_ctx_idle
                            .borrow_mut()
                            .call(&idle_cb, &[recs])
                    {
                        Ok(_) => {},
                        Err(e) => {
                            panic!("Error in main.wl 'idle': {:?}", e);
                        }
                    }
                }

                let gui_rec_self = gui_rec_idle.clone();

                gui_rec_idle
                    .borrow_mut()
                    .run(gui_rec_self, wl_ctx_idle.clone(),
                         state, *root_entity.borrow());
            });
        app.run();
    });
}
