!@wlambda;
!@import std;
!@import hx;

!DIALOG_BTN_ID = $p(90001,99);
!HEX_MENU_ID   = $p(9999,5);
!HEX_MAP_ID    = $p(9999,2);

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

!wait_for_hex_menu = {
    hx:query_state[];
    !hex_menu = filter { _.id.0 == HEX_MENU_ID } hx:zones[];
    while is_none[hex_menu.0] {
        hx:query_state[];
        .hex_menu = filter { _.id.0 == HEX_MENU_ID } hx:zones[];
        std:thread:sleep :ms => 10;
    };
};

!hex_menu_pos = {
    wait_for_hex_menu[];

    !hex_menu = filter { _.id.0 == HEX_MENU_ID } hx:zones[];
    !hex_menu = hex_menu.0;
    std:assert is_some[hex_menu] "Finding the hex menu";
    hex_menu.pos
};

!set_hex_wh_from_hover = {
    hx:query_state[];
    !hex_map = filter { _.id.0 == HEX_MAP_ID } hx:zones[];
    !hex_map = hex_map.0;
    !tile_size = hex_map.zone.2;
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

!move_to_hex = {
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
!@export move_to_hex = move_to_hex;

!@export click_on_hex {|1<2| !(pos, btn) = @;
    !btn = ? is_none[btn] :left btn;
    move_to_hex pos;
    hx:mouse_down btn;
    hx:mouse_up btn;
};

!@export click_text_contains {!(text, btn) = @;
    !btn = ? is_none[btn] :left btn;

    !pos = (hx:id_by_text_contains text).0.1;
    !x = pos.x + pos.z * 0.5;
    !y = pos.y + pos.w * 0.5;

    hx:mouse_move $f(x, y);
    hx:mouse_down _1;
    hx:mouse_up _1;
};

!@export menu_click_text {|1<2| !(text, btn) = @;
    !btn = ? is_none[btn] :left btn;

    wait_for_hex_menu[];

    !menu_pos = hex_menu_pos[];

    !pos = (filter { _.0.0 == HEX_MENU_ID } ~ hx:id_by_text text).0.1;
    !x = (menu_pos.x + menu_pos.z * 0.5) + pos.x + pos.z * 0.5;
    !y = (menu_pos.y + menu_pos.w * 0.5) + pos.y + pos.w * 0.5;

    hx:mouse_move $f(x, y);
    hx:mouse_down btn;
    hx:mouse_up btn;
};

!@export matrix_wait {!(fun) = @;
    !gen1 = hx:matrix_generation[];
    fun[];
    !gen2 = hx:matrix_generation[];
    while gen1 == gen2 {
        std:thread:sleep :ms => 10;
        .gen2 = hx:matrix_generation[];
    };
};

!hex_offs = {!(x, y, dir) = @;
    !even = x % 2 == 0;

    $i(x, y)
    + (match dir
        :TR => $i(1, ? even -1 0)
        :BR => $i(1, ? even 0 1)
        :B  => $i(0, 1)
        :BL => $i(-1, ? even 0 1)
        :TL => $i(-1, ? even -1 0)
        :T  => $i(0, -1))
};

!@export get_all_adj = {!(pos) = @;
    !out = $[];
    iter dir $[:T, :B, :TR, :BR, :BL, :TL] {
        !pos = hex_offs pos.0 pos.1 dir;
        !cell = hx:get_cell pos;
        if cell.node_id.0 != "Nop" {
            std:push out ~ cell;
        };
    };
    out
};
