// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::hexo_consts::*;
use crate::rect::*;
use crate::painter::FemtovgPainter;

use tuix::*;
use femtovg::FontId;

use std::rc::Rc;
use std::cell::RefCell;

pub const UI_BG_KNOB_STROKE       : f64 = 8.0;
pub const UI_MG_KNOB_STROKE       : f64 = 3.0;
pub const UI_FG_KNOB_STROKE       : f64 = 5.0;
pub const UI_BG_KNOB_STROKE_CLR   : (f64, f64, f64) = UI_LBL_BG_CLR;
pub const UI_MG_KNOB_STROKE_CLR   : (f64, f64, f64) = UI_ACCENT_CLR;
pub const UI_FG_KNOB_STROKE_CLR   : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_FG_KNOB_MODPOS_CLR   : (f64, f64, f64) = UI_HLIGHT_CLR;
pub const UI_FG_KNOB_MODNEG_CLR   : (f64, f64, f64) = UI_SELECT_CLR;
pub const UI_TXT_KNOB_CLR         : (f64, f64, f64) = UI_PRIM_CLR;
pub const UI_TXT_KNOB_HOVER_CLR   : (f64, f64, f64) = UI_HLIGHT_CLR;
pub const UI_TXT_KNOB_MOD_CLR     : (f64, f64, f64) = UI_HLIGHT2_CLR;
pub const UI_GUI_BG_CLR           : (f64, f64, f64) = UI_BG_CLR;
pub const UI_GUI_CLEAR_CLR        : (f64, f64, f64) = UI_LBL_BG_CLR;
pub const UI_BORDER_WIDTH         : f64 = 2.0;
pub const UI_KNOB_RADIUS          : f64 = 25.0;
pub const UI_KNOB_SMALL_RADIUS    : f64 = 14.0;
pub const UI_KNOB_FONT_SIZE       : f64 = 11.0;

fn circle_point(r: f32, angle: f32) -> (f32, f32) {
    let (y, x) = angle.sin_cos();
    (x * r, y * r)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HLStyle {
    None,
    Inactive,
    Hover(bool),
}

#[derive(Debug)]
pub struct Knob {
    sbottom:        (f32, f32),
    s:              [(f32, f32); 7],
    arc_len:        [f32; 5],
    full_len:       f32,
    s1_len:         f32,
    s2_len:         f32,
    radius:         f32,
    font_size_lbl:  f32,
    font_size_data: f32,
}

impl Knob {
    pub fn new(radius: f32, font_size_lbl: f32, font_size_data: f32) -> Self {
        let init_rot : f32 = 90.;

        let mut s       = [(0.0_f32, 0.0_f32); 7];
        let mut arc_len = [0.0_f32; 5];

        let sbottom = circle_point(radius, init_rot.to_radians());

        s[0] = circle_point(radius, (init_rot + 10.0_f32).to_radians());
        s[1] = circle_point(radius, (init_rot + 60.0_f32).to_radians());
        s[2] = circle_point(radius, (init_rot + 120.0_f32).to_radians());
        s[3] = circle_point(radius, (init_rot + 180.0_f32).to_radians());
        s[4] = circle_point(radius, (init_rot + 240.0_f32).to_radians());
        s[5] = circle_point(radius, (init_rot + 300.0_f32).to_radians());
        s[6] = circle_point(radius, (init_rot + 350.0_f32).to_radians());

        let s1_len  = ((s[0].0 - s[1].1).powf(2.0) + (s[0].0 - s[1].1).powf(2.0)).sqrt();
        let s2_len  = ((s[1].0 - s[2].1).powf(2.0) + (s[1].0 - s[2].1).powf(2.0)).sqrt();

        // TODO: If I stumble across this the next time, simplify this.
        let full_len = s2_len * 2.0 + s2_len * 4.0;

        arc_len[0] = s2_len                  / full_len;
        arc_len[1] = (s2_len + s2_len)       / full_len;
        arc_len[2] = (s2_len + 2.0 * s2_len) / full_len;
        arc_len[3] = (s2_len + 3.0 * s2_len) / full_len;
        arc_len[4] = (s2_len + 4.0 * s2_len) / full_len;

        Self {
            sbottom,
            s,
            arc_len,
            full_len,
            s1_len,
            s2_len,
            radius,
            font_size_lbl,
            font_size_data,
        }
    }

    pub fn get_center_offset(&self, line_width: f32) -> (f32, f32) {
        ((self.get_label_rect().2 / 2.0).ceil() + UI_SAFETY_PAD,
         self.radius + (line_width / 2.0).ceil() + UI_SAFETY_PAD)
    }

    pub fn get_fine_adjustment_mark(&self) -> (f32, f32, f32, f32) {
        let mut r = self.get_fine_adjustment_rect();
        r.1 = (r.1 - UI_ELEM_TXT_H * 0.5).round();
        r.3 = (r.3 + UI_ELEM_TXT_H * 0.5).round();

        let mut size = (self.font_size_lbl * 0.25).round();
        if (size as i32) % 2 != 0 {
            size += 1.0;
        }
        ((r.0 + size * 1.0).round(),
         r.1 + (r.3 * 0.5 + size * 0.5).round(),
         size,
         size)
    }

    pub fn get_fine_adjustment_rect(&self) -> (f32, f32, f32, f32) {
        self.get_label_rect()
    }

    pub fn get_coarse_adjustment_rect(&self) -> (f32, f32, f32, f32) {
        let width = self.radius * 2.0;
        ((self.sbottom.0 - self.radius).round(),
         -self.radius,
         width.round(),
         (self.radius * 2.0).round())
    }

    pub fn get_value_rect(&self, double: bool) -> (f32, f32, f32, f32) {
        let width = self.radius * 2.0;
        if double {
            ((self.sbottom.0 - self.radius).round(),
             (self.sbottom.1 - (self.radius + UI_ELEM_TXT_H)).round(),
             width.round(),
             2.0 * UI_ELEM_TXT_H)
        } else {
            ((self.sbottom.0 - self.radius).round(),
             (self.sbottom.1 - (self.radius + UI_ELEM_TXT_H * 0.5)).round(),
             width.round(),
             UI_ELEM_TXT_H)
        }
    }

    pub fn get_label_rect(&self) -> (f32, f32, f32, f32) {
        let width = self.radius * 2.25;
        ((self.sbottom.0 - width * 0.5).round(),
         (self.sbottom.1 + 0.5 * UI_BG_KNOB_STROKE).round(),
         width.round(),
         UI_ELEM_TXT_H)
    }

    pub fn get_decor_rect1(&self) -> (f32, f32, f32, f32) {
        ((self.s[0].0      - 0.25 * UI_BG_KNOB_STROKE).round(),
         (self.sbottom.1    - 0.5 * UI_BG_KNOB_STROKE).round(),
         ((self.s[6].0 - self.s[0].0).abs()
                           + 0.5 * UI_BG_KNOB_STROKE).round(),
         UI_BG_KNOB_STROKE * 1.0)
    }

    pub fn draw_name(&self, p: &mut dyn Painter, x: f32, y: f32, s: &str) {
        let r = self.get_label_rect();
        p.label(
            self.font_size_lbl, 0, UI_TXT_KNOB_CLR,
            x + r.0, y + r.1, r.2, r.3, s);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_value_label(
        &self, double: bool, first: bool, p: &mut dyn Painter,
        x: f32, y: f32, highlight: HLStyle, s: &str
    ) {
        let r = self.get_value_rect(double);

        let r =
            if double {
                if first {
                    (r.0, r.1 + 1.0, r.2, UI_ELEM_TXT_H)
                } else {
                    (r.0, r.1 + UI_ELEM_TXT_H - 1.0, r.2, UI_ELEM_TXT_H)
                }
            } else {
                r
            };

        let color =
            match highlight {
                HLStyle::Hover(_fine) => { UI_TXT_KNOB_HOVER_CLR },
                HLStyle::Inactive     => { UI_INACTIVE_CLR },
                _                     => { UI_TXT_KNOB_CLR },
            };

        let some_right_padding = 6.0;
        let light_font_offs    = 4.0;

        p.label(
            self.font_size_data, 0, color,
            x + r.0 + light_font_offs,
            y + r.1,
            r.2 - some_right_padding,
            r.3, s);
    }

    pub fn draw_mod_arc(
        &self, p: &mut dyn Painter, xo: f32, yo: f32,
        value: f32, modamt: Option<f32>,
        fg_clr: (f32, f32, f32))
    {
        if let Some(modamt) = modamt {
            if modamt > 0.0 {
                self.draw_oct_arc(
                    p, xo, yo,
                    UI_MG_KNOB_STROKE,
                    UI_FG_KNOB_MODPOS_CLR,
                    None,
                    (value + modamt).clamp(0.0, 1.0));
                self.draw_oct_arc(
                    p, xo, yo,
                    UI_MG_KNOB_STROKE,
                    fg_clr,
                    Some(UI_FG_KNOB_MODPOS_CLR),
                    value);
            } else {
                self.draw_oct_arc(
                    p, xo, yo,
                    UI_MG_KNOB_STROKE,
                    UI_FG_KNOB_MODNEG_CLR,
                    Some(UI_FG_KNOB_MODNEG_CLR),
                    value);
                self.draw_oct_arc(
                    p, xo, yo,
                    UI_MG_KNOB_STROKE,
                    fg_clr,
                    None,
                    (value + modamt).clamp(0.0, 1.0));
            }
        } else {
            self.draw_oct_arc(
                p, xo, yo,
                UI_MG_KNOB_STROKE,
                fg_clr,
                Some(fg_clr),
                value);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_oct_arc(
        &self, p: &mut dyn Painter, x: f32, y: f32, line_w: f32,
        color: (f32, f32, f32),
        dot_color: Option<(f32, f32, f32)>,
        value: f32
    ) {
        let arc_len = &self.arc_len;

        let (next_idx, prev_arc_len) =
            if value > arc_len[4] {
                (6, arc_len[4])
            } else if value > arc_len[3] {
                (5, arc_len[3])
            } else if value > arc_len[2] {
                (4, arc_len[2])
            } else if value > arc_len[1] {
                (3, arc_len[1])
            } else if value > arc_len[0] {
                (2, arc_len[0])
            } else {
                (1, 0.0)
            };

        let mut s : [(f32, f32); 7] = self.s;
        for p in s.iter_mut() {
            p.0 += x;
            p.1 += y;
        }

        // The segment len is used to calculate the ratio of the traveled
        // total length.
        let segment_len = self.s2_len;
        let prev       = s[next_idx - 1];
        let last       = s[next_idx];
        let rest_len   = value - prev_arc_len;
        let rest_ratio = rest_len / (segment_len / self.full_len);
//        println!("i[{}] prev_arc_len={:1.3}, rest_len={:1.3}, value={:1.3}, seglen={:1.3}",
//                 next_idx, prev_arc_len, rest_len, value,
//                 segment_len / self.full_len);
        let partial =
            ((last.0 - prev.0) * rest_ratio,
             (last.1 - prev.1) * rest_ratio);

        s[next_idx] = (
            prev.0 + partial.0,
            prev.1 + partial.1
        );

        if let Some(clr) = dot_color {
            p.arc_stroke(
                0.9 * line_w * 0.5,
                clr,
                0.9 * line_w * 1.5,
                0.0, 2.0 * std::f32::consts::PI,
                prev.0 + partial.0,
                prev.1 + partial.1);
        }

        p.path_stroke(line_w, color, &mut s.iter().copied().take(next_idx + 1), false);
    }
}


pub trait ParamModel {
    /// Should return the normalized paramter value.
    fn get(&self) -> f32 { self.value }

    /// Should return true if the UI for the parameter can be changed
    /// by the user. In HexoSynth this might return false if the
    /// corresponding input is controlled by an output port.
    fn enabled(&self) -> bool;

    /// Should return a value in the range 0.0 to 1.0 for displayed knob position.
    /// For instance: a normalized value in the range -1.0 to 1.0 needs to be mapped
    /// to 0.0 to 1.0 by: `(x + 1.0) * 0.5`
    fn get_ui_range(&self) -> Option<f32>;

    /// Should return the modulation amount for the 0..1 UI knob range.
    /// Internally you should transform that into the appropriate
    /// modulation amount in relation to what [get_ui_range] returns.
    fn get_ui_mod_amt(&self) -> Option<f32>;

    fn fmt(&self, buf: &mut [u8]) -> usize;
    fn fmt_mod(&self, buf: &mut [u8]) -> usize;
    fn fmt_norm(&self, buf: &mut [u8]) -> usize;
    fn fmt_name(&self, buf: &mut [u8]) -> usize;

    fn get_denorm(&self) -> f32;
}

pub struct DummyParamModel {
    value: f32,
    modamt: Option<f32>,
}

impl DummyParamModel {
    pub fn new() -> Self {
        Self {
            value: 0.25,
            modamt: Some(0.25),
        }
    }
}

impl ParamModel for DummyParamModel {
    fn enabled(&self) -> bool { self.get() > 0.1 }
    fn get_ui_mod_amt(&self) -> Option<f32> { self.modamt }
    fn get_ui_range(&self) -> Option<f32> { self.get() }
    fn get_denorm(&self) -> f32 { self.get() * 100.0 }
    fn get(&self) -> f32 { self.value }

    fn fmt_name<'a>(&self, buf: &'a mut [u8]) -> usize {
        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);
        match write!(bw, "{}", "dummy") {
            Ok(_)  => bw.buffer().len(),
            Err(_) => 0,
        }
    }

    fn fmt_norm<'a>(&self, buf: &'a mut [u8]) -> usize {
        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);
        match write!(bw, "{:6.4}", self.get()) {
            Ok(_)  => bw.buffer().len(),
            Err(_) => 0,
        }
    }

    fn fmt_mod<'a>(&self, buf: &'a mut [u8]) -> usize {
        let modamt =
            if let Some(ma) = self.modamt {
                ma
            } else {
                return 0;
            };
        let norm = self.get();

        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);
        match write!(bw, "{:6.3}", norm + modamt) {
            Ok(_)  => bw.buffer().len(),
            Err(_) => 0,
        }
    }

    fn fmt<'a>(&self, buf: &'a mut [u8]) -> usize {
        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);
        match write!(bw, "{:6.3}", self.get_denorm()) {
            Ok(_)  => bw.buffer().len(),
            Err(_) => 0,
        }
    }
}

pub struct HexKnob {
    font:      Option<FontId>,
    font_mono: Option<FontId>,
    lbl_buf:   [u8; 15],
    model:     Rc<RefCell<dyn DummyParamModel>>,
    knob:      Knob,
}

impl HexKnob {
    pub fn new(size: f32) -> Self {
        HexKnob {
            font:       None,
            font_mono:  None,
            lbl_buf:    [0; 15],
            model:      Rc::new(RefCell::new(DummyParamModel::new())),
            // TODO: compute Knob parameters at draw time dependent on the
            //       space the knob has! Same with the font sizes!
            knob:       Knob::new(50.0, 10.0, 10.0),
        }
    }
}

impl Widget for HexKnob {
    type Ret  = Entity;
    type Data = Rc<RefCell<dyn ParamModel>>;

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity.set_position_type(state, PositionType::ParentDirected)
              .set_clip_widget(state, entity)
    }

    fn on_update(&mut self, state: &mut State, entity: Entity, data: &Self::Data) {
        self.model = data.clone();
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(window_event) = event.message.downcast::<WindowEvent>() {
            println!("EV: {:?}", window_event);

            match window_event {
                WindowEvent::MouseDown(btn) => {
                },
                WindowEvent::MouseUp(btn) => {
                },
                WindowEvent::MouseMove(x, y) => {
                },
                WindowEvent::MouseScroll(x, y) => {
                },
                _ => {},
            }
        }
    }

    fn on_draw(&mut self, state: &mut State, entity: Entity, canvas: &mut Canvas) {
        if self.font.is_none() {
            self.font      = Some(canvas.add_font_mem(std::include_bytes!("font.ttf")).expect("can load font"));
            self.font_mono = Some(canvas.add_font_mem(std::include_bytes!("font_mono.ttf")).expect("can load font"));
        }

        let bounds = state.data.get_bounds(entity);

        let p = &mut FemtovgPainter {
            canvas:     canvas,
            font:       self.font.unwrap(),
            font_mono:  self.font_mono.unwrap(),
        };

        let pos : Rect = bounds.into();

        let (xo, yo) = (
            (pos.x + pos.w / 2.0).round(),
            (pos.y + pos.h / 2.0).round()
        );

        let id     = data.id();
        let modamt = model.get_ui_mod_amt();

        self.knob.draw_oct_arc(
            p, xo, yo,
            UI_BG_KNOB_STROKE,
            UI_BG_KNOB_STROKE_CLR,
            None,
            1.0);

        let dc1 = self.knob.get_decor_rect1();
        p.rect_fill(
            UI_BG_KNOB_STROKE_CLR,
            xo + dc1.0, yo + dc1.1, dc1.2, dc1.3);

        let valrect = self.knob.get_value_rect(modamt.is_some());
        p.rect_fill(
            UI_BG_KNOB_STROKE_CLR,
            valrect.0 + xo, valrect.1 + yo, valrect.2, valrect.3);

        let lblrect = self.knob.get_label_rect();
        p.rect_fill(
            UI_BG_KNOB_STROKE_CLR,
            lblrect.0 + xo, lblrect.1 + yo, lblrect.2, lblrect.3);

        let r = self.knob.get_fine_adjustment_mark();
        p.rect_fill(
            UI_BG_KNOB_STROKE_CLR,
            xo + r.0, yo + r.1, r.2, r.3);

        let highlight = ui.hl_style_for(id, None);
        let value =
            if let Some(v) = model.get_ui_range(id) {
                (v as f32).clamp(0.0, 1.0)
            } else { 0.0 };

        let mut hover_fine_adj = false;

        // TODO: Get hover status from `self` (fine vs coarse area)
        let hover = false;
        let fine_hover = false;

        let highlight =
            if !model.enabled() {
                HLStyle::Inactive
            } else if hover || fine_hover {
                HLStyle::Hover(fine_hover)
            } else {
                HLStyle::None
            };

        match highlight {
            HLStyle::Inactive => {
                self.knob.draw_oct_arc(
                    p, xo, yo,
                    UI_MG_KNOB_STROKE,
                    UI_INACTIVE_CLR,
                    None,
                    1.0);

                self.knob.draw_mod_arc(
                    p, xo, yo, value, modamt,
                    UI_INACTIVE2_CLR);
            },
            HLStyle::Hover(fine) => {
                if fine_hover {
                    hover_fine_adj = true;

                    let r = self.knob.get_fine_adjustment_mark();
                    p.rect_fill(
                        UI_TXT_KNOB_HOVER_CLR,
                        xo + r.0, yo + r.1, r.2, r.3);
                }

                self.knob.draw_oct_arc(
                    p, xo, yo,
                    UI_MG_KNOB_STROKE,
                    UI_MG_KNOB_STROKE_CLR,
                    None,
                    1.0);

                self.knob.draw_mod_arc(
                    p, xo, yo, value, modamt,
                    UI_FG_KNOB_STROKE_CLR);

            },
            HLStyle::None => {
                self.knob.draw_oct_arc(
                    p, xo, yo,
                    UI_MG_KNOB_STROKE,
                    UI_MG_KNOB_STROKE_CLR,
                    None,
                    1.0);

                self.knob.draw_mod_arc(
                    p, xo, yo, value, modamt,
                    UI_FG_KNOB_STROKE_CLR);

            },
        }

        //---------------------------------------------------------------------------

        let len = model.fmt(&mut self.lbl_buf[..]);
        let val_s = std::str::from_utf8(&self.lbl_buf[0..len]).unwrap();
        self.draw_value_label(modamt.is_some(), true, p, xo, yo, highlight, val_s);

        if modamt.is_some() {
            let len = model.fmt_mod(&mut self.lbl_buf[..]);
            let val_s = std::str::from_utf8(&self.lbl_buf[0..len]).unwrap();
            self.draw_value_label(true, false, p, xo, yo, highlight, val_s);
        }

        if hover_fine_adj {
            let len = model.fmt_norm(&mut self.lbl_buf[..]);
            let val_s = std::str::from_utf8(&self.lbl_buf[0..len]).unwrap();
            // + 2.0 for the marker cube, to space it from the minus sign.
            self.knob.draw_name(p, xo + 2.0, yo, &val_s);
        } else {
            let len = model.fmt_name(&mut self.lbl_buf[..]);
            let val_s = std::str::from_utf8(&self.lbl_buf[0..len]).unwrap();
            self.knob.draw_name(p, xo, yo, &val_s);
        }
//
//        ui.define_active_zone(
//            ActiveZone::new_drag_zone(
//                id,
//                Rect::from_tpl(self.get_coarse_adjustment_rect()).offs(xo, yo), true)
//            .dbgid(DBGID_KNOB_COARSE));
//        ui.define_active_zone(
//            ActiveZone::new_drag_zone(
//                id,
//                Rect::from_tpl(self.get_fine_adjustment_rect()).offs(xo, yo), false)
//            .dbgid(DBGID_KNOB_FINE));
    }
}

