use hexotk::widgets::HexDir;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum CellDir {
    TR,
    BR,
    B,
    BL,
    TL,
    T,
    /// Center
    C
}

impl CellDir {
    pub fn from(edge: u8) -> Self {
        match edge {
            0 => CellDir::TR,
            1 => CellDir::BR,
            2 => CellDir::B,
            3 => CellDir::BL,
            4 => CellDir::TL,
            5 => CellDir::T,
            _ => CellDir::C,
        }
    }

    #[inline]
    pub fn is_output(&self) -> bool {
        let e = self.to_edge();
        e <= 2
    }

    #[inline]
    pub fn is_input(&self) -> bool {
        !self.is_output()
    }

    #[inline]
    pub fn to_edge(&self) -> u8 {
        *self as u8
    }

    pub fn to_menu_pos(&self) -> (i32, i32) {
        match self {
            // out 1 - TR
            CellDir::TR => (0, 1),
            // out 2 - BR
            CellDir::BR => (1, 1),
            // out 3 - B
            CellDir::B  => (0, 1),
            // in 3 - BL
            CellDir::BL => (-1, 1),
            // in 2 - TL
            CellDir::TL => (-1, 0),
            // in 1 - T
            CellDir::T  => (0, -1),
            _           => (0, 0),
        }
    }

    pub fn to_offs(&self) -> (i32, i32) {
        match self {
            // out 1 - TR
            CellDir::TR => (0, 1),
            // out 2 - BR
            CellDir::BR => (1, 1),
            // out 3 - B
            CellDir::B  => (0, 1),
            // in 3 - BL
            CellDir::BL => (-1, 1),
            // in 2 - TL
            CellDir::TL => (-1, 0),
            // in 1 - T
            CellDir::T  => (0, -1),
            _           => (0, 0),
        }
    }
}

impl From<HexDir> for CellDir {
    fn from(h: HexDir) -> Self {
        CellDir::from(h.to_edge())
    }
}

impl From<CellDir> for HexDir {
    fn from(c: CellDir) -> Self {
        HexDir::from(c.to_edge())
    }
}
