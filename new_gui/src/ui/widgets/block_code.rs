// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::ui::*;

use tuix::*;
use femtovg::FontId;

use std::rc::Rc;
use std::cell::RefCell;

use std::collections::HashMap;

pub trait BlockModel {
    fn rows(&self) -> usize;
    fn contains(&self, idx: usize) -> Option<usize>;
    fn expanded(&self) -> bool;
    fn label(&self, buf: &mut [u8]) -> usize;
    fn has_input(&self, idx: usize) -> bool;
    fn has_output(&self, idx: usize) -> bool;
    fn input_label(&self, idx: usize, buf: &mut [u8]) -> usize;
    fn output_label(&self, idx: usize, buf: &mut [u8]) -> usize;
}

pub trait BlockCodeModel {
    fn area_size(&self, id: usize) -> (usize, usize);
    fn block_at(&self, id: usize, x: usize, y: usize) -> Option<&dyn BlockModel>;
    fn origin_at(&self, id: usize, x: usize, y: usize) -> Option<(usize, usize)>;
}

#[derive(Debug, Clone)]
pub struct VisBlock {
    rows:     usize,
    contains: (Option<usize>, Option<usize>),
    expanded: bool,
    typ:      String,
    lbl:      String,
    inputs:   Vec<Option<String>>,
    outputs:  Vec<Option<String>>,
}

impl BlockModel for VisBlock {
    fn rows(&self) -> usize { self.rows }
    fn contains(&self, idx: usize) -> Option<usize> {
        if idx == 0 { self.contains.0 }
        else { self.contains.1 }
    }
    fn expanded(&self) -> bool { true }
    fn label(&self, buf: &mut [u8]) -> usize {
        use std::io::Write;
        let mut bw = std::io::BufWriter::new(buf);
        match write!(bw, "{}", self.lbl) {
            Ok(_) => bw.buffer().len(),
            _ => 0,
        }
    }
    fn has_input(&self, idx: usize) -> bool {
        self.inputs.get(idx).map(|s| s.is_some()).unwrap_or(false)
    }
    fn has_output(&self, idx: usize) -> bool {
        self.outputs.get(idx).map(|s| s.is_some()).unwrap_or(false)
    }
    fn input_label(&self, idx: usize, buf: &mut [u8]) -> usize {
        use std::io::Write;
        if let Some(lbl_opt) = self.inputs.get(idx) {
            if let Some(lbl) = lbl_opt {
                let mut bw = std::io::BufWriter::new(buf);
                match write!(bw, "{}", lbl) {
                    Ok(_) => bw.buffer().len(),
                    _ => 0,
                }
            } else { 0 }
        } else { 0 }
    }
    fn output_label(&self, idx: usize, buf: &mut [u8]) -> usize {
        use std::io::Write;
        if let Some(lbl_opt) = self.outputs.get(idx) {
            if let Some(lbl) = lbl_opt {
                let mut bw = std::io::BufWriter::new(buf);
                match write!(bw, "{}", lbl) {
                    Ok(_) => bw.buffer().len(),
                    _ => 0,
                }
            } else { 0 }
        } else { 0 }
    }
}

#[derive(Debug, Clone)]
pub struct BlockDSPBlock {
    vis_idx: usize,
}

#[derive(Debug, Clone)]
pub struct BlockDSPArea {
    vis:        Vec<VisBlock>,
    blocks:     HashMap<(usize, usize), Rc<RefCell<BlockDSPBlock>>>,
    origin_map: HashMap<(usize, usize), (usize, usize)>,
    size:       (usize, usize),
}

impl BlockDSPArea {
    fn new(w: usize, h: usize) -> Self {
        Self {
            vis:        vec![],
            blocks:     HashMap::new(),
            origin_map: HashMap::new(),
            size:       (w, h),
        }
    }

    fn set_block_at(&mut self, x: usize, y: usize, block: VisBlock) {
        let vis_idx =
            if let Some(b) = self.blocks.get(&(x, y)) {
                self.vis[b.borrow().vis_idx] = block;
                b.borrow().vis_idx
            } else {
                self.vis.push(block);
                self.vis.len() - 1
            };

        self.blocks.insert((x, y), Rc::new(RefCell::new(BlockDSPBlock {
            vis_idx
        })));
        self.update_origin_map();
        self.update_size();
    }

    fn update_size(&mut self) {
        let mut min_w = 1;
        let mut min_h = 1;

        for ((ox, oy), _) in &self.origin_map {
            if min_w < (ox + 1) { min_w = ox + 1; }
            if min_h < (oy + 1) { min_h = oy + 1; }
        }

        self.size = (min_w, min_h);
    }

    fn update_origin_map(&mut self) {
        self.origin_map.clear();

        for ((ox, oy), block) in &self.blocks {
            let block = block.borrow();
            let vb = &self.vis[block.vis_idx];

            for r in 0..vb.rows {
                self.origin_map.insert((*ox, *oy + r), (*ox, *oy));
            }
        }
    }

    fn check_space_at(&self, x: usize, y: usize, rows: usize) -> bool {
        for i in 0..rows {
            let yo = y + i;

            if self.origin_map.get(&(x, yo)).is_some() {
                return false;
            }
        }

        true
    }

//    pub fn add_block(&mut self, 
}

#[derive(Debug, Clone, Default)]
pub struct BlockType {
    pub category:       String,
    pub name:           String,
    pub rows:           usize,
    pub inputs:         Vec<Option<String>>,
    pub outputs:        Vec<Option<String>>,
    pub area_count:     usize,
    pub user_input:     bool,
    pub description:    String,
}

impl BlockType {
    fn instanciate_vis_block(&self, user_input: Option<String>) -> VisBlock {
        VisBlock {
            rows:     self.rows,
            contains:
                match self.area_count {
                    0 => (None, None),
                    1 => (Some(1), None),
                    2 => (Some(1), Some(1)),
                    _ => (None, None),
                },
            expanded: true,
            typ:      self.name.clone(),
            lbl:
                if let Some(inp) = user_input { inp }
                else { self.name.clone() },
            inputs:   self.inputs.clone(),
            outputs:  self.outputs.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockCodeLanguage {
    types:  HashMap<String, BlockType>,
}

impl BlockCodeLanguage {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    pub fn define(&mut self, typ: BlockType) {
        self.types.insert(typ.name.clone(), typ);
    }

    pub fn get_type_list(&self) -> Vec<(String, String, bool)> {
        let mut out = vec![];
        for (_, typ) in &self.types {
            out.push((typ.category.clone(), typ.name.clone(), typ.user_input));
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockDSPError {
    UnknownArea(usize),
    UnknownLanguageType(String),
    NoSpaceAvailable(usize, usize, usize),
}

#[derive(Debug, Clone)]
pub struct BlockDSPCode {
    language:   Rc<RefCell<BlockCodeLanguage>>,
    areas:      Vec<BlockDSPArea>,
}

impl BlockDSPCode {
    pub fn new(lang: Rc<RefCell<BlockCodeLanguage>>) -> Self {
        Self {
            language: lang,
            areas:    vec![BlockDSPArea::new(16, 16)],
        }
    }

    pub fn instanciate_at(
        &mut self, id: usize, x: usize, y: usize,
        typ: &str, user_input: Option<String>
    ) -> Result<(), BlockDSPError>
    {
        let lang = self.language.borrow();

        println!("TYPES: {:?}", lang.types);

        if let Some(area) = self.areas.get_mut(id) {
            if let Some(typ) = lang.types.get(typ) {
                if !area.check_space_at(x, y, typ.rows) {
                    return Err(BlockDSPError::NoSpaceAvailable(x, y, typ.rows));
                }
            }
        } else {
            return Err(BlockDSPError::UnknownArea(id));
        }

        if let Some(typ) = lang.types.get(typ) {
            let mut vis_block = typ.instanciate_vis_block(user_input);

            if let Some(area_id) = &mut vis_block.contains.0 {
                self.areas.push(BlockDSPArea::new(1, 1));
                *area_id = self.areas.len() - 1;
            }

            if let Some(area_id) = &mut vis_block.contains.1 {
                self.areas.push(BlockDSPArea::new(1, 1));
                *area_id = self.areas.len() - 1;
            }

            if let Some(area) = self.areas.get_mut(id) {
                area.set_block_at(x, y, vis_block);
            }

            Ok(())
        } else {
            return Err(BlockDSPError::UnknownLanguageType(typ.to_string()));
        }
    }
}

impl BlockCodeModel for BlockDSPCode {
    fn area_size(&self, id: usize) -> (usize, usize) {
        self.areas.get(id).map(|a| a.size).unwrap_or((0, 0))
    }

    fn block_at(&self, id: usize, x: usize, y: usize) -> Option<&dyn BlockModel> {
        let area  = self.areas.get(id)?;
        let block = area.blocks.get(&(x, y))?;
        let idx   = block.borrow().vis_idx;
        Some(area.vis.get(idx)?)
    }

    fn origin_at(&self, id: usize, x: usize, y: usize)
        -> Option<(usize, usize)>
    {
        self.areas
            .get(id)
            .map(|a| a.origin_map.get(&(x, y)).copied())
            .flatten()
    }
}


pub struct DummyBlockCode {
    blocks: HashMap<(usize, usize, usize), VisBlock>,
}

impl DummyBlockCode {
    pub fn new() -> Self {
        let mut s = Self {
            blocks: HashMap::new(),
        };

        s.blocks.insert((0, 1, 2), VisBlock {
            rows: 3,
            contains: (None, None),
            expanded: false,
            lbl: "get: x".to_string(),
            typ: "get".to_string(),
            inputs:  vec![
                None,
                Some("a".to_string()),
                Some("i".to_string())],
            outputs: vec![None, None, Some(">".to_string())],
        });
        s.blocks.insert((0, 2, 4), VisBlock {
            rows: 1,
            contains: (None, None),
            expanded: false,
            lbl: "sin".to_string(),
            typ: "sin".to_string(),
            inputs:  vec![Some("".to_string())],
            outputs: vec![Some(">".to_string())],
        });
        s.blocks.insert((0, 2, 3), VisBlock {
            rows: 1,
            contains: (None, None),
            expanded: false,
            lbl: "1.2".to_string(),
            typ: "value".to_string(),
            inputs:  vec![None],
            outputs: vec![Some(">".to_string())],
        });
        s.blocks.insert((0, 3, 3), VisBlock {
            rows: 2,
            contains: (None, None),
            expanded: false,
            lbl: "+".to_string(),
            typ: "add".to_string(),
            inputs:  vec![
                Some("".to_string()),
                Some("".to_string())],
            outputs: vec![None, Some(">".to_string())],
        });

        s.blocks.insert((0, 4, 4), VisBlock {
            rows: 1,
            contains: (Some(1), Some(2)),
            expanded: true,
            lbl: "if".to_string(),
            typ: "if".to_string(),
            inputs:  vec![Some("c".to_string())],
            outputs: vec![Some(">".to_string())],
        });

        s.blocks.insert((1, 0, 0), VisBlock {
            rows: 1,
            contains: (None, None),
            expanded: true,
            lbl: "1.0".to_string(),
            typ: "value".to_string(),
            inputs:  vec![None],
            outputs: vec![Some(">".to_string())],
        });

        s.blocks.insert((2, 0, 0), VisBlock {
            rows: 1,
            contains: (None, None),
            expanded: true,
            lbl: "2.0".to_string(),
            typ: "value".to_string(),
            inputs:  vec![None],
            outputs: vec![Some(">".to_string())],
        });

        s
    }
}

/*

BlockDSPCode requirements/functionality:
- Hold VisBlock's for BlockCodeModel trait
- Support edits of the blocks
    - adding new blocks (from a LanguageModel)
    - removing blocks
    - moving single blocks
    - moving chains of blocks
    - splitting chains
    - growing/shrinking sub areas on adding/removing nodes
    - provide and set information about the environment
        - input variables (query from LanguageModel?)
        - output variables (query from LanguageModel?)
        - local variables (settable)
        - persistent variables (settable)
    - subroutines
        - adding new ones
        - removing
        - adding blocks inside their areas
        - removing blocks from their areas
        - adding parameter names
        - searching the return block
    - serialize the block code into an AST using an external
      builder pattern.
    - undo/redo management?!
*/

impl BlockCodeModel for DummyBlockCode {
    fn area_size(&self, id: usize) -> (usize, usize) {
        match id {
            1 => (1, 1),
            2 => (1, 1),
            _ => (16, 16),
        }
    }

    fn block_at(&self, id: usize, x: usize, y: usize) -> Option<&dyn BlockModel> {
        Some(self.blocks.get(&(id, x, y))?)
    }

    fn origin_at(&self, id: usize, x: usize, y: usize)
        -> Option<(usize, usize)>
    {
        for ((area_id, bx, by), b) in self.blocks.iter() {
            if *area_id == id {
                if x == *bx && y >= *by && y <= *by + (b.rows - 1) {
                    return Some((*bx, *by));
                }
            }
        }

        None
    }
}

#[derive(Clone)]
pub enum BlockCodeMessage {
    SetCode(Rc<RefCell<dyn BlockCodeModel>>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockPos {
    Block { id: usize, x: usize, y: usize, row: usize, col: usize },
    Cell  { id: usize, x: usize, y: usize },
}

pub struct BlockCode {
    font_size:      f32,
    font:           Option<FontId>,
    font_mono:      Option<FontId>,
    code:           Rc<RefCell<dyn BlockCodeModel>>,

    block_size:     f32,

    areas:          Vec<Vec<(usize, Rect)>>,
    hover:          Option<(usize, usize, usize, usize)>,

    m_down:         Option<BlockPos>,

    on_change:      Option<Box<dyn Fn(&mut Self, &mut State, Entity, (usize, usize))>>,
    on_expand:      Option<Box<dyn Fn(&mut Self, &mut State, Entity, usize)>>,
    on_click:       Option<Box<dyn Fn(&mut Self, &mut State, Entity, (usize, usize, usize), bool)>>,
    on_hover:       Option<Box<dyn Fn(&mut Self, &mut State, Entity, bool, usize)>>,
}

impl BlockCode {
    pub fn new() -> Self {
        Self {
            font_size:      14.0,
            font:           None,
            font_mono:      None,
            code:           Rc::new(RefCell::new(DummyBlockCode::new())),

            block_size:     30.0,

            areas:          vec![],
            hover:          None,
            m_down:         None,

            on_change:      None,
            on_expand:      None,
            on_click:     None,
            on_hover:       None,
        }
    }

    pub fn on_change<F>(mut self, on_change: F) -> Self
    where
        F: 'static + Fn(&mut Self, &mut State, Entity, (usize, usize)),
    {
        self.on_change = Some(Box::new(on_change));

        self
    }

    pub fn on_expand<F>(mut self, on_expand: F) -> Self
    where
        F: 'static + Fn(&mut Self, &mut State, Entity, usize),
    {
        self.on_expand = Some(Box::new(on_expand));

        self
    }

    pub fn on_click<F>(mut self, on_click: F) -> Self
    where
        F: 'static + Fn(&mut Self, &mut State, Entity, (usize, usize, usize), bool),
    {
        self.on_click = Some(Box::new(on_click));

        self
    }

    pub fn on_hover<F>(mut self, on_hover: F) -> Self
    where
        F: 'static + Fn(&mut Self, &mut State, Entity, bool, usize),
    {
        self.on_hover = Some(Box::new(on_hover));

        self
    }

    pub fn reset_areas(&mut self) {
        for a in self.areas.iter_mut() {
            a.clear();
        }
    }

    pub fn store_area_pos(&mut self, area_id: usize, level: usize, pos: Rect) {
        if level >= self.areas.len() {
            self.areas.resize_with(area_id + 1, || vec![]);
        }

        self.areas[level].push((area_id, pos));
    }

    pub fn draw_area(&mut self, p: &mut FemtovgPainter, area_id: usize, pos: Rect, level: usize) {
        p.clip_region(pos.x, pos.y, pos.w, pos.h);

        let block_h = self.block_size;
        let block_w = block_h * 2.0;

        let cols = (pos.w / block_w).ceil() as usize;
        let rows = (pos.h / block_h).ceil() as usize;

        p.rect_fill(UI_ACCENT_BG1_CLR, pos.x, pos.y, pos.w, pos.h);

        for row in 0..rows {
            for col in 0..cols {
                let x = col as f32 * block_w;
                let y = row as f32 * block_h;

                let marker_px = (block_h * 0.2).floor();
                draw_markers(
                    p, pos.x + x, pos.y + y,
                    block_w, block_h, marker_px);
            }
        }

        let mut next_areas = vec![];

        let mut lbl_buf : [u8; 20] = [0; 20];

        for row in 0..rows {
            for col in 0..cols {
                let x = col as f32 * block_w;
                let y = row as f32 * block_h;

                let mut hover_here =
                    if let Some(hover) = self.hover {
                        hover.0 == area_id && col == hover.1 && row == hover.2
                    } else {
                        false
                    };

                let mut hover_row = -1;
                let mut hover_col = -1;

                if let Some((area, x, y, subcol)) = self.hover {
                    if area == area_id {
                        if let Some((bx, by)) =
                            self.code.borrow().origin_at(area, x, y)
                        {
                            hover_row = (y - by) as i32;
                            hover_col = subcol as i32;
                            hover_here = bx == col && by == row;
                        }
                    }
                }

                let bg_color =
                    if hover_here { UI_ACCENT_CLR }
                    else { UI_ACCENT_BG2_CLR };
                let border_color =
                    if hover_here { UI_HLIGHT_CLR }
                    else { UI_PRIM_CLR };

                if let Some(block) =
                    self.code.borrow().block_at(area_id, col, row)
                {
                    let w = block_w;
                    let h = block.rows() as f32 * block_h;

                    p.rect_fill(bg_color, pos.x + x, pos.y + y, w, h);

                    p.rect_stroke(
                        2.0, border_color,
                        pos.x + x + 1.0,
                        pos.y + y + 1.0,
                        w - 2.0, h - 2.0);

                    let hole_px = (0.6 * block_h).ceil();

                    let len = block.label(&mut lbl_buf[..]);
                    let val_s = std::str::from_utf8(&lbl_buf[0..len]).unwrap();
                    p.label(
                        self.block_size * 0.5,
                        0, UI_PRIM_CLR,
                        pos.x + x, pos.y + y, w, h, val_s);

//                                    let fs =
//                                        calc_font_size_from_text(
//                                            p, name_lbl, fs, maxwidth);

                    for i in 0..block.rows() {
                        if block.has_input(i) {
                            let row = i as f32 * block_h;
                            p.rect_fill(
                                bg_color,
                                pos.x + x,
                                pos.y + y + row
                                + ((block_h - hole_px) * 0.5).floor(),
                                3.0,
                                hole_px);

                            let len = block.input_label(i, &mut lbl_buf[..]);
                            let val_s = std::str::from_utf8(&lbl_buf[0..len]).unwrap();
                            p.label(
                                self.block_size * 0.4,
                                -1,
                                UI_PRIM_CLR,
                                pos.x + x,
                                pos.y + row + y - 1.0,
                                (block_w * 0.5).floor(),
                                block_h,
                                val_s);

                            if hover_here
                               && hover_col == 0
                               && hover_row == (i as i32)
                            {
                                let sel_block_w = (block_w * 0.5 * 0.8).floor();
                                let sel_block_h = (block_h * 0.8).floor();

                                p.rect_stroke(
                                    4.0, UI_SELECT_CLR,
                                    (pos.x + x
                                     + ((block_w * 0.5 - sel_block_w) * 0.5)).floor(),
                                    (pos.y + row + y
                                     + ((block_h - sel_block_h) * 0.5)).floor(),
                                    sel_block_w,
                                    sel_block_h);
                            }
                        }

                        if block.has_output(i) {
                            let row = i as f32 * block_h;
                            p.rect_fill(
                                bg_color,
                                pos.x + x + w - 3.0,
                                pos.y + y + row
                                + ((block_h - hole_px) * 0.5).floor(),
                                3.0,
                                hole_px);

                            let len = block.output_label(i, &mut lbl_buf[..]);
                            let val_s = std::str::from_utf8(&lbl_buf[0..len]).unwrap();
                            p.label(
                                self.block_size * 0.3,
                                1,
                                UI_PRIM_CLR,
                                (pos.x + x + (block_w * 0.5)).floor(),
                                pos.y + row + y - 1.0,
                                (block_w * 0.5).floor(),
                                block_h,
                                val_s);

                            if hover_here
                               && hover_col == 1
                               && hover_row == (i as i32)
                            {
                                let sel_block_w = (block_w * 0.5 * 0.8).floor();
                                let sel_block_h = (block_h * 0.8).floor();

                                p.rect_stroke(
                                    4.0, UI_SELECT_CLR,
                                    (pos.x + x + (block_w * 0.5)
                                     + ((block_w * 0.5 - sel_block_w) * 0.5)).floor(),
                                    (pos.y + row + y
                                     + ((block_h - sel_block_h) * 0.5)).floor(),
                                    sel_block_w,
                                    sel_block_h);
                            }
                        }
                    }

                    if let Some(cont_id) = block.contains(0) {
                        let (area_w, area_h) =
                            self.code.borrow().area_size(cont_id);
                        let bpos = Rect {
                            x: pos.x + x, // + border
                            y: pos.y + y + h,
                            w: (area_w as f32 * block_w + block_w * 0.3).floor(),
                            h: (area_h as f32 * block_h + block_w * 0.3).floor(),
                        };

                        next_areas.push((cont_id, bpos, border_color, bg_color));

                        if let Some(cont_id) = block.contains(1) {
                            let (area_w, area_h) =
                                self.code.borrow().area_size(cont_id);
                            let bpos = Rect {
                                x: bpos.x,
                                y: bpos.y + bpos.h,
                                w: (area_w as f32 * block_w + block_w * 0.3).floor(),
                                h: (area_h as f32 * block_h + block_w * 0.3).floor(),
                            };

                            next_areas.push((cont_id, bpos, border_color, bg_color));
                        }
                    }

                } else if hover_here {
                    p.rect_stroke(
                        2.0, UI_HLIGHT_CLR,
                        pos.x + x + 1.0,
                        pos.y + y + 1.0,
                        block_w - 2.0, block_h - 2.0);
                }
            }
        }

        for cont_area in next_areas.into_iter() {
            let (cont_id, pos, border_color, bg_color) = cont_area;

            let (area_w, area_h) = self.code.borrow().area_size(cont_id);
            let apos = Rect {
                x: pos.x + 4.0, // + border
                y: pos.y + 4.0,
                w: pos.w,
                h: pos.h,
            };
            p.rect_fill(
                border_color,
                apos.x - 4.0, apos.y - 4.0, apos.w + 8.0, apos.h + 8.0);
            p.rect_fill(
                UI_ACCENT_BG1_CLR,
                apos.x - 2.0, apos.y - 2.0, apos.w + 4.0, apos.h + 4.0);
            p.rect_fill(
                bg_color,
                (pos.x + block_w * 0.25).floor(),
                pos.y - 2.0,
                (block_w * 0.5).floor(),
                8.0);

            self.store_area_pos(cont_id, level + 1, Rect {
                x: apos.x - 4.0,
                y: apos.y - 4.0,
                w: apos.w + 8.0,
                h: apos.h + 8.0,
            });
            self.draw_area(p, cont_id, apos, level + 1);
        }

        p.reset_clip_region();
    }

    fn find_area_at(&self, x: f32, y: f32) -> Option<(usize, usize, usize, usize)> {
        let block_h = self.block_size;
        let block_w = block_h * 2.0;

        for lvl in self.areas.iter().rev() {
            for a in lvl.iter() {
                let (id, pos) = *a;

                if pos.is_inside(x, y) {
                    let xo = x - pos.x;
                    let yo = y - pos.y;
                    let xi = (xo / block_w).floor() as usize;
                    let yi = (yo / block_h).floor() as usize;

                    let sub_col =
                        if (xo - xi as f32 * block_w) > (block_w * 0.5) {
                            1
                        } else {
                            0
                        };

                    return Some((a.0, xi, yi, sub_col));
                }
            }
        }

        None
    }

    fn find_pos_at(&self, x: f32, y: f32) -> Option<BlockPos> {
        if let Some((area, x, y, subcol)) = self.find_area_at(x, y) {
            if let Some((ox, oy)) =
                self.code.borrow().origin_at(area, x, y)
            {
                let row = y - oy;
                Some(BlockPos::Block { id: area, x, y, col: subcol, row })
            } else {
                Some(BlockPos::Cell { id: area, x, y })
            }
        } else {
            None
        }
    }
}

fn draw_markers(p: &mut FemtovgPainter, x: f32, y: f32, block_w: f32, block_h: f32, marker_px: f32) {
    p.path_stroke(
        1.0,
        UI_ACCENT_DARK_CLR,
        &mut ([
            (x,             y + marker_px),
            (x,             y),
            (x + marker_px, y),
        ].iter().copied()
         .map(|p| (p.0.floor() + 0.5, p.1.floor() + 0.5))), false);

    p.path_stroke(
        1.0,
        UI_ACCENT_DARK_CLR,
        &mut ([
            (block_w + x - marker_px, y),
            (block_w + x,             y),
            (block_w + x,             y + marker_px),
        ].iter().copied()
         .map(|p| (p.0.floor() - 0.5, p.1.floor() + 0.5))), false);

    p.path_stroke(
        1.0,
        UI_ACCENT_DARK_CLR,
        &mut ([
            (x,             block_h + y - marker_px),
            (x,             block_h + y),
            (x + marker_px, block_h + y),
        ].iter().copied()
         .map(|p| (p.0.floor() + 0.5, p.1.floor() - 0.5))), false);

    p.path_stroke(
        1.0,
        UI_ACCENT_DARK_CLR,
        &mut ([
            (block_w + x - marker_px, block_h + y),
            (block_w + x,             block_h + y),
            (block_w + x,             block_h + y - marker_px),
        ].iter().copied()
         .map(|p| (p.0.floor() - 0.5, p.1.floor() - 0.5))), false);
}

impl Widget for BlockCode {
    type Ret  = Entity;
    type Data = ();

    fn widget_name(&self) -> String {
        "block-code".to_string()
    }

    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity.set_clip_widget(state, entity)
              .set_element(state, "block_code")
    }

    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(grid_msg) = event.message.downcast::<BlockCodeMessage>() {
            match grid_msg {
                BlockCodeMessage::SetCode(code) => {
                    self.code = code.clone();
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                        .target(Entity::root()));
                },
            }
        }

        if let Some(window_event) = event.message.downcast::<WindowEvent>() {
            match window_event {
                WindowEvent::MouseDown(MouseButton::Left) => {
                    let (x, y) = (state.mouse.cursorx, state.mouse.cursory);
                    self.m_down = self.find_pos_at(x, y);

//                    self.
//                    self.drag = true;
//                    self.drag_src_idx = self.xy2pos(state, entity, x, y);
//
//                    if let Some((inputs, _)) = self.drag_src_idx {
//                        if inputs {
//                            if self.items.0.len() == 1 {
//                                self.drag_src_idx = Some((false, 0));
//                            }
//                        } else {
//                            if self.items.1.len() == 1 {
//                                self.drag_src_idx = Some((true, 0));
//                            }
//                        }
//                    }
//
                    state.capture(entity);
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                            .target(Entity::root()));
                },
                WindowEvent::MouseUp(MouseButton::Left) => {
                    let (x, y) = (state.mouse.cursorx, state.mouse.cursory);

                    let m_up = self.find_pos_at(x, y);
                    if m_up == self.m_down {
                        println!("CLICK: {:?}", m_up);
                    } else {
                        println!("DRAG: {:?} => {:?}", self.m_down, m_up);
                    }

                    self.m_down = None;

//                    if let Some((_drag, con)) = self.get_current_con() {
//                        self.con = Some(con);
//
//                        if let Some(callback) = self.on_change.take() {
//                            (callback)(self, state, entity, con);
//                            self.on_change = Some(callback);
//                        }
//                    } else {
//                        self.con = None;
//                    }
//
//                    self.drag = false;
//                    self.drag_src_idx = None;
//
                    state.release(entity);
                    state.insert_event(
                        Event::new(WindowEvent::Redraw)
                            .target(Entity::root()));
                },
                WindowEvent::MouseMove(x, y) => {
                    let old_hover = self.hover;
                    let mut found = false;

                    self.hover = self.find_area_at(*x, *y);

                    if old_hover != self.hover {
                        state.insert_event(
                            Event::new(WindowEvent::Redraw)
                                .target(Entity::root()));
                    }
                },
                _ => {},
            }
        }
    }

    fn on_draw(&mut self, state: &mut State, entity: Entity, canvas: &mut Canvas) {
        let bounds = state.data.get_bounds(entity);
        if self.font.is_none() {
            self.font      = Some(canvas.add_font_mem(std::include_bytes!("font.ttf")).expect("can load font"));
            self.font_mono = Some(canvas.add_font_mem(std::include_bytes!("font_mono.ttf")).expect("can load font"));
        }

        let p = &mut FemtovgPainter {
            canvas:     canvas,
            font:       self.font.unwrap(),
            font_mono:  self.font_mono.unwrap(),
        };

        let pos : Rect = bounds.into();
        let pos = pos.floor();

        self.reset_areas();

        self.store_area_pos(0, 0, pos);
        self.draw_area(p, 0, pos, 0);
    }
}
