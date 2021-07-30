!@wlambda;
!@import std;
!@import hx;
!@import t = test_lib;

!drag_hex_from_to = {|2<3| !(from, to, btn) = @;
    !btn = ? is_none[btn] :left btn;
    t:move_to_hex from;
    hx:mouse_down btn;
    t:move_to_hex to;
    hx:mouse_up btn;
};

!reset = {
    iter x 0 => 10 {
        iter y 0 => 10 {
            hx:set_cell $i(x, y) ${ node_id = "nop" => 0 };
        };
    };
};


!@export drag_hex_from_to = drag_hex_from_to;
!@export reset10x10 = reset;
