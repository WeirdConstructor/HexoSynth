    fn draw(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, p: &mut dyn Painter, pos: Rect) {
//        let size = self.cell_size;
//
//        let pad     = 10.0;
//        let size_in = size - pad;
//        let (w, h)  = hex_size2wh(size);

        let drag_source =
            if let Some(drag_az) = ui.drag_zone_for(data.id()) {
                if let ZoneType::HexFieldClick { pos, ..} = drag_az.zone_type {
                    Some(pos)
                } else {
                    None
                }
            } else {
                None
            };

        let marked =
            if let Some(az) = ui.hover_zone_for(data.id()) {
                if let ZoneType::HexFieldClick { pos, ..} = az.zone_type {
                    data.with(|data: &mut HexGridData| {
                        if data.last_hover_pos != pos {
                            data.last_hover_pos = pos;
                            data.model.cell_hover(pos.0, pos.1);
                        }
                    });

                    pos
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

        let id = data.id();

        data.with(|data: &mut HexGridData| {
//            p.rect_fill(
//                self.bg_color,
//                pos.x, pos.y,
//                pos.w, pos.h);

//            let nx = data.model.width();
//            let ny = data.model.height();

//////            if let Some(ht) = ui.get_hex_transform(id) {
//////                if self.transformable {
//////                    let scale = ht.scale().clamp(0.5, 2.0);
//////                    data.hex_trans = ht.set_scale(scale);
//////                }
//////            }

//////            ui.define_active_zone(
//////                ActiveZone::new_hex_field(
//////                    id,
//////                    pos,
//////                    self.y_offs,
//////                    data.hex_trans,
//////                    size));

            let (scroll_x, scroll_y) = (
                data.hex_trans.x_offs(),
                data.hex_trans.y_offs()
            );
            //d// println!("scroll_x={}, scroll_y={}", scroll_x, scroll_y);
            let scale = data.hex_trans.scale();

            p.clip_region(pos.x, pos.y, pos.w, pos.h);
            let (mv_x, mv_y) = (
                pos.w * 0.5 + scroll_x * scale,
                pos.h * 0.5 + scroll_y * scale
            );

            p.move_and_scale(pos.x + mv_x, pos.y + mv_y, 0.0, 0.0, scale);

            let pos = Rect {
                x: - pos.w * 0.5,
                y: - pos.h * 0.5,
                w: pos.w,
                h: pos.h,
            };

            for xi in 0..nx {
                let x = xi as f64;

                for yi in 0..ny {
                    let y =
                        if xi % 2 == 0 { yi as f64 - 0.5 }
                        else           { yi as f64 };

                    let xo = pos.x + x * 0.75 * w + size;
                    let yo = pos.y + (1.00 + y) * h;

                    let yo = if self.y_offs { yo - 0.5 * h } else { yo };

                    let spos = Rect {
                        x: 0.0,
                        y: 0.0,
                        w: pos.w,
                        h: pos.h,
                    };

                    // Assume the tiles are bigger than they are, so we don't miss:
                    let tile_size_check_factor = 0.1;
                    let w_check_pad = w * tile_size_check_factor;
                    let h_check_pad = w * tile_size_check_factor;
                    if !hex_at_is_inside(
                            xo * scale + mv_x - w_check_pad * scale,
                            yo * scale + mv_y - h_check_pad * scale,
                            (w + w_check_pad) * scale,
                            (h + h_check_pad) * scale,
                            spos)
                    {
                        continue;
                    }

                    if !data.model.cell_visible(xi, yi) {
                        continue;
                    }

                    let th  = p.font_height(self.center_font_size as f32, false) as f64;
                    let fs  = self.center_font_size;
                    let th2 = p.font_height(self.edge_font_size as f32, false) as f64;
                    let fs2 = self.edge_font_size;

                    let (line, clr) =
                        if marked.0 == xi && marked.1 == yi {
                            (5.0, UI_GRID_HOVER_BORDER_CLR)
                        } else  if Some((xi, yi)) == drag_source {
                            (3.0, UI_GRID_DRAG_BORDER_CLR)
                        } else if data.model.cell_empty(xi, yi) {
                            (3.0, UI_GRID_EMPTY_BORDER_CLR)
                        } else {
                            (3.0, hex_color_idx2clr(data.model.cell_color(xi, yi)))
                        };

                    // padded outer hex
                    draw_hexagon(p, size_in, line, xo, yo, clr, |p, pos, sz| {
                        let mut label_buf = [0; 20];

                        match pos {
                            HexDecorPos::Center(x, y) => {
                                if let Some(cell_vis) = data.model.cell_label(xi, yi, &mut label_buf) {
                                    let (s, hc, led) = (
                                        cell_vis.label,
                                        cell_vis.hlight,
                                        cell_vis.rg_colors
                                    );

                                    let (txt_clr, clr) =
                                        match hc {
                                            HexHLight::Normal => (UI_GRID_TXT_CENTER_CLR, clr),
                                            HexHLight::Plain  => (UI_GRID_TXT_CENTER_CLR, clr),
                                            HexHLight::Accent => (UI_GRID_TXT_CENTER_CLR, UI_GRID_TXT_CENTER_CLR),
                                            HexHLight::HLight => (UI_GRID_TXT_CENTER_HL_CLR, UI_GRID_TXT_CENTER_HL_CLR),
                                            HexHLight::Select => (UI_GRID_TXT_CENTER_SL_CLR, UI_GRID_TXT_CENTER_SL_CLR),
                                        };

                                    let fs =
                                        if hc == HexHLight::Plain { fs * 1.4 }
                                        else { fs };

                                    let num_fs = fs * 0.8;
                                    let y_inc = -1.0 + p.font_height(fs as f32, false) as f64;
                                    let mut lbl_it = s.split(' ');

                                    if let Some(name_lbl) = lbl_it.next() {
                                        let maxwidth =
                                            if hc == HexHLight::Plain {
                                                (size * 1.3) as f32
                                            } else { (size * 0.82) as f32 };

                                        let mut fs = fs;
                                        //d// println!("TEXT: {:8.3} => {} (@{})", p.text_width(fs as f32, false, name_lbl), name_lbl, size * scale);
                                        while p.text_width(fs as f32, false, name_lbl) > maxwidth {
                                            fs *= 0.9;
                                        }

                                        p.label(
                                            fs, 0, txt_clr,
                                            x - 0.5 * sz.0,
                                            y - 0.5 * th,
                                            sz.0, th, name_lbl,
                                            dbgid_pack(DBGID_HEX_TILE_NAME, xi as u16, yi as u16));
                                    }

                                    if let Some(num_lbl) = lbl_it.next() {
                                        p.label(
                                            num_fs, 0, txt_clr,
                                            x - 0.5 * sz.0,
                                            y - 0.5 * th + y_inc,
                                            sz.0, th, num_lbl,
                                            dbgid_pack(DBGID_HEX_TILE_NUM, xi as u16, yi as u16));
                                    }

                                    if let Some(led) = led {
                                        draw_led(p, x, y - th, led);
                                    }

                                    if hc != HexHLight::Plain {
                                        draw_hexagon(
                                            p, size * 0.5, line * 0.5, x, y, clr,
                                            |_p, _pos, _sz| ());
                                    }
                                }
                            },
                            HexDecorPos::Top(x, y) => {
                                if let Some((s, _)) = data.model.cell_edge(xi, yi, HexDir::T, &mut label_buf) {
                                    p.label(
                                        fs2, 0, UI_GRID_TXT_EDGE_CLR,
                                        x - 0.5 * sz.0,
                                        y - 1.0,
                                        sz.0, th, s,
                                        dbgid_pack(DBGID_HEX_TILE_T, xi as u16, yi as u16));
                                }
                            },
                            HexDecorPos::Bottom(x, y) => {
                                if let Some((s, et)) = data.model.cell_edge(xi, yi, HexDir::B, &mut label_buf) {
                                    p.label(
                                        fs2, 0, UI_GRID_TXT_EDGE_CLR,
                                        x - 0.5 * sz.0,
                                        y - th,
                                        sz.0, th, s,
                                        dbgid_pack(DBGID_HEX_TILE_B, xi as u16, yi as u16));

                                    et.draw(p, x, y, 90.0);
                                }
                            },
                            HexDecorPos::TopLeft(x, y) => {
                                if let Some((s, _)) = data.model.cell_edge(xi, yi, HexDir::TL, &mut label_buf) {
                                    p.label_rot(
                                        fs2, 0, 300.0, UI_GRID_TXT_EDGE_CLR,
                                        (x - 0.5 * sz.0).floor(),
                                        (y - 0.5 * th2).floor(),
                                        0.0,
                                        (0.5 * th2).floor() + 2.0,
                                        sz.0, th2, s,
                                        dbgid_pack(DBGID_HEX_TILE_TL, xi as u16, yi as u16));
                                }
                            },
                            HexDecorPos::TopRight(x, y) => {
                                if let Some((s, et)) = data.model.cell_edge(xi, yi, HexDir::TR, &mut label_buf) {
                                    p.label_rot(
                                        fs2, 0, 60.0, UI_GRID_TXT_EDGE_CLR,
                                        (x - 0.5 * sz.0).floor(),
                                        (y - 0.5 * th2).floor(),
                                        0.0,
                                        (0.5 * th2).floor() + 2.0,
                                        sz.0, th2, s,
                                        dbgid_pack(DBGID_HEX_TILE_TR, xi as u16, yi as u16));

                                    et.draw(p, x, y, -30.0);
                                }
                            },
                            HexDecorPos::BotLeft(x, y) => {
                                if let Some((s, _)) = data.model.cell_edge(xi, yi, HexDir::BL, &mut label_buf) {
                                    p.label_rot(
                                        fs2, 0, 60.0, UI_GRID_TXT_EDGE_CLR,
                                        (x - 0.5 * sz.0).floor(),
                                        (y - 0.5 * th2).floor(),
                                        0.0,
                                        -(0.5 * th2).floor() - 2.0,
                                        sz.0, th2, s,
                                        dbgid_pack(DBGID_HEX_TILE_BL, xi as u16, yi as u16));
                                }
                            },
                            HexDecorPos::BotRight(x, y) => {
                                if let Some((s, et)) = data.model.cell_edge(xi, yi, HexDir::BR, &mut label_buf) {
                                    p.label_rot(
                                        fs2, 0, 300.0, UI_GRID_TXT_EDGE_CLR,
                                        (x - 0.5 * sz.0).floor(),
                                        (y - 0.5 * th2).floor(),
                                        0.0,
                                        -(0.5 * th2).floor() - 2.0,
                                        sz.0, th2, s,
                                        dbgid_pack(DBGID_HEX_TILE_BR, xi as u16, yi as u16));

                                    et.draw(p, x, y, 30.0);
                                }
                            },
                        }
                    });
                }
            }

            p.reset_scale();
            p.reset_clip_region();
        });
    }

    fn event(&self, ui: &mut dyn WidgetUI, data: &mut WidgetData, ev: &UIEvent) {
        if let UIEvent::Click { id, button, .. } = ev {
            if let Some(az) = ui.hover_zone_for(data.id()) {
                if az.id == data.id() && *id == data.id() {
                    if let ZoneType::HexFieldClick { pos, .. } = az.zone_type {
                        data.with(|data: &mut HexGridData| {
                            data.model.cell_click(
                                pos.0, pos.1, *button,
                                ui.is_key_pressed(UIKey::Ctrl));
                        });
                    }
                }
            }
        }
    }
}
