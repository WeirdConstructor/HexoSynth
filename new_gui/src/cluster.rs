// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoSynth. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use std::collections::HashSet;
use hexodsp::{Matrix, Cell, CellDir, NodeId};
use hexodsp::matrix::MatrixError;

#[derive(Clone)]
pub struct Cluster {
    cells:          Vec<Cell>,
    poses:          HashSet<(usize, usize)>,
    ignore_pos:     HashSet<(usize, usize)>,
}

impl Cluster {
    pub fn new() -> Self {
        Self {
            cells:      vec![],
            poses:      HashSet::new(),
            ignore_pos: HashSet::new(),
        }
    }

    pub fn for_poses<F: FnMut(&(usize, usize))>(&self, mut f: F) {
        for p in self.poses.iter() { f(p) }
    }

    pub fn for_cells<F: FnMut(&Cell)>(&self, mut f: F) {
        for c in self.cells.iter() { f(c) }
    }

    pub fn ignore_pos(&mut self, pos: (usize, usize)) {
        self.ignore_pos.insert(pos);
    }

    #[allow(dead_code)]
    pub fn intersects_with(&self, other: &Cluster) -> bool {
        for p in self.poses.iter() {
            if other.poses.contains(p) {
                return true;
            }
        }

        false
    }

    pub fn add_cluster_at(&mut self, m: &mut Matrix, pos: (usize, usize)) {
        let mut stack = vec![pos];

        while let Some(pos) = stack.pop() {
            if self.ignore_pos.contains(&pos) {
                continue;
            }
            if self.poses.contains(&pos) {
                continue;
            }

            if let Some(cell) = m.get_copy(pos.0, pos.1) {
                if !cell.is_empty() {
                    for edge in 0..6 {
                        let dir = CellDir::from(edge);
                        if let Some(new_pos) =
                            cell.is_port_dir_connected(m, dir)
                        {
                            stack.push(new_pos);
                        }
                    }

                    self.cells.push(cell);
                    self.poses.insert(pos);
                }
            }
        }
    }

    /// Removes cluster cells from the matrix. You must wrap this
    /// with `m.change_matrix(...)`!!!
    pub fn remove_cells(&mut self, m: &mut Matrix) {
        for pos in &self.poses {
            m.place(pos.0, pos.1, Cell::empty(NodeId::Nop));
        }
    }

    /// Adds cluster cells from the matrix. You must wrap this
    /// with `m.change_matrix(...)`!!!
    pub fn place(&self, m: &mut Matrix) -> Result<(), MatrixError> {
        m.place_multiple(&self.cells)
    }

    pub fn move_cluster_cells_dir_path(&mut self, path: &[CellDir]) -> Result<(), MatrixError> {
        let mut cells = self.cells.clone();

        for dir in path {
            for cell in &mut cells {
                if !cell.offs_dir(*dir) {
                    return Err(MatrixError::PosOutOfRange);
                }
            }
        }

        self.poses.clear();
        for c in &cells {
            self.poses.insert(c.pos());
        }
        self.cells = cells;

        Ok(())
    }
}
