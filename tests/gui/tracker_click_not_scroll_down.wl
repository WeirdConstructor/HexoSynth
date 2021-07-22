!@import t = tests:test_lib;
!@import hx;

hx:mouse_move 1074 100;
t:mouse_click :left;

t:key :Enter;

hx:query_state[];

std:assert
    is_none[hx:id_by_text["255"]]
    "tracker scrolled down despite click on the header";

std:assert
    is_some[hx:id_by_text["00"]]
    "tracker still shows top row";
