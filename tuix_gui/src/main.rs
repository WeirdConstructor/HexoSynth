// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use tuix::*;

mod hexo_consts;
mod painter;
mod hexgrid;
mod rect;

use painter::FemtovgPainter;
use hexgrid::HexGrid;

fn main() {
    let mut app =
        Application::new(
            WindowDescription::new(),
            |state, window| {
                let row = Row::new().build(state, window, |builder| builder);

                let hex  = HexGrid::new(1, 64.0).build(state, row, |builder| builder);
                let hex2 = HexGrid::new(2, 72.0).build(state, row, |builder| builder);
            });
    app.run();
}
