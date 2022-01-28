// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[allow(dead_code)]
impl Rect {
    pub fn from_tpl(t: (f32, f32, f32, f32)) -> Self {
        Self { x: t.0, y: t.1, w: t.2, h: t.3 }
    }

    pub fn from(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn floor(&self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
            w: self.w.floor(),
            h: self.h.floor(),
        }
    }

    pub fn resize(&self, w: f32, h: f32) -> Self {
        Self {
            x: self.x,
            y: self.y,
            w,
            h,
        }
    }

    pub fn scale(&self, factor: f32) -> Self {
        Self {
            x: self.x,
            y: self.y,
            w: self.w * factor,
            h: self.h * factor,
        }
    }

    pub fn center(&self) -> Self {
        Self {
            x: self.x + self.w * 0.5,
            y: self.y + self.h * 0.5,
            w: 1.0,
            h: 1.0,
        }
    }

    pub fn crop_left(&self, delta: f32) -> Self {
        Self {
            x: self.x + delta,
            y: self.y,
            w: self.w - delta,
            h: self.h,
        }
    }

    pub fn crop_right(&self, delta: f32) -> Self {
        Self {
            x: self.x,
            y: self.y,
            w: self.w - delta,
            h: self.h,
        }
    }

    pub fn crop_bottom(&self, delta: f32) -> Self {
        Self {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h - delta,
        }
    }

    pub fn crop_top(&self, delta: f32) -> Self {
        Self {
            x: self.x,
            y: self.y + delta,
            w: self.w,
            h: self.h - delta,
        }
    }

    pub fn shrink(&self, delta_x: f32, delta_y: f32) -> Self {
        Self {
            x: self.x + delta_x,
            y: self.y + delta_y,
            w: self.w - 2.0 * delta_x,
            h: self.h - 2.0 * delta_y,
        }
    }

    pub fn grow(&self, delta_x: f32, delta_y: f32) -> Self {
        Self {
            x: self.x - delta_x,
            y: self.y - delta_y,
            w: self.w + 2.0 * delta_x,
            h: self.h + 2.0 * delta_y,
        }
    }

    pub fn offs(&self, x: f32, y: f32) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
            w: self.w,
            h: self.h,
        }
    }

    pub fn move_into(mut self, pos: &Rect) -> Self {
        if self.x < pos.x { self.x = pos.x; }
        if self.y < pos.y { self.y = pos.y; }

        if (self.x + self.w) > (pos.x + pos.w) {
            self.x = (pos.x + pos.w) - self.w;
        }

        if (self.y + self.h) > (pos.y + pos.h) {
            self.y = (pos.y + pos.h) - self.h;
        }

        self
    }

    pub fn aabb_is_inside(&self, aabb: Rect) -> bool {
        if self.is_inside(aabb.x,          aabb.y)          { return true; }
        if self.is_inside(aabb.x + aabb.w, aabb.y)          { return true; }
        if self.is_inside(aabb.x,          aabb.y + aabb.h) { return true; }
        if self.is_inside(aabb.x + aabb.w, aabb.y + aabb.h) { return true; }
        false
    }

    pub fn is_inside(&self, x: f32, y: f32) -> bool {
           x >= self.x && x <= (self.x + self.w)
        && y >= self.y && y <= (self.y + self.h)
    }
}

impl From<tuix::BoundingBox> for Rect {
    fn from(bb: tuix::BoundingBox) -> Self {
        Self {
            x: bb.x,
            y: bb.y,
            w: bb.w,
            h: bb.h,
        }
    }
}
