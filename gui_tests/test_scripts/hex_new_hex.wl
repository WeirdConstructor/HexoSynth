!@import t = wlambda_lib:test_lib;
!@import hx;

hx:mouse_move $f(424, 45);
t:set_hex_wh_from_hover[];

t:move_mouse_hex_dir :B => 3;

# open hex menu
t:mouse_click :left;
t:set_hex_wh_from_hover[];
# open "osc" sub menu
t:move_mouse_hex_dir :T;
t:mouse_click :left;
# select "sin"
t:move_mouse_hex_dir :BR;
t:mouse_click :left;

!cell = hx:get_cell $i(0, 3);

std:assert_eq cell.node_id.0 "Sin";
std:assert_eq cell.node_id.1 0;
std:assert_eq cell.pos $i(0, 3);
std:assert_str_eq cell.ports $[$n,$n,$n,$n,$n,$n];
