!@import t = wlambda_lib:test_lib;
!@import hx;

hx:query_state[];

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

# This is the test plan:
#
# - Tests for new cell connection
#   - Case New Node with Input
#     - In count 0 (no such module (yet 2021-07-29)!)
#     - 6: In count 1
#       - 6: Old out count 0
#       - Old out count 1
#       - Old already set
#       - Old select param
#     - 2: Use Default
#       - 2: Old use default
#     - 3: Select param
#       - 3: Old select param
#   - Case New Node with Output
#     - 5: Out count 0
#       - 5: Old in count 1
#     - Out count 1
#       - Old in count 0 (no such module (yet 2021-07-29)!)
#       - Old already set
#       - Old select
#     - 1: Use Default
#       - 1: Old use default
#     - 4: Select Param
#       - 4: Old select param

# Test 1: New Node with Default Output
reset[];
hx:set_cell $i(1, 0) ${ node_id = "tseq" => 0 };
drag_hex_from_to $i(0, 0) $i(1, 0);

t:menu_click_text "CV"   :left;
t:menu_click_text "TSeq" :left;

!tseq1 = hx:get_cell $i(0, 0);
!tseq0 = hx:get_cell $i(1, 0);
std:assert_eq tseq1.ports.4 "trk1";
std:assert_eq tseq0.ports.1 "clock";

# Test 2: New Node with Default Input
reset[];
hx:set_cell $i(1, 0) ${ node_id = "tseq" => 0 };
drag_hex_from_to $i(2, 1) $i(1, 0);

t:menu_click_text "CV"   :left;
t:menu_click_text "TSeq" :left;

!tseq1 = hx:get_cell $i(2, 1);
!tseq0 = hx:get_cell $i(1, 0);
std:assert_eq tseq1.ports.1 "clock";
std:assert_eq tseq0.ports.4 "trk1";

# Test 3: New Node with Select I/O
reset[];
hx:set_cell $i(1, 0) ${ node_id = "tseq" => 0 };
drag_hex_from_to $i(2, 1) $i(1, 0) :right;

t:menu_click_text "CV"    :left;
t:menu_click_text "TSeq"  :left;
t:menu_click_text "trig"  :left;
t:menu_click_text "trk2"  :left;

!tseq1 = hx:get_cell $i(2, 1);
!tseq0 = hx:get_cell $i(1, 0);
std:assert_eq tseq1.ports.1 "trig";
std:assert_eq tseq0.ports.4 "trk2";

# Test 4: New Node with Select I/O
reset[];
hx:set_cell $i(1, 0) ${ node_id = "tseq" => 0 };
drag_hex_from_to $i(0, 0) $i(1, 0) :right;

t:menu_click_text "CV"    :left;
t:menu_click_text "TSeq"  :left;
t:menu_click_text "trk3"  :left;
t:menu_click_text "clock" :left;

!tseq1 = hx:get_cell $i(0, 0);
!tseq0 = hx:get_cell $i(1, 0);
std:assert_eq tseq1.ports.4 "trk3";
std:assert_eq tseq0.ports.1 "clock";

# Test 5: New out count 0, old in count 1
reset[];
hx:set_cell $i(1, 0) ${ node_id = "fbrd" => 0 };
drag_hex_from_to $i(0, 0) $i(1, 0) :right;

t:menu_click_text "I/O"   :left;
t:menu_click_text "FbWr"  :left;

!fbwr = hx:get_cell $i(0, 0);
!fbrd = hx:get_cell $i(1, 0);
std:assert_str_eq fbwr.ports $[$n,$n,$n,$n,$n,$n];
std:assert_eq fbrd.ports.1 "atv";

# Test 6: New in count 1, old out count 0
reset[];
hx:set_cell $i(1, 0) ${ node_id = "fbwr" => 0 };
drag_hex_from_to $i(2, 1) $i(1, 0) :right;

t:menu_click_text "I/O"   :left;
t:menu_click_text "FbRd"  :left;

!fbrd = hx:get_cell $i(2, 1);
!fbwr = hx:get_cell $i(1, 0);
std:assert_str_eq fbwr.ports $[$n,$n,$n,$n,$n,$n];
std:assert_eq fbrd.ports.1 "atv";

# Test 7: New in count 1, old out count 1
reset[];
hx:set_cell $i(1, 0) ${ node_id = "sin" => 0 };
drag_hex_from_to $i(2, 1) $i(1, 0) :right;

t:menu_click_text "I/O"  :left;
t:menu_click_text "FbRd" :left;

!fbrd = hx:get_cell $i(2, 1);
!fbwr = hx:get_cell $i(1, 0);
std:assert_eq fbwr.ports.4 "sig";
std:assert_eq fbrd.ports.1 "atv";
