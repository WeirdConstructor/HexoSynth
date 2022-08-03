// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::constants::*;
use super::*;

pub trait GraphMinMaxSource {
    fn read(&mut self, dst: &mut [(f64, f64)]);
    fn fmt_val(&mut self, buf: &mut [u8]) -> usize;
}

#[derive(Debug)]
pub struct GraphMinMax {
    width:      f64,
    height:     f64,
}

pub struct GraphMinMaxData {
    source:               Box<dyn GraphMinMaxSource>,
    buf:                  Vec<(f64, f64)>,
    minmax_buf:           Vec<(f64, f64)>,
    minmax_sample_count:  usize,
    txt_wd:               WidgetData,
    font_size:            f64,
    lbl_buf:              [u8; 50],
}

#[allow(clippy::new_ret_no_self)]
impl GraphMinMaxData {
    pub fn new(
        font_size: f64,
        txt_src: Rc<TextSourceRef>,
        minmax_sample_count: usize,
        source: Box<dyn GraphMinMaxSource>
    ) -> Box<dyn std::any::Any>
    {
        let mut buf = vec![];
        buf.resize(2 * minmax_sample_count, (0.0, 0.0));

        let mut minmax_buf = vec![];
        minmax_buf.resize(minmax_sample_count, (0.0, 0.0));

        let wt_text = Rc::new(Text::new_no_padding(font_size));

        let txt_wd =
            WidgetData::new(
                  wt_text, AtomId::new(999, 999),
                  UIPos::center(12, 12),
                  TextData::new(txt_src));

        Box::new(Self {
            source,
            buf,
            minmax_buf,
            minmax_sample_count,
            txt_wd,
            font_size,
            lbl_buf: [0; 50],
        })
    }
}

impl GraphMinMax {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height
        }
    }
}

const WAVEFORM_SCALE_FACTOR : f64 = 0.9;

impl WidgetType for GraphMinMax {
    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
        let out_pos = Rect::from(pos.x, pos.y, self.width, self.height);
        let in_pos  = pos.shrink(UI_GRPH_BORDER, UI_GRPH_BORDER);

        p.rect_fill(UI_GRPH_BORDER_CLR, out_pos.x, out_pos.y, out_pos.w, out_pos.h);
        p.rect_fill(UI_GRPH_BG,         in_pos.x,  in_pos.y,  in_pos.w,  in_pos.h);

        data.with(|data: &mut GraphMinMaxData| {
            let txt_h = p.font_height(data.font_size as f32, true) as f64;
            let txt_h_b = txt_h * 1.5;
            let val_pos  = in_pos.resize(in_pos.w, txt_h);
            let grph_pos = in_pos.crop_bottom(txt_h_b).crop_top(txt_h);

            data.source.read(&mut data.minmax_buf[..]);

            let xd = 1.0 / (data.minmax_sample_count - 1) as f64;
            let mut x = 0.0;

            let mut last_minmax = (-1.0, 1.0);

            for i in 0..data.minmax_sample_count {
                let (min, max) = data.minmax_buf[i];

                let min = min.clamp(-1.0, 1.0) * WAVEFORM_SCALE_FACTOR;
                let max = max.clamp(-1.0, 1.0) * WAVEFORM_SCALE_FACTOR;
                let min = (min + 1.0) * 0.5;
                let max = (max + 1.0) * 0.5;

                // - 1.0 for preventing bleeding into the border.
                let gx = x * (grph_pos.w - 1.0) + 0.5;
                let gy1 = (1.0 - min) * grph_pos.h;
                let gy2 = (1.0 - max) * grph_pos.h;

                data.buf[i * 2] = (
                    (grph_pos.x + gx),
                    (grph_pos.y + gy1)
                );

                if (last_minmax.1 - 0.00001) <= max {
                    // (probably) Rising edge
                    data.buf[i * 2 + 1] = (
                        (grph_pos.x + gx + 0.5),
                        (grph_pos.y + gy2)
                    );
                } else {
                    // (probably) Falling edge
                    data.buf[i * 2 + 1] = (
                        (grph_pos.x + gx - 0.5),
                        (grph_pos.y + gy2)
                    );
                }

                last_minmax = (min, max);

                x += xd;
            }

            p.path_stroke(
                1.0,
                UI_GRPH_BORDER_CLR,
                &mut ([
                    (grph_pos.x             , grph_pos.y + grph_pos.h * 0.5),
                    (grph_pos.x + grph_pos.w, grph_pos.y + grph_pos.h * 0.5),
                ].iter().copied().map(|p| (p.0.floor(), p.1.floor() + 0.5))), false);

            p.path_stroke(
                0.9,
                UI_GRPH_LINE_CLR,
                &mut data.buf.iter().copied().map(|p| (p.0, p.1 + 0.5)),
                false);

            let len = data.source.fmt_val(&mut data.lbl_buf[..]);
            let val_s = std::str::from_utf8(&data.lbl_buf[0..len]).unwrap();
            p.label(data.font_size, 0, UI_GRPH_PHASE_CLR,
                val_pos.x, val_pos.y, val_pos.w, txt_h, val_s,
                DBGID_GRPH_VALUE);

            let txt_pos = Rect {
                x: grph_pos.x,
                y: val_pos.y + grph_pos.h + val_pos.h,
                w: grph_pos.w,
                h: txt_h_b,
            };
            p.rect_fill(UI_GRPH_BG, txt_pos.x, txt_pos.y, txt_pos.w, txt_pos.h);
            data.txt_wd.draw(ui, p, txt_pos.offs(0.0, 0.0));
        });
    }

    fn size(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _avail: (f64, f64)) -> (f64, f64) {
        (self.width, self.height)
    }

    fn event(&self, _ui: &mut dyn WidgetUI, _data: &mut WidgetData, _ev: &UIEvent) {
    }
}
