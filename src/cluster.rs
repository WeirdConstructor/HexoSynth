use std::collections::HashSet;
use hexodsp::{Matrix, Cell};

struct Cluster {
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

    pub fn ignore_pos(&mut self, pos: (usize, usize)) {
        self.ignore_pos.insert(pos);
    }

    pub fn intersects_with(&self, other: &Cluster) -> bool {
        for p in self.poses.iter() {
            if other.poses.contains(p) {
                return true;
            }
        }

        false
    }

//    pub fn add_cluster_input_tail(&mut self, m: &mut Matrix, pos: (usize, usize)) {
//    }
//
//    pub fn add_cluster_output_tail(&mut self, m: &mut Matrix, pos: (usize, usize)) {
//    }

    pub fn add_cluster_at(&mut self, m: &mut Matrix, pos: (usize, usize)) {
        let mut stack = vec![pos];
        self.poses.insert(pos);

        while let Some(pos) = stack.pop() {
            if self.ignore_pos.contains(&pos) {
                continue;
            }

            if let Some(cell) = m.get_copy(pos.0, pos.1) {
                if !cell.is_empty() {
                    // figure out adjacent cells
                    // for those positions that are not present in `poses`
                    // and are not present in ignore_pos!
                    // => push each adjacent connected cell
                    // => add entry in poses
                }
            }
        }
    }

    pub fn remove_cluster_cells_from(&mut self, m: &mut Matrix) {
    }
}
