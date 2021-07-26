!@wlambda;
!@import std;
!@import hx;

!@export mouse_click = {
    hx:mouse_down _;
    hx:mouse_up _;
};

!@export key = {
    iter key @ {
        hx:key_down key;
        hx:key_up   key;
    };
};

!hex_wh = {
    !tile_size = _;
    $f(2.0 * tile_size, std:num:sqrt[3.0] * tile_size)
};

!HEX_WH = hex_wh[54.0];
!HEX_INIT_POS = $f(424, 45);

!set_hex_wh_from_hover = {
    hx:query_state[];
    !tile_size = hx:hover[].zone.2;
    .HEX_WH = hex_wh tile_size;
};

!@export set_hex_wh_from_hover = set_hex_wh_from_hover;

!hex_dir_pos = {!(wh, pos, dir) = @;
    !(dir, count) =
        if type[dir] == "pair" { dir } { $p(dir, 1.0) };
    match dir
        :T  => $f(pos.x, pos.y - count * wh.1)
        :TR => $f(pos.x + 0.75 * count * wh.0, pos.y - 0.5  * count * wh.1)
        :BR => $f(pos.x + 0.75 * count * wh.0, pos.y + 0.5  * count * wh.1)
        :B  => $f(pos.x, pos.y + count * wh.1)
        :TL => $f(pos.x - 0.75 * count * wh.0, pos.y - 0.5  * count * wh.1)
        :BL => $f(pos.x - 0.75 * count * wh.0, pos.y + 0.5  * count * wh.1)
        { pos };
};

!@export hex_wh = hex_wh;
!@export hex_dir_pos = hex_dir_pos;

!@export move_mouse_hex_dir = {
    hx:query_state[];
    !next_pos = hex_dir_pos HEX_WH hx:mouse_pos[] _;
    hx:mouse_move next_pos;
};

!@export move_to_hex {
    !pos = _;
    set_hex_wh_from_hover[];

    !x = HEX_INIT_POS.x + 0.75 * pos.x * HEX_WH.x;
    !y =
        if pos.x % 2 == 0 {
            HEX_INIT_POS.y + pos.y * HEX_WH.y;
        } {
            HEX_INIT_POS.y + (0.5 + pos.y) * HEX_WH.y;
        };
    !new_pos = $f(x, y);
    hx:mouse_move new_pos;
};
