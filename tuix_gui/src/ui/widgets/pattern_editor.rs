// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::ui::*;

use tuix::*;
use femtovg::FontId;

pub use hexodsp::dsp::tracker::UIPatternModel;
pub use hexodsp::dsp::tracker::PatternData;

use std::sync::{Arc, Mutex};

pub const UI_TRK_ROW_HEIGHT        : f32 = 14.0;
pub const UI_TRK_COL_WIDTH         : f32 = 38.0;
pub const UI_TRK_FONT_SIZE         : f32 = 12.0;
pub const UI_TRK_COL_DIV_PAD       : f32 = 3.0;
pub const UI_TRK_BG_CLR            : (f32, f32, f32) = UI_LBL_BG_CLR;
pub const UI_TRK_BG_ALT_CLR        : (f32, f32, f32) = UI_LBL_BG_ALT_CLR;
pub const UI_TRK_COL_DIV_CLR       : (f32, f32, f32) = UI_PRIM2_CLR;
//pub const UI_TRK_BORDER_CLR        : (f32, f32, f32) = UI_ACCENT_CLR;
//pub const UI_TRK_BORDER_HOVER_CLR  : (f32, f32, f32) = UI_HLIGHT_CLR;
//pub const UI_TRK_BORDER_EDIT_CLR   : (f32, f32, f32) = UI_SELECT_CLR;
//pub const UI_TRK_BORDER_INACT_CLR  : (f32, f32, f32) = UI_INACTIVE_CLR;
pub const UI_TRK_TEXT_CLR          : (f32, f32, f32) = UI_TXT_CLR;
pub const UI_TRK_CURSOR_BG_CLR     : (f32, f32, f32) = UI_PRIM2_CLR;
pub const UI_TRK_CURSOR_BG_HOV_CLR : (f32, f32, f32) = UI_PRIM_CLR;
pub const UI_TRK_CURSOR_BG_SEL_CLR : (f32, f32, f32) = UI_SELECT_CLR;
pub const UI_TRK_CURSOR_FG_CLR     : (f32, f32, f32) = UI_LBL_BG_CLR;
pub const UI_TRK_PHASEROW_BG_CLR   : (f32, f32, f32) = UI_HLIGHT_CLR;
pub const UI_TRK_PHASEROW_FG_CLR   : (f32, f32, f32) = UI_LBL_BG_CLR;

#[derive(Debug)]
pub struct PatternEditor {
    font:       Option<FontId>,
    font_mono:  Option<FontId>,
    rows:       usize,
    columns:    usize,

    pattern:        Arc<Mutex<dyn UIPatternModel>>,
    cursor:         (usize, usize),
    enter_mode:     EnterMode,

    cell_zone:      Rect,

    last_set_value: u16,

    edit_step:      usize,
    octave:         u16,
    follow_phase:   bool,
    info_line:      String,
    update_info_line: bool,
}

impl PatternEditor {
    pub fn new(columns: usize) -> Self {
        Self {
            font:       None,
            font_mono:  None,
            rows:       10,
            columns,

            pattern: Arc::new(Mutex::new(PatternData::new(256))),
            cursor: (1, 2),
            enter_mode: EnterMode::None,

            cell_zone: Rect::from(0.0, 0.0, 0.0, 0.0),

            last_set_value: 0,

            edit_step: 4,
            octave:    4,
            follow_phase: false,
            update_info_line: true,
            info_line: String::from(""),
        }
    }

    pub fn calc_row_offs(&self, rows: usize) -> usize {
        let rows = rows as i64;
        let mut cur = self.cursor.0 as i64;

        let margin = rows * 1 / 3;
        let margin = (margin / 2) * 2;
        let page_rows = rows - margin;

        if page_rows <= 0 {
            return cur as usize;
        }

        let mut scroll_page = 0;

        while cur >= page_rows {
            cur -= page_rows;
            scroll_page += 1;
        }

        if scroll_page > 0 {
            (scroll_page * page_rows - (margin / 2)) as usize
        } else {
            0
        }
    }

    fn handle_key_event(&mut self, state: &mut State, key: &Key) {
        let mut pat = self.pattern.lock().unwrap();

        let mut edit_step = self.edit_step as i16;

        if state.modifiers.ctrl {
            edit_step = 1;
        }

        if edit_step < 1 { edit_step = 1; }

        let octave = self.octave;

        let mut reset_entered_value = false;

        match key {
            Key::Home => {
                self.cursor.0 = 0;
                reset_entered_value = true;
            },
            Key::End => {
                self.cursor.0 = pat.rows() - self.edit_step;
                reset_entered_value = true;
            },
            Key::PageUp => {
                if state.modifiers.shift {
                    pat.change_value(
                        self.cursor.0,
                        self.cursor.1,
                        0x100);

                } else {
                    advance_cursor(
                        &mut self.cursor,
                        -2 * edit_step as i16,
                        0, &mut *pat);
                }
                reset_entered_value = true;
            },
            Key::PageDown => {
                if state.modifiers.shift {
                    pat.change_value(
                        self.cursor.0,
                        self.cursor.1,
                        -0x100);

                } else {
                    advance_cursor(
                        &mut self.cursor,
                        2 * edit_step as i16,
                        0, &mut *pat);
                }
                reset_entered_value = true;
            },
            Key::ArrowUp => {
                if state.modifiers.shift {
                    if state.modifiers.ctrl {
                        pat.change_value(
                            self.cursor.0,
                            self.cursor.1,
                            0x100);
                    } else {
                        pat.change_value(
                            self.cursor.0,
                            self.cursor.1,
                            0x10);
                    }

                } else if let EnterMode::Rows(_) = self.enter_mode {
                    let rows = pat.rows() + 1;
                    pat.set_rows(rows);
                    self.update_info_line = true;

                } else {
                    advance_cursor(
                        &mut self.cursor,
                        -edit_step as i16,
                        0, &mut *pat);
                }
                reset_entered_value = true;
            },
            Key::ArrowDown => {
                if state.modifiers.shift {
                    if state.modifiers.ctrl {
                        pat.change_value(
                            self.cursor.0,
                            self.cursor.1,
                            -0x100);
                    } else {
                        pat.change_value(
                            self.cursor.0,
                            self.cursor.1,
                            -0x10);
                    }

                } else if let EnterMode::Rows(_) = self.enter_mode {
                    if pat.rows() > 0 {
                        let rows = pat.rows() - 1;
                        pat.set_rows(rows);
                        self.update_info_line = true;
                    }
                } else {
                    advance_cursor(
                        &mut self.cursor,
                        edit_step as i16,
                        0, &mut *pat);
                }
                reset_entered_value = true;
            },
            Key::ArrowLeft => {
                if state.modifiers.shift {
                    pat.change_value(
                        self.cursor.0,
                        self.cursor.1,
                        -0x1);

                } else {
                    advance_cursor(
                        &mut self.cursor, 0, -1, &mut *pat);
                }
                reset_entered_value = true;
            },
            Key::ArrowRight => {
                if state.modifiers.shift {
                    pat.change_value(
                        self.cursor.0,
                        self.cursor.1,
                        0x1);

                } else {
                    advance_cursor(
                        &mut self.cursor, 0, 1, &mut *pat);
                }
                reset_entered_value = true;
            },
            Key::Delete => {
                pat.clear_cell(
                    self.cursor.0,
                    self.cursor.1);
                advance_cursor(
                    &mut self.cursor,
                    edit_step as i16, 0, &mut *pat);
                reset_entered_value = true;
            },
            Key::Character(c) => {
                match &c[..] {
                    "+" => {
                        self.octave += 1;
                        self.octave = self.octave.min(9);
                        self.update_info_line = true;
                    },
                    "-" => {
                        if self.octave > 0 {
                            self.octave -= 1;
                            self.update_info_line = true;
                        }
                    },
                    "/" => {
                        if self.edit_step > 0 {
                            self.edit_step -= 1;
                        }
                        self.update_info_line = true;
                    },
                    "*" => {
                        self.edit_step += 1;
                        self.update_info_line = true;
                    },
                    _ => {},
                }

                match self.enter_mode {
                    EnterMode::EnterValues(v) => {
                        match &c[..] {
                            "." => {
                                pat.set_cell_value(
                                    self.cursor.0,
                                    self.cursor.1,
                                    self.last_set_value);
                                advance_cursor(
                                    &mut self.cursor,
                                    edit_step as i16, 0, &mut *pat);
                                reset_entered_value = true;
                            },
                            "," => {
                                let cell_value =
                                    pat.get_cell_value(
                                        self.cursor.0,
                                        self.cursor.1);
                                self.last_set_value = cell_value;
                                advance_cursor(
                                    &mut self.cursor,
                                    edit_step as i16, 0, &mut *pat);
                                reset_entered_value = true;
                            },
                            _ if pat.is_col_note(self.cursor.1) => {
                                if let Some(value) =
                                    note_from_char(&c[..], octave)
                                {
                                    pat.set_cell_value(
                                        self.cursor.0,
                                        self.cursor.1,
                                        value as u16);
                                    advance_cursor(
                                        &mut self.cursor,
                                        edit_step as i16, 0, &mut *pat);
                                    self.last_set_value = value as u16;
                                }
                            },
                            _ => {
                                if let Some(value) = num_from_char(&c[..]) {
                                    match v {
                                        EnterValue::None => {
                                            let nv = value << 0x8;
                                            self.enter_mode =
                                                EnterMode::EnterValues(
                                                    EnterValue::One(nv as u16));
                                            pat.set_cell_value(
                                                self.cursor.0,
                                                self.cursor.1,
                                                nv as u16);
                                            self.last_set_value = nv as u16;
                                        },
                                        EnterValue::One(v) => {
                                            let nv = v | (value << 0x4);
                                            self.enter_mode =
                                                EnterMode::EnterValues(
                                                    EnterValue::Two(nv as u16));
                                            pat.set_cell_value(
                                                self.cursor.0,
                                                self.cursor.1,
                                                nv as u16);
                                            self.last_set_value = nv as u16;
                                        },
                                        EnterValue::Two(v) => {
                                            let nv = v | value;
                                            self.enter_mode =
                                                EnterMode::EnterValues(
                                                    EnterValue::None);
                                            pat.set_cell_value(
                                                self.cursor.0,
                                                self.cursor.1,
                                                nv as u16);
                                            self.last_set_value = nv as u16;
                                            advance_cursor(
                                                &mut self.cursor,
                                                edit_step as i16, 0, &mut *pat);
                                        },
                                    }
                                }
                            },
                        }
                    },
                    EnterMode::Rows(v) => {
                        match v {
                            EnterValue::None => {
                                if let Some(value) = num_from_char(&c[..]) {
                                    pat.set_rows((value << 4) as usize);
                                    self.update_info_line = true;
                                    self.enter_mode =
                                        EnterMode::Rows(EnterValue::One(value));
                                }
                            },
                            EnterValue::One(v) => {
                                if let Some(value) = num_from_char(&c[..]) {
                                    pat.set_rows((v << 4 | value) as usize);
                                    self.update_info_line = true;
                                    self.enter_mode = EnterMode::None;
                                }
                            },
                            _ => {
                                self.enter_mode = EnterMode::None;
                            },
                        }
                    },
                    EnterMode::EditStep => {
                        if let Some(value) = num_from_char(&c[..]) {
                            if state.modifiers.ctrl {
                                self.edit_step = (value + 0x10) as usize;
                            } else {
                                self.edit_step = value as usize;
                            }
                            self.update_info_line = true;
                        }

                        self.enter_mode = EnterMode::None;
                    },
                    EnterMode::Octave => {
                        if let Some(value) = num_from_char(&c[..]) {
                            self.octave = value;
                            self.update_info_line = true;
                        }

                        self.enter_mode = EnterMode::None;
                    },
                    EnterMode::ColType => {
                        match &c[..] {
                            "n" => {
                                pat.set_col_note_type(self.cursor.1);
                            },
                            "s" => {
                                pat.set_col_step_type(self.cursor.1);
                            },
                            "v" => {
                                pat.set_col_value_type(self.cursor.1);
                            },
                            "g" => {
                                pat.set_col_gate_type(self.cursor.1);
                            },
                            _ => {},
                        }
                        self.enter_mode = EnterMode::None;
                    },
                    EnterMode::Delete => {
                        match &c[..] {
                            "r" => {
                                for i in 0..pat.cols() {
                                    pat.clear_cell(
                                        self.cursor.0,
                                        i);
                                }
                            },
                            "c" => {
                                for i in 0..pat.rows() {
                                    pat.clear_cell(
                                        i,
                                        self.cursor.1);
                                }
                            },
                            "s" => {
                                for i in 0..self.edit_step {
                                    pat.clear_cell(
                                        self.cursor.0 + i,
                                        self.cursor.1);
                                }
                            },
                            _ => {},
                        }
                        self.enter_mode = EnterMode::None;
                    },
                    EnterMode::None => {
                        match &c[..] {
                            "e" => {
                                self.enter_mode = EnterMode::EditStep;
                            },
                            "r" => {
                                self.enter_mode =
                                    EnterMode::Rows(EnterValue::None);
                            },
                            "o" => {
                                self.enter_mode = EnterMode::Octave;
                            },
                            "c" => {
                                self.enter_mode = EnterMode::ColType;
                            },
                            "d" => {
                                self.enter_mode = EnterMode::Delete;
                            },
                            "f" => {
                                self.follow_phase = !self.follow_phase;
                                self.update_info_line = true;
                            },
                            _ => {},
                        }
                    },
                }
            },
            Key::Escape => {
                self.enter_mode = EnterMode::None;
            },
            Key::Enter => {
                self.enter_mode =
                    match self.enter_mode {
                        EnterMode::EnterValues(_)
                            => EnterMode::None,
                        _   => EnterMode::EnterValues(EnterValue::None),
                    }
            },
            _ => {},
        }

        if reset_entered_value {
            if let EnterMode::EnterValues(_) = self.enter_mode {
                self.enter_mode = EnterMode::EnterValues(EnterValue::None);
            }
        }
    }
}

impl Widget for PatternEditor {
    type Ret  = Entity;
    type Data = Arc<Mutex<dyn UIPatternModel>>;

    fn widget_name(&self) -> String {
        "patterneditor".to_string()
    }

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity
            .set_width(state, Pixels(UI_TRK_COL_WIDTH * 7.0))
            .set_element(state, "pattern-editor")
            .set_focusable(state, true)
            .set_hoverable(state, true)
    }

    fn on_update(&mut self, _state: &mut State, _entity: Entity, data: &Self::Data) {
        self.pattern = data.clone();
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(window_event) = event.message.downcast::<WindowEvent>() {

            match window_event {
                WindowEvent::FocusOut => {
                    self.enter_mode = EnterMode::None;
                },
                  WindowEvent::MouseDown(MouseButton::Left)
                | WindowEvent::MouseDown(MouseButton::Right) => {

                    let bounds = state.data.get_bounds(entity);

                    let x = state.mouse.cursorx - bounds.x;
                    let y = state.mouse.cursory - bounds.y;

                    let pat = self.pattern.lock().unwrap();

                    let xi = (x - self.cell_zone.x) / UI_TRK_COL_WIDTH;
                    let yi = (y - self.cell_zone.y) / UI_TRK_ROW_HEIGHT;

                    let xi = xi.max(1.0);
                    let yi = yi.max(1.0);

                    let row_scroll_offs = self.calc_row_offs(self.rows);
                    let yr = (yi as usize - 1) + row_scroll_offs;

                    //d// println!("INDEX: {} {},{} => {},{}", index, x, y, xi, yi);
                    self.cursor = (yr, xi as usize - 1);

                    self.cursor.0 = self.cursor.0.min(pat.rows() - 1);
                    self.cursor.1 = self.cursor.1.min(pat.cols() - 1);

                    state.set_focus(entity);

                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                            .target(Entity::root()));
                },
//                WindowEvent::MouseUp(MouseButton::Middle) => {
//                },
//                WindowEvent::MouseUp(btn) => {
//                },
//                WindowEvent::MouseMove(x, y) => {
//                },
                WindowEvent::MouseScroll(_x, y) => {
                    let pat = self.pattern.lock().unwrap();

                    if *y > 0.0 {
                        if self.cursor.0 > 0 {
                            self.cursor.0 -= 1;
                        }
                    } else {
                        if (self.cursor.0 + 1) < pat.rows() {
                            self.cursor.0 += 1;
                        }
                    }

                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                            .target(Entity::root()));
                },
                WindowEvent::CharInput(c) => {
                    self.handle_key_event(
                        state,
                        &Key::Character(c.to_string()));

                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                            .target(Entity::root()));
                },
                WindowEvent::KeyDown(_code, key) => {
                    //d// println!("KEY: {:?}", key);

                    if let Some(key) = key {
                        self.handle_key_event(state, key);
                    }

                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                            .target(Entity::root()));
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
        let pos = Rect {
            x: pos.x.floor(),
            y: pos.y.floor(),
            w: pos.w.floor(),
            h: pos.h.floor(),
        };

        p.clip_region(pos.x, pos.y, pos.w, pos.h);

//        let id        = data.id();
//        let highlight = ui.hl_style_for(id, None);

        let orig_pos  = pos;

        let mut pat = self.pattern.lock().unwrap();

        if self.cursor.0 >= pat.rows() {
            self.cursor.0 = pat.rows() - 1;
        }

        let notify_click = false;

//        let border_color =
//                match highlight {
//                    HLStyle::Hover(_) => {
//                        if data.enter_mode != EnterMode::None {
//                            UI_TRK_BORDER_EDIT_CLR
//                        } else {
//                            UI_TRK_BORDER_HOVER_CLR
//                        }
//                    },
//                    HLStyle::Inactive => {
//                        notify_click = true;
//                        UI_TRK_BORDER_INACT_CLR
//                    },
//                    _ => {
//                        data.enter_mode = EnterMode::None;
//                    UI_TRK_BORDER_CLR
//                    },
//                };
//        ;

        p.rect_fill(UI_TRK_BG_CLR, pos.x, pos.y, pos.w, pos.h);

//        let pos =
//            rect_border(p, UI_TRK_BORDER, border_color, UI_TRK_BG_CLR, pos);

        let mode_line =
            match self.enter_mode {
                EnterMode::EnterValues(_) => {
                    Some("> [Values]")
                },
                EnterMode::EditStep => {
                    Some("> [Step] (0-F, Ctrl + 0-F)")
                },
                EnterMode::Octave => {
                    Some("> [Octave] (0-8)")
                },
                EnterMode::ColType => {
                    Some("> [Column] (n)ote,(s)tep,(v)alue,(g)ate")
                },
                EnterMode::Delete => {
                    Some("> [Delete] (r)ow,(c)olumn,(s)tep")
                },
                EnterMode::Rows(EnterValue::None) => {
                    Some("> [Rows] (0-F 00-F0, Up/Down +1/-1)")
                },
                EnterMode::Rows(EnterValue::One(_)) => {
                    Some("> [Rows] (0-F 00-0F)")
                },
                EnterMode::Rows(EnterValue::Two(_)) => None,
                EnterMode::None => {
                    if notify_click {
                        Some("*** >>> click for keyboard focus <<< ***")
                    } else {
                        None
                    }
                },
            };

        if let Some(mode_line) = mode_line {
            p.label_mono(
                UI_TRK_FONT_SIZE * 0.9,
                -1,
                UI_TRK_TEXT_CLR,
                pos.x,
                pos.y,
                pos.w,
                UI_TRK_ROW_HEIGHT,
                &mode_line);
        }

        if self.update_info_line {
            self.info_line =
                format!(
                    "ES: {:02} | Oct: {:02} | Curs: {} | R: {:02}",
                    self.edit_step,
                    self.octave,
                    if self.follow_phase { "->" }
                    else                 { "." },
                    pat.rows());
            self.update_info_line = false;
        }

        p.label_mono(
            UI_TRK_FONT_SIZE,
            -1,
            UI_TRK_TEXT_CLR,
            pos.x,
            pos.y + UI_TRK_ROW_HEIGHT,
            pos.w,
            UI_TRK_ROW_HEIGHT,
            &self.info_line);

//            ui.define_active_zone(
//                ActiveZone::new_keyboard_zone(id, pos)
//                .dbgid(DBGID_PATEDIT));

        for ic in 0..self.columns {
            let x = (ic + 1) as f32 * UI_TRK_COL_WIDTH;
            let y = 2.0             * UI_TRK_ROW_HEIGHT;

            p.label_mono(
                UI_TRK_FONT_SIZE,
                0,
                UI_TRK_TEXT_CLR,
                pos.x + x,
                pos.y + y,
                UI_TRK_COL_WIDTH,
                UI_TRK_ROW_HEIGHT,
                if pat.is_col_note(ic) {
                    "Note"
                } else if pat.is_col_step(ic) {
                    "Step"
                } else if pat.is_col_gate(ic) {
                    "Gate"
                } else {
                    "Value"
                });
        }

        p.path_stroke(
            1.0,
            UI_TRK_COL_DIV_CLR,
            &mut [
                (pos.x,         pos.y + 3.0 * UI_TRK_ROW_HEIGHT - 0.5),
                (pos.x + pos.w, pos.y + 3.0 * UI_TRK_ROW_HEIGHT - 0.5),
            ].iter().copied(),
            false);

        let pos = pos.crop_top(2.0 * UI_TRK_ROW_HEIGHT);

        self.cell_zone = Rect {
            x: pos.x - orig_pos.x,
            y: pos.y - orig_pos.y,
            w: pos.w,
            h: pos.h,
        };

        self.rows = (self.cell_zone.h / UI_TRK_ROW_HEIGHT).round() as usize - 1;

        // center the cursor row
        // - but lock the start of the pattern to the top
        // - and lock the end of the pattern to the end
        let row_scroll_offs = self.calc_row_offs(self.rows);

        for ir in 0..self.rows {
            let y = (ir + 1) as f32 * UI_TRK_ROW_HEIGHT;
            let ir = row_scroll_offs as usize + ir;

            if ir >= pat.rows() {
                break;
            }

            if self.edit_step > 0 && ir % self.edit_step == 0 {
                p.rect_fill(
                    UI_TRK_BG_ALT_CLR,
                    pos.x,
                    pos.y + y,
                    pos.w,
                    UI_TRK_ROW_HEIGHT);
            }

            p.label_mono(
                UI_TRK_FONT_SIZE,
                1,
                UI_TRK_TEXT_CLR,
                pos.x - UI_TRK_COL_DIV_PAD,
                pos.y + y,
                UI_TRK_COL_WIDTH,
                UI_TRK_ROW_HEIGHT,
                &format!("{:-02}", ir));

            // TODO: FIXME: We need to find a good way to access phase values!
            let phase = 0.0;
//                if let Some(phase) = ui.atoms().get_phase_value(id) {
//                    phase as f32
//                } else { 0.0 };

            let phase_row = (pat.rows() as f32 * phase).floor() as usize;

            if self.follow_phase {
                self.cursor.0 = phase_row;
            }

            for ic in 0..self.columns {
                let x = (ic + 1) as f32 * UI_TRK_COL_WIDTH;
                let is_note_col = pat.is_col_note(ic);

                let txt_clr =
                    if (ir, ic) == self.cursor || ir == phase_row {
                        p.rect_fill(
                            if (ir, ic) == self.cursor {
                                if state.focused == entity {
                                    UI_TRK_CURSOR_BG_SEL_CLR
                                } else if state.hovered == entity {
                                    UI_TRK_CURSOR_BG_HOV_CLR
                                } else {
                                    UI_TRK_CURSOR_BG_CLR
                                }
                            } else { UI_TRK_PHASEROW_BG_CLR },
                            pos.x + x,
                            pos.y + y,
                            UI_TRK_COL_WIDTH,
                            UI_TRK_ROW_HEIGHT);

                        if (ir, ic) == self.cursor {
                            UI_TRK_CURSOR_FG_CLR
                        } else {
                            UI_TRK_PHASEROW_FG_CLR
                        }
                    } else if
                           (   state.focused == entity
                            || state.hovered == entity)
                        && ir == self.cursor.0
                    {
                        let hl_clr =
                            if state.focused == entity {
                                UI_TRK_CURSOR_BG_SEL_CLR
                            } else { // if state.hovered == entity {
                                UI_TRK_CURSOR_BG_HOV_CLR
                            };

                        if (ir, ic) == self.cursor {
                            p.rect_fill(
                                hl_clr,
                                pos.x + x,
                                pos.y + y,
                                UI_TRK_COL_WIDTH,
                                UI_TRK_ROW_HEIGHT);

                        } else {
                            if self.enter_mode != EnterMode::None {
                                p.path_stroke(
                                    1.0,
                                    hl_clr,
                                    &mut [
                                        (pos.x + x + 1.5,              pos.y + y + UI_TRK_ROW_HEIGHT - 0.5),
                                        (pos.x + x + UI_TRK_COL_WIDTH - 0.5, pos.y + y + UI_TRK_ROW_HEIGHT - 0.5),
                                        (pos.x + x + UI_TRK_COL_WIDTH - 0.5, pos.y + y + 0.5),
                                        (pos.x + x + 1.5,              pos.y + y + 0.5),
                                    ].iter().copied(),
                                    true);
                            }
                        }

                        UI_TRK_TEXT_CLR
                    } else {
                        UI_TRK_TEXT_CLR
                    };

                let cell_value = pat.get_cell_value(ir, ic);
                if let Some(s) = pat.get_cell(ir, ic) {

                    if is_note_col {
                        p.label_mono(
                            UI_TRK_FONT_SIZE,
                            0,
                            txt_clr,
                            pos.x + x,
                            pos.y + y,
                            UI_TRK_COL_WIDTH,
                            UI_TRK_ROW_HEIGHT,
                            value2note_name(cell_value)
                                .unwrap_or(s));
                    } else {
                        p.label_mono(
                            UI_TRK_FONT_SIZE,
                            0,
                            txt_clr,
                            pos.x + x,
                            pos.y + y,
                            UI_TRK_COL_WIDTH,
                            UI_TRK_ROW_HEIGHT,
                            s);
                    }
                } else {
                    p.label_mono(
                        UI_TRK_FONT_SIZE,
                        0,
                        txt_clr,
                        pos.x + x,
                        pos.y + y,
                        UI_TRK_COL_WIDTH,
                        UI_TRK_ROW_HEIGHT,
                        "---");
                }
            }
        }

        for ic in 0..self.columns {
            let x = (ic + 1) as f32 * UI_TRK_COL_WIDTH;

            p.path_stroke(
                1.0,
                UI_TRK_COL_DIV_CLR,
                &mut [
                    (pos.x + x + 0.5, pos.y),
                    (pos.x + x + 0.5, pos.y + pos.h),
                ].iter().copied(),
                false);
        }

        p.reset_clip_region();
    }
}

fn value2note_name(val: u16) -> Option<&'static str> {
    if !(21..=127).contains(&val) {
        return None;
    }

    Some(match val {
        21 => "A-0",
        22 => "A#0",
        23 => "B-0",
        24 => "C-1", 25 => "C#1", 26 => "D-1", 27 => "D#1", 28 => "E-1", 29 => "F-1",
        30 => "F#1", 31 => "G-1", 32 => "G#1", 33 => "A-1", 34 => "A#1", 35 => "B-1",
        36 => "C-2", 37 => "C#2", 38 => "D-2", 39 => "D#2", 40 => "E-2", 41 => "F-2",
        42 => "F#2", 43 => "G-2", 44 => "G#2", 45 => "A-2", 46 => "A#2", 47 => "B-2",
        48 => "C-3", 49 => "C#3", 50 => "D-3", 51 => "D#3", 52 => "E-3", 53 => "F-3",
        54 => "F#3", 55 => "G-3", 56 => "G#3", 57 => "A-3", 58 => "A#3", 59 => "B-3",
        60 => "C-4", 61 => "C#4", 62 => "D-4", 63 => "D#4", 64 => "E-4", 65 => "F-4",
        66 => "F#4", 67 => "G-4", 68 => "G#4", 69 => "A-4", 70 => "A#4", 71 => "B-4",
        72 => "C-5", 73 => "C#5", 74 => "D-5", 75 => "D#5", 76 => "E-5", 77 => "F-5",
        78 => "F#5", 79 => "G-5", 80 => "G#5", 81 => "A-5", 82 => "A#5", 83 => "B-5",
        84 => "C-6", 85 => "C#6", 86 => "D-6", 87 => "D#6", 88 => "E-6", 89 => "F-6",
        90 => "F#6", 91 => "G-6", 92 => "G#6", 93 => "A-6", 94 => "A#6", 95 => "B-6",
        96 => "C-7", 97 => "C#7", 98 => "D-7", 99 => "D#7", 100 => "E-7", 101 => "F-7",
        102 => "F#7", 103 => "G-7", 104 => "G#7", 105 => "A-7", 106 => "A#7", 107 => "B-7",
        108 => "C-8", 109 => "C#8", 110 => "D-8", 111 => "D#8", 112 => "E-8", 113 => "F-8",
        114 => "F#8", 115 => "G-8", 116 => "G#8", 117 => "A-8", 118 => "A#8", 119 => "B-8",
        120 => "C-9", 121 => "C#9", 122 => "D-9", 123 => "D#9", 124 => "E-9", 125 => "F-9",
        126 => "F#9", 127 => "G-9", 128 => "G#9", 129 => "A-9", 130 => "A#9", 131 => "B-9",
        _ => "???",
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EnterValue {
    None,
    One(u16),
    Two(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EnterMode {
    None,
    EnterValues(EnterValue),
    EditStep,
    Octave,
    ColType,
    Delete,
    Rows(EnterValue),
}

pub fn advance_cursor(
    cursor:   &mut (usize, usize),
    row_offs: i16,
    col_offs: i16,
    pat:      &mut dyn UIPatternModel)
{
    if row_offs >= 0 {
        let row_offs = row_offs as usize;
        if ((*cursor).0 + row_offs) < pat.rows() {
            (*cursor).0 += row_offs;
        }

    } else {
        let row_offs = row_offs.abs() as usize;
        if (*cursor).0 >= row_offs {
            (*cursor).0 -= row_offs;
        } else {
            (*cursor).0 = 0;
        }
    }

    if col_offs >= 0 {
        let col_offs = col_offs as usize;
        if ((*cursor).1 + col_offs) < pat.cols() {
            (*cursor).1 += col_offs;
        }

    } else {
        let col_offs = col_offs.abs() as usize;
        if (*cursor).1 >= col_offs {
            (*cursor).1 -= col_offs;
        }
    }
}

fn note_from_char(c: &str, octave: u16) -> Option<u16> {
    let octave = (octave + 1) * 12;

    match c {
        "z" => Some(octave),
        "s" => Some(octave + 1),
        "x" => Some(octave + 2),
        "d" => Some(octave + 3),
        "c" => Some(octave + 4),
        "v" => Some(octave + 5),
        "g" => Some(octave + 6),
        "b" => Some(octave + 7),
        "h" => Some(octave + 8),
        "n" => Some(octave + 9),
        "j" => Some(octave + 10),
        "m" => Some(octave + 11),

        "," => Some(octave + 12),
        "l" => Some(octave + 13),
        "." => Some(octave + 14),
        ";" => Some(octave + 15),
        // "/" => Some(octave + 16), // collides with the "/" bind for edit step

        "q" => Some(octave + 12),
        "2" => Some(octave + 13),
        "w" => Some(octave + 14),
        "3" => Some(octave + 15),
        "e" => Some(octave + 16),
        "r" => Some(octave + 17),
        "5" => Some(octave + 18),
        "t" => Some(octave + 19),
        "6" => Some(octave + 20),
        "y" => Some(octave + 21),
        "7" => Some(octave + 22),
        "u" => Some(octave + 23),

        "i" => Some(octave + 24),
        "9" => Some(octave + 25),
        "o" => Some(octave + 26),
        "0" => Some(octave + 27),
        "p" => Some(octave + 28),
        "[" => Some(octave + 29),
        "=" => Some(octave + 30),
        "]" => Some(octave + 31),
        _ => None,
    }
}

fn num_from_char(c: &str) -> Option<u16> {
    match c {
        "0" => Some(0),
        "1" => Some(1),
        "2" => Some(2),
        "3" => Some(3),
        "4" => Some(4),
        "5" => Some(5),
        "6" => Some(6),
        "7" => Some(7),
        "8" => Some(8),
        "9" => Some(9),
        "a" => Some(0xA),
        "b" => Some(0xB),
        "c" => Some(0xC),
        "d" => Some(0xD),
        "e" => Some(0xE),
        "f" => Some(0xF),
        _ => None,
    }
}
