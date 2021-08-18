# Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
# This file is a part of HexoSynth. Released under GPL-3.0-or-later.
# See README.md and COPYING for details.

!@import t = wlambda_lib:test_lib;
!@import hx;
!@import h = wlambda_lib:hex_lib;

!tests = $[];

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

std:push tests "test_1_new_node_with_def_out" => {
    hx:set_cell $i(1, 0) ${ node_id = "tseq" => 0 };
    h:drag_hex_from_to $i(0, 0) $i(1, 0);

    t:matrix_wait {
        t:menu_click_text "Mod"  :left;
        t:menu_click_text "TSeq" :left;
    };

    !tseq1 = hx:get_cell $i(0, 0);
    !tseq0 = hx:get_cell $i(1, 0);
    std:assert_eq tseq1.ports.4 "trk1";
    std:assert_eq tseq0.ports.1 "clock";
};

std:push tests "test_2_new_node_with_def_inp" => {
    hx:set_cell $i(1, 0) ${ node_id = "tseq" => 0 };
    h:drag_hex_from_to $i(2, 1) $i(1, 0);

    t:matrix_wait {
        t:menu_click_text "Mod"  :left;
        t:menu_click_text "TSeq" :left;
    };

    !tseq1 = hx:get_cell $i(2, 1);
    !tseq0 = hx:get_cell $i(1, 0);
    std:assert_eq tseq1.ports.1 "clock";
    std:assert_eq tseq0.ports.4 "trk1";
};

std:push tests "test_3_new_node_with_select_io" => {
    hx:set_cell $i(1, 0) ${ node_id = "tseq" => 0 };
    h:drag_hex_from_to $i(2, 1) $i(1, 0) :right;

    t:matrix_wait {
        t:menu_click_text "Mod"   :left;
        t:menu_click_text "TSeq"  :left;
        t:menu_click_text "trig"  :left;
        t:menu_click_text "trk2"  :left;
    };

    !tseq1 = hx:get_cell $i(2, 1);
    !tseq0 = hx:get_cell $i(1, 0);
    std:assert_eq tseq1.ports.1 "trig";
    std:assert_eq tseq0.ports.4 "trk2";
};

std:push tests "test_4_new_node_with_select_io_old" => {
    hx:set_cell $i(1, 0) ${ node_id = "tseq" => 0 };
    h:drag_hex_from_to $i(0, 0) $i(1, 0) :right;

    t:matrix_wait {
        t:menu_click_text "Mod"   :left;
        t:menu_click_text "TSeq"  :left;
        t:menu_click_text "trk3"  :left;
        t:menu_click_text "clock" :left;
    };

    !tseq1 = hx:get_cell $i(0, 0);
    !tseq0 = hx:get_cell $i(1, 0);
    std:assert_eq tseq1.ports.4 "trk3";
    std:assert_eq tseq0.ports.1 "clock";
};

std:push tests "test_5_new_node_out_cnt_0_old_in_1" => {
    hx:set_cell $i(1, 0) ${ node_id = "fbrd" => 0 };
    h:drag_hex_from_to $i(0, 0) $i(1, 0) :right;

    t:matrix_wait {
        t:menu_click_text "I/O"   :left;
        t:menu_click_text "FbWr"  :left;
    };

    !fbwr = hx:get_cell $i(0, 0);
    !fbrd = hx:get_cell $i(1, 0);
    std:assert_str_eq fbwr.ports $[$n,$n,$n,$n,$n,$n];
    std:assert_eq fbrd.ports.1 "atv";
};

std:push tests "test_6_new_node_in_1_old_out_0" => {
    hx:set_cell $i(1, 0) ${ node_id = "fbwr" => 0 };
    h:drag_hex_from_to $i(2, 1) $i(1, 0) :right;

    t:matrix_wait {
        t:menu_click_text "I/O"   :left;
        t:menu_click_text "FbRd"  :left;
    };

    !fbrd = hx:get_cell $i(2, 1);
    !fbwr = hx:get_cell $i(1, 0);
    std:assert_str_eq fbwr.ports $[$n,$n,$n,$n,$n,$n];
    std:assert_eq fbrd.ports.1 "atv";
};

std:push tests "test_6_new_node_in_1_old_out_1" => {
    hx:set_cell $i(1, 0) ${ node_id = "sin" => 0 };
    h:drag_hex_from_to $i(2, 1) $i(1, 0) :right;

    t:matrix_wait {
        t:menu_click_text "I/O"  :left;
        t:menu_click_text "FbRd" :left;
    };

    !fbrd = hx:get_cell $i(2, 1);
    !fbwr = hx:get_cell $i(1, 0);
    std:assert_eq fbwr.ports.4 "sig";
    std:assert_eq fbrd.ports.1 "atv";
};

tests
