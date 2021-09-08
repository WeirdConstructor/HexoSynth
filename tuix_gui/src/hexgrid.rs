// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::hexo_consts::*;
use crate::rect::*;
use crate::painter::FemtovgPainter;

use tuix::*;
use femtovg::FontId;

use std::rc::Rc;

macro_rules! hxclr {
    ($i: expr) => {
        (
            ($i >> 16 & 0xFF) as f32 / 255.0,
            ($i >> 8  & 0xFF) as f32 / 255.0,
            ($i       & 0xFF) as f32 / 255.0,
        )
    }
}

pub const HEX_CLRS : [(f32, f32, f32); 18] = [
    hxclr!(0x922f93), // 0
    hxclr!(0x862b37),
    hxclr!(0xb45745),
    hxclr!(0x835933),
    hxclr!(0xa69b64),
    hxclr!(0xbec8a6),
    hxclr!(0x346c38), // 6
    hxclr!(0x1fb349),
    hxclr!(0x4cdb80),
    hxclr!(0x59bca3),
    hxclr!(0x228f9d),
    hxclr!(0x03b5e7),
    hxclr!(0x3b5eca), // 12
    hxclr!(0x594fa1),
    hxclr!(0xc2b2eb),
    hxclr!(0xac70fa),
    hxclr!(0x9850a9),
    hxclr!(0xdc4fc1), // 17
];

pub fn hex_color_idx2clr(idx: u8) -> (f32, f32, f32) {
    HEX_CLRS[idx as usize % HEX_CLRS.len()]
}

pub const UI_GRID_TXT_CENTER_CLR    : (f32, f32, f32) = UI_PRIM_CLR;
pub const UI_GRID_TXT_CENTER_HL_CLR : (f32, f32, f32) = UI_HLIGHT_CLR;
pub const UI_GRID_TXT_CENTER_SL_CLR : (f32, f32, f32) = UI_SELECT_CLR;
pub const UI_GRID_TXT_EDGE_CLR      : (f32, f32, f32) = UI_PRIM_CLR;
pub const UI_GRID_CELL_BORDER_CLR   : (f32, f32, f32) = UI_ACCENT_CLR;
pub const UI_GRID_EMPTY_BORDER_CLR  : (f32, f32, f32) = UI_ACCENT_DARK_CLR;
pub const UI_GRID_HOVER_BORDER_CLR  : (f32, f32, f32) = UI_ACCENT_CLR;
pub const UI_GRID_DRAG_BORDER_CLR   : (f32, f32, f32) = UI_HLIGHT_CLR;
pub const UI_GRID_BG1_CLR           : (f32, f32, f32) = UI_ACCENT_BG1_CLR;
pub const UI_GRID_BG2_CLR           : (f32, f32, f32) = UI_ACCENT_BG2_CLR;
pub const UI_GRID_SIGNAL_OUT_CLR    : (f32, f32, f32) = UI_HLIGHT_CLR;
pub const UI_GRID_LED_CLR           : (f32, f32, f32) = UI_PRIM_CLR;
pub const UI_GRID_LED_R             : f32             = 5.0;


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum HexDir {
    TR,
    BR,
    B,
    BL,
    TL,
    T
}

impl HexDir {
    pub fn from(edge: u8) -> Self {
        match edge {
            0 => HexDir::TR,
            1 => HexDir::BR,
            2 => HexDir::B,
            3 => HexDir::BL,
            4 => HexDir::TL,
            5 => HexDir::T,
            _ => HexDir::TR,
        }
    }

    #[inline]
    pub fn is_right_half(&self) -> bool {
        let e = self.as_edge();
        e <= 2
    }

    #[inline]
    pub fn is_left_half(&self) -> bool {
        !self.is_right_half()
    }

    #[inline]
    pub fn as_edge(&self) -> u8 {
        *self as u8
    }
}

use hexodsp::CellDir;

impl From<HexDir> for CellDir {
    fn from(h: HexDir) -> Self {
        CellDir::from(h.as_edge())
    }
}

impl From<CellDir> for HexDir {
    fn from(c: CellDir) -> Self {
        HexDir::from(c.as_edge())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum HexEdge {
    NoArrow,
    Arrow,
    ArrowValue { value: (f32, f32) },
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum HexHLight {
    Normal,
    Plain,
    Accent,
    HLight,
    Select,
}

#[derive(Debug)]
pub struct HexCell<'a> {
    pub label:      &'a str,
    pub hlight:     HexHLight,
    pub rg_colors:  Option<(f32, f32)>,
}

pub trait HexGridModel {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn cell_visible(&self, x: usize, y: usize) -> bool;
    fn cell_empty(&self, x: usize, y: usize) -> bool;
    fn cell_color(&self, x: usize, y: usize) -> u8 { 0 }
    fn cell_label<'a>(&self, x: usize, y: usize, out: &'a mut [u8])
        -> Option<HexCell<'a>>; // (&'a str, HexCell, Option<(f32, f32)>)>;
    /// Edge: 0 top-right, 1 bottom-right, 2 bottom, 3 bottom-left, 4 top-left, 5 top
    fn cell_edge<'a>(&self, x: usize, y: usize, edge: HexDir, out: &'a mut [u8])
        -> Option<(&'a str, HexEdge)>;
    fn cell_click(&self, x: usize, y: usize, btn: MButton, modkey: bool);
    fn cell_hover(&self, _x: usize, _y: usize) { }
}

#[derive(Debug, Clone)]
pub struct HexGridOld {
    center_font_size: f32,
    edge_font_size:   f32,
    bg_color:         (f32, f32, f32),
    y_offs:           bool,
    transformable:    bool,
    cell_size:        f32,
}

impl HexGridOld {
    pub fn new(center_font_size: f32, edge_font_size: f32, cell_size: f32) -> Self {
        Self {
            center_font_size,
            edge_font_size,
            bg_color:   UI_GRID_BG1_CLR,
            y_offs:     false,
            transformable: true,
            cell_size,
        }
    }

    pub fn new_y_offs_pinned(center_font_size: f32, edge_font_size: f32, cell_size: f32) -> Self {
        Self {
            center_font_size,
            edge_font_size,
            bg_color:       UI_GRID_BG1_CLR,
            y_offs:         true,
            transformable:  false,
            cell_size,
        }
    }

    pub fn bg_color(mut self, clr: (f32, f32, f32)) -> Self {
        self.bg_color = clr;
        self
    }
}

#[derive(Clone)]
pub struct HexGridData {
    model:          Rc<dyn HexGridModel>,
    last_hover_pos: (usize, usize),
//    hex_trans:      HexGridTransform,
}

impl HexGridData {
    pub fn new(model: Rc<dyn HexGridModel>) -> Box<Self> {
        Box::new(Self {
            model,
            last_hover_pos: (0, 0),
            // hex_trans: HexGridTransform::new()
        })
    }
}

fn hex_size2wh(size: f32) -> (f32, f32) {
    (2.0_f32 * size, (3.0_f32).sqrt() * size)
}

fn hex_at_is_inside(x: f32, y: f32, w: f32, h: f32, pos: Rect) -> bool {
    let aabb = Rect {
        x: x - 0.5 * w,
        y: y - 0.5 * h,
        w,
        h,
    };

    pos.aabb_is_inside(aabb)
}

enum HexDecorPos {
    Center(f32, f32),
    Top(f32, f32),
    TopLeft(f32, f32),
    TopRight(f32, f32),
    Bottom(f32, f32),
    BotLeft(f32, f32),
    BotRight(f32, f32),
}

impl HexEdge {
    fn draw(&self, p: &mut FemtovgPainter, x: f32, y: f32, rot: f32) {
        match self {
            HexEdge::NoArrow => {},
            HexEdge::Arrow => {
                draw_arrow(p, UI_GRID_TXT_EDGE_CLR, x, y, 0.0, 0.0, 10.0, rot);
            },
            HexEdge::ArrowValue { value } => {
                draw_arrow(p, UI_GRID_SIGNAL_OUT_CLR, x, y, 0.0, 0.0, 10.0, rot);
                let clr = (
                    value.0,
                    value.1,
                    0.3,
                );
                draw_arrow(p, clr, x, y, 1.0, 0.0, 7.0, rot);
            },
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_arrow(p: &mut FemtovgPainter, clr: (f32, f32, f32), x: f32, y: f32, xo: f32, yo: f32, size: f32, rot: f32) {
    p.path_fill_rot(
        clr,
        rot,
        x, y,
        xo + 1.0, yo + 1.0,
        &mut ([
            (0.0_f32, -0.6_f32),
            (0.0,      0.6),
            (1.4,      0.0),
        ].iter().copied()
         .map(|p| ((size * p.0),
                   (size * p.1)))),
        true);
}

fn draw_hexagon<F: Fn(&mut FemtovgPainter, HexDecorPos, (f32, f32, f32))>(p: &mut FemtovgPainter,
    size: f32, line: f32, x: f32, y: f32, clr: (f32, f32, f32), decor_fun: F) {

    let (w, h) = hex_size2wh(size);

    let sz = (w, h, size);

    p.path_stroke(
        line,
        clr,
        &mut ([
            (x - 0.50 * w, y          ),
            (x - 0.25 * w, y - 0.5 * h),
            (x + 0.25 * w, y - 0.5 * h),
            (x + 0.50 * w, y          ),
            (x + 0.25 * w, y + 0.5 * h),
            (x - 0.25 * w, y + 0.5 * h),
        ].iter().copied().map(|p| (p.0.floor(), p.1.floor()))), true);

    decor_fun(p,
        HexDecorPos::Center(x.floor(), y.floor()), sz);

    decor_fun(p,
        HexDecorPos::Top(
            x.floor(),
            (y - 0.5 * h).floor(),
        ), sz);

    decor_fun(p,
        HexDecorPos::TopRight(
            (x + 0.75 * size).floor(),
            (y - 0.25 * h   ).floor(),
        ), sz);

    decor_fun(p,
        HexDecorPos::TopLeft(
            (x - 0.75 * size).floor(),
            (y - 0.25 * h   ).floor(),
        ), sz);

    decor_fun(p,
        HexDecorPos::Bottom(
            x.floor(),
            (y + 0.5 * h).floor(),
        ), sz);

    decor_fun(p,
        HexDecorPos::BotRight(
            (x + 0.75 * size).floor(),
            (y + 0.25 * h   ).floor(),
        ), sz);

    decor_fun(p,
        HexDecorPos::BotLeft(
            (x - 0.75 * size).floor(),
            (y + 0.25 * h   ).floor(),
        ), sz);
}

fn draw_led(p: &mut FemtovgPainter, x: f32, y: f32, led_value: (f32, f32)) {
    let r = UI_GRID_LED_R;
    /*
          ____
         /    \
        /      \
        |  *   |
        |  xy  |
        \      /
         \____/
    */
    let path = &[
        (x - r,                  y - (r * 0.5)),
        (x - (r * 0.5),          y - r),
        (x + (r * 0.5),          y - r),
        (x + r,                  y - (r * 0.5)),

        (x + r,                  y + (r * 0.5)),
        (x + (r * 0.5),          y + r),
        (x - (r * 0.5),          y + r),
        (x - r,                  y + (r * 0.5)),
    ];

    let led_clr_border = (
        UI_GRID_LED_CLR.0 * 0.3,
        UI_GRID_LED_CLR.1 * 0.3,
        UI_GRID_LED_CLR.2 * 0.3,
    );
    let led_clr = (
        led_value.0 as f32,
        led_value.1 as f32,
        0.3,
    );
    p.path_fill(led_clr, &mut path.iter().copied(), true);
    p.path_stroke(1.0, led_clr_border, &mut path.iter().copied(), true);
}

#[derive(Default)]
pub struct HexGrid {
    id:        usize,
    font:      Option<FontId>,
    font_mono: Option<FontId>,
    tile_size: f32,
    scale:     f32,
}

impl HexGrid {
    pub fn new(id: usize, tile_size: f32) -> Self {
        HexGrid {
            id,
            font:       None,
            font_mono:  None,
            scale:      1.0,
            tile_size,
        }
    }
}

impl Widget for HexGrid {
    type Ret  = Entity;
    type Data = ();

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity.set_position_type(state, PositionType::ParentDirected)
              .set_clip_widget(state, entity)
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(window_event) = event.message.downcast::<WindowEvent>() {
        let posx = state.data.get_posx(entity);
        let posy = state.data.get_posy(entity);
        let width = state.data.get_width(entity);
        let height = state.data.get_height(entity);
//            println!("WINEVENT[{}]: {:?} {:?}", self.id, window_event, width);
            match window_event {
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

        let x = bounds.x + 100.0;
        let y = bounds.y + 100.0;
        let (w, h) = hex_size2wh(60.0);

        p.path_stroke(
            3.0,
            (1.0, 0.0, 1.0),
            &mut ([
                (x - 0.50 * w, y          ),
                (x - 0.25 * w, y - 0.5 * h),
                (x + 0.25 * w, y - 0.5 * h),
                (x + 0.50 * w, y          ),
                (x + 0.25 * w, y + 0.5 * h),
                (x - 0.25 * w, y + 0.5 * h),
            ].iter().copied().map(|p| (p.0.floor(), p.1.floor()))), true);

        //---------------------------------------------------------------------

        let pos : Rect = bounds.into();
        let size = self.tile_size;

        let pad     = 10.0;
        let size_in = size - pad;
        let (w, h)  = hex_size2wh(size);

        p.rect_fill_r(UI_GRID_BG1_CLR, pos);
    }
}

