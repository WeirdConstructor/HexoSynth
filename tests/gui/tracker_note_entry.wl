!@import t = tests:test_lib;
!@import hx;

hx:query_state[];

# Create a tracker and select it
hx:set_cell $i(0,0) ${
    node_id = "tseq" => 0,
    ports   = $["clock"]
};

t:move_to_hex $i(0, 0);
t:mouse_click :left;

hx:query_state[];
std:assert_eq (hx:id_by_text :TSeq).0.2 "TSeq";

# Click into tracker, for first cell:
!(first_line_x, first_line_y) = (hx:id_by_text "00").0.1;
hx:mouse_move
    first_line_x + 30
    first_line_y - 10;
t:mouse_click :left;

t:key :c :n;    # Make note column
t:key :Enter;   # Pattern edit mode.
t:key :y;       # Enter a note

hx:query_state[];

std:assert_eq
    (hx:id_by_text :A-5).0.0.1
    "DBGID_PATEDIT_CELL"
    "First cell displays note name";

!pat = hx:pattern_data_for_tracker 0;
std:assert_eq (pat.get_cell 0 0) "051"
    "first cell contains note data";
