!@import t = tests:test_lib;
!@import hx;

hx:query_state[];

!(tracker_x, tracker_y) = (hx:id_by_text "00").0.1;

hx:mouse_move
    tracker_x + 30
    tracker_y - 10;
t:mouse_click :left;

t:key :Enter;

t:key :f :f :f;

hx:query_state[];

std:assert
    is_none[hx:id_by_text["255"]]
    "tracker scrolled down despite click on the header";

std:assert
    is_some[hx:id_by_text["00"]]
    "tracker still shows top row";

std:assert_eq
    (hx:id_by_text :fff).0.0.1
    "DBGID_PATEDIT_CELL";
