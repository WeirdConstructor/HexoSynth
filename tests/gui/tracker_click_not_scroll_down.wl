!@import t = tests:test_lib;
!@import hx;

hx:query_state[];

!(first_line_x, first_line_y) = (hx:id_by_text "00").0.1;

hx:mouse_move
    first_line_x + 30
    first_line_y - 10;

t:mouse_click :left;

t:key :Enter;   # Pattern edit mode.
t:key :f :f :f; # Enter "fff" into the first cell.

hx:query_state[];

std:assert
    is_none[hx:id_by_text["255"]]
    "tracker scrolled down despite click on the header";

std:assert_eq
    hx:id_by_text["00"].0.0.1
    "DBGID_PATEDIT_ROW"
    "first tracker row still visible";

std:assert_eq
    (hx:id_by_text :fff).0.0.1
    "DBGID_PATEDIT_CELL";

!pat = hx:pattern_data_for_tracker 0;
std:assert_eq pat.get_cursor[]   $i(2, 2)  "cursor advanced";
std:assert_eq (pat.get_cell 0 0) "fff"     "first cell contains right data";

