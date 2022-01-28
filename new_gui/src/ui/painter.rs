// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoTK. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use super::Rect;

use femtovg::{
    renderer::OpenGl,
    Canvas,
    FontId,
    Color,
};

pub struct FemtovgPainter<'a> {
    pub canvas:     &'a mut Canvas<OpenGl>,
    pub font:       FontId,
    pub font_mono:  FontId,
}

fn color_paint(color: (f32, f32, f32)) -> femtovg::Paint {
    femtovg::Paint::color(
        Color::rgbf(
            color.0 as f32,
            color.1 as f32,
            color.2 as f32))
}

impl<'a> FemtovgPainter<'a> {
    #[allow(unused_variables)]
    fn label_with_font(
        &mut self, size: f32, align: i8, rot: f32, color: (f32, f32, f32),
        x: f32, y: f32, xoi: f32, yoi: f32, w: f32, h: f32,
        text: &str, font: FontId)
    {
        let mut paint = color_paint(color);
        paint.set_font(&[font]);
        paint.set_font_size(size as f32);
        paint.set_text_baseline(femtovg::Baseline::Middle);
        let x = x.round();

        let (x, y) =
            if rot > 0.0 {
                self.canvas.save();
                let x = x as f32;
                let y = y as f32;
                let wh = (w / 2.0) as f32;
                let hh = (h / 2.0) as f32;

                let rot = rot.to_radians() as f32;

                self.canvas.translate(x + wh, y + hh);
                self.canvas.rotate(rot);
                self.canvas.translate(xoi as f32, yoi as f32);

                (-wh, -hh)
            } else {
                (x, y)
            };

//        let mut p = femtovg::Path::new();
//        p.rect(x as f32, y as f32, w as f32, h as f32);
//        self.canvas.stroke_path(&mut p, paint);
        match align {
            -1 => {
                paint.set_text_align(femtovg::Align::Left);
                let _ =
                    self.canvas.fill_text(
                        x as f32,
                        (y + h / 2.0).round() as f32,
                        text, paint);
            },
            0  => {
                paint.set_text_align(femtovg::Align::Center);
                let _ =
                    self.canvas.fill_text(
                        (x + (w / 2.0)) as f32,
                        (y + h / 2.0).round() as f32,
                        text, paint);
            },
            _  => {
                paint.set_text_align(femtovg::Align::Right);
                let _ =
                    self.canvas.fill_text(
                        (x + w) as f32,
                        (y + h / 2.0).round() as f32,
                        text, paint);
            },
        }

//        let mut p = femtovg::Path::new();
//        let mut paint2 = color_paint((1.0, 1.0, 1.0));
//        p.rect((x - 1.0) as f32, (y - 1.0) as f32, 2.0, 2.0);
//        p.rect(((x + 0.5 * w) - 1.0) as f32, ((y + 0.5 * h) - 1.0) as f32, 2.0, 2.0);
//        self.canvas.stroke_path(&mut p, paint2);

        if rot > 0.0 {
//            self.canvas.translate(-(0.5 * w) as f32, 0.0);
            self.canvas.restore();
        }
    }
}

impl<'a> FemtovgPainter<'a> {
    pub fn clip_region(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.canvas.save();
        self.canvas.scissor(x as f32, y as f32, w as f32, h as f32);
    }

    pub fn reset_clip_region(&mut self) {
        self.canvas.reset_scissor();
        self.canvas.restore();
    }

    pub fn path_fill_rot(&mut self, color: (f32, f32, f32),
                     rot: f32, x: f32, y: f32, xo: f32, yo: f32,
                     segments: &mut dyn std::iter::Iterator<Item = (f32, f32)>,
                     closed: bool) {

        self.canvas.save();
        let rot = rot.to_radians();

        self.canvas.translate(x as f32, y as f32);
        self.canvas.rotate(rot as f32);
        self.canvas.translate(xo as f32, yo as f32);

        self.path_fill(color, segments, closed);

        self.canvas.restore();
    }

    #[allow(dead_code)]
    pub fn path_stroke_rot(&mut self, width: f32, color: (f32, f32, f32),
                       rot: f32, x: f32, y: f32, xo: f32, yo: f32,
                       segments: &mut dyn std::iter::Iterator<Item = (f32, f32)>,
                       closed: bool) {

        self.canvas.save();
        let rot = rot.to_radians();

        self.canvas.translate(x as f32, y as f32);
        self.canvas.rotate(rot as f32);
        self.canvas.translate(xo as f32, yo as f32);

        self.path_stroke(width, color, segments, closed);

        self.canvas.restore();
    }

    pub fn path_fill(&mut self, color: (f32, f32, f32), segments: &mut dyn std::iter::Iterator<Item = (f32, f32)>, closed: bool) {
        let mut p = femtovg::Path::new();
        let paint = color_paint(color);

        let mut first = true;
        for s in segments {
            if first {
                p.move_to(s.0 as f32, s.1 as f32);
                first = false;
            } else {
                p.line_to(s.0 as f32, s.1 as f32);
            }
        }

        if closed { p.close(); }

        self.canvas.fill_path(&mut p, paint);
    }

    pub fn path_stroke(&mut self, width: f32, color: (f32, f32, f32),
                   segments: &mut dyn std::iter::Iterator<Item = (f32, f32)>,
                   closed: bool)
    {
        let mut p = femtovg::Path::new();
        let mut paint = color_paint(color);
        paint.set_line_join(femtovg::LineJoin::Round);
        // paint.set_line_cap(femtovg::LineCap::Round);
        paint.set_line_width(width as f32);

        let mut first = true;
        for s in segments {
            if first {
                p.move_to(s.0 as f32, s.1 as f32);
                first = false;
            } else {
                p.line_to(s.0 as f32, s.1 as f32);
            }
        }

        if closed { p.close(); }

        self.canvas.stroke_path(&mut p, paint);
    }

    pub fn arc_stroke(&mut self, width: f32, color: (f32, f32, f32), radius: f32, from_rad: f32, to_rad: f32, x: f32, y: f32) {
        let mut p = femtovg::Path::new();
        let mut paint = color_paint(color);
        paint.set_line_width(width as f32);
        p.arc(x as f32, y as f32, radius as f32, from_rad as f32, to_rad as f32, femtovg::Solidity::Hole);
        self.canvas.stroke_path(&mut p, paint);
    }

    #[allow(dead_code)]
    pub fn rect_stroke_r(&mut self, width: f32, color: (f32, f32, f32), rect: Rect) {
        self.rect_stroke(width, color, rect.x, rect.y, rect.w, rect.h)
    }

    pub fn rect_fill_r(&mut self, color: (f32, f32, f32), rect: Rect) {
        self.rect_fill(color, rect.x, rect.y, rect.w, rect.h)
    }

    pub fn rect_fill(&mut self, color: (f32, f32, f32), x: f32, y: f32, w: f32, h: f32) {
        let mut pth = femtovg::Path::new();
        pth.rect(x as f32, y as f32, w as f32, h as f32);
        self.canvas.fill_path(&mut pth, color_paint(color));
    }

    pub fn rect_stroke(&mut self, width: f32, color: (f32, f32, f32), x: f32, y: f32, w: f32, h: f32) {
        let mut pth = femtovg::Path::new();
        pth.rect(x as f32, y as f32, w as f32, h as f32);
        let mut paint = color_paint(color);
        paint.set_line_width(width as f32);
        self.canvas.stroke_path(&mut pth, paint);
    }

    pub fn label(&mut self, size: f32, align: i8, color: (f32, f32, f32), x: f32, y: f32, w: f32, h: f32, text: &str) {
        self.label_with_font(size, align, 0.0, color, x, y, 0.0, 0.0, w, h, text, self.font);
    }

    pub fn label_rot(&mut self, size: f32, align: i8, rot: f32, color: (f32, f32, f32), x: f32, y: f32, xo: f32, yo: f32, w: f32, h: f32, text: &str) {
        self.label_with_font(size, align, rot, color, x, y, xo, yo, w, h, text, self.font);
    }

    pub fn label_mono(&mut self, size: f32, align: i8, color: (f32, f32, f32), x: f32, y: f32, w: f32, h: f32, text: &str) {
        self.label_with_font(size, align, 0.0, color, x, y, 0.0, 0.0, w, h, text, self.font_mono);
    }

    pub fn text_width(&mut self, size: f32, mono: bool, text: &str) -> f32 {
        let mut paint = color_paint((1.0, 0.0, 1.0));
        if mono {
            paint.set_font(&[self.font_mono]);
        } else {
            paint.set_font(&[self.font]);
        }
        paint.set_font_size(size);
        if let Ok(metr) = self.canvas.measure_text(0.0, 0.0, text, paint) {
            metr.width()
        } else {
            20.0
        }
    }

    pub fn font_height(&mut self, size: f32, mono: bool) -> f32 {
        let mut paint = color_paint((1.0, 0.0, 1.0));
        if mono {
            paint.set_font(&[self.font_mono]);
        } else {
            paint.set_font(&[self.font]);
        }
        paint.set_font_size(size);
        if let Ok(metr) = self.canvas.measure_font(paint) {
            metr.height()
        } else {
            UI_ELEM_TXT_H as f32
        }
    }

    pub fn translate(&mut self, x: f32, y: f32) { // , x2: f64, y2: f64, factor: f64) {
        self.canvas.save();
//        self.cur_scale = factor as f32;
//        let factor = self.cur_scale;
//        self.canvas.translate(x as f32, y as f32);
        self.canvas.translate(x, y);
//        self.canvas.scale(factor, factor);
//        self.canvas.translate(x2 as f32, y2 as f32);
//        self.canvas.translate(-x as f32 / factor, -y as f32 / factor);
    }

    pub fn restore(&mut self) {
        self.canvas.restore();
    }
}

pub fn calc_font_size_from_text(
    p: &mut FemtovgPainter,
    txt: &str,
    mut max_fs: f32,
    max_width: f32
) -> f32
{
    while p.text_width(max_fs, false, txt) > max_width {
        max_fs *= 0.9;
    }

    max_fs
}

pub const UI_ELEM_TXT_H     : f32 =  16.0;
