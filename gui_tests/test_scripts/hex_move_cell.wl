!@import t = wlambda_lib:test_lib;
!@import hx;

hx:query_state[];

# Create a tracker and select it
hx:set_cell $i(0,0) ${
    node_id = "tseq" => 0,
    ports   = $["clock"]
};

t:move_to_hex $i(0, 0);
t:mouse_click :left;

# Press left mouse button and drag to 3,4, and release mouse button
hx:mouse_down :left;
t:move_to_hex $i(3, 4);
hx:mouse_up :left;

hx:query_state[];

!cell = hx:get_cell $i(3, 4);

std:assert_eq cell.node_id.0 "TSeq";
std:assert_eq cell.pos $i(3, 4);
