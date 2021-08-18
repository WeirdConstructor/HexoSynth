# Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
# This file is a part of HexoSynth. Released under GPL-3.0-or-later.
# See README.md and COPYING for details.

!@import t = wlambda_lib:test_lib;
!@import hx;
!@import h = wlambda_lib:hex_lib;

!tests = $[];

std:push tests "drag_empty_empty_default_pair" => {
    t:matrix_wait {
        h:drag_hex_from_to $i(2, 1) $i(1, 0) :left;
        t:menu_click_text "Mod"  :left;
        t:menu_click_text "Ad"   :left;
        t:menu_click_text "Mod"  :left;
        t:menu_click_text "TSeq" :left;
    };

    !(a, b) = $[
        hx:get_cell $i(2, 1),
        hx:get_cell $i(1, 0),
    ];
    std:assert_eq a.ports.1 "inp";
    std:assert_eq b.ports.4 "trk1";
};

std:push tests "drag_empty_empty_default_pair_rev" => {
    t:matrix_wait {
        h:drag_hex_from_to $i(1, 0) $i(2, 1) :left;
        t:menu_click_text "Mod"  :left;
        t:menu_click_text "TSeq" :left;
        t:menu_click_text "Mod"  :left;
        t:menu_click_text "Ad"   :left;
    };

    !(a, b) = $[
        hx:get_cell $i(2, 1),
        hx:get_cell $i(1, 0),
    ];
    std:assert_eq a.ports.1 "inp";
    std:assert_eq b.ports.4 "trk1";
};

std:push tests "drag_empty_empty_set_io" => {
    t:matrix_wait {
        h:drag_hex_from_to $i(1, 0) $i(2, 1) :right;
        t:menu_click_text "Mod"  :left;
        t:menu_click_text "TSeq" :left;
        t:menu_click_text "Mod"  :left;
        t:menu_click_text "Ad"   :left;
        t:menu_click_text "trk3" :left;
        t:menu_click_text "trig" :left;
    };

    !(a, b) = $[
        hx:get_cell $i(2, 1),
        hx:get_cell $i(1, 0),
    ];

    std:assert_eq a.ports.1 "trig";
    std:assert_eq b.ports.4 "trk3";
};

std:push tests "drag_empty_empty_set_io_rev" => {
    t:matrix_wait {
        h:drag_hex_from_to $i(2, 1) $i(1, 0) :right;
        t:menu_click_text "Mod"  :left;
        t:menu_click_text "Ad"   :left;
        t:menu_click_text "Mod"  :left;
        t:menu_click_text "TSeq" :left;
        t:menu_click_text "trig" :left;
        t:menu_click_text "trk3" :left;
    };

    !(a, b) = $[
        hx:get_cell $i(2, 1),
        hx:get_cell $i(1, 0),
    ];

    std:assert_eq a.ports.1 "trig";
    std:assert_eq b.ports.4 "trk3";
};

!setup_sin_sin_cluster = {!(a_pos, b_pos) = @;
    t:matrix_wait {
        hx:set_cell a_pos ${
            node_id = "sin" => 0,
            ports   = $[$n, $n, $n, $n, "sig"],
        };
    };

    t:matrix_wait {
        hx:set_cell b_pos ${
            node_id = "sin" => 1,
            ports   = $[$n, "det", $n, $n, $n, $n],
        };
    };

};

std:push tests "drag_cluster" => {
    setup_sin_sin_cluster $i(1, 1) $i(2, 2);

    t:matrix_wait {
        h:drag_hex_from_to $i(2, 2) $i(2, 1) :left;
    };

    !(a, b) = $[
        hx:get_cell $i(1, 0),
        hx:get_cell $i(2, 1),
    ];
    std:assert_eq a.node_id.0 "Sin";
    std:assert_eq b.node_id.0 "Sin";

    std:assert_eq b.ports.1 "det";
    std:assert_eq a.ports.4 "sig";

    hx:query_state[];
};

std:push tests "drag_cluster_err" => {
    setup_sin_sin_cluster $i(1, 1) $i(2, 2);

    h:drag_hex_from_to $i(2, 2) $i(2, 0) :left;

    hx:query_state[];

    !id = hx:id_by_text_contains "out of Range";
    std:assert_eq id.0.0.1 "DBGID_TEXT_HEADER";

    hx:query_state[];
    t:click_text_contains "Ok" :left;

    hx:query_state[];
    !id = hx:id_by_text_contains "out of Range";
    std:assert is_none[id];
};

std:push tests "drag_split_cluster" => {
    setup_sin_sin_cluster $i(1, 1) $i(2, 2);

    t:matrix_wait {
        h:drag_hex_from_to $i(1, 1) $i(2, 2) :right;
    };

    !(a, b) = $[
        hx:get_cell $i(1, 1),
        hx:get_cell $i(3, 2),
    ];
    std:assert_eq a.node_id.0 "Sin";
    std:assert_eq b.node_id.0 "Sin";

    std:assert_eq b.ports.1 "det";
    std:assert_eq a.ports.4 "sig";

    hx:query_state[];
};

std:push tests "drag_split_cluster_err" => {
    setup_sin_sin_cluster $i(0, 0) $i(1, 0);

    h:drag_hex_from_to $i(1, 0) $i(0, 0) :right;

    hx:query_state[];

    !id = hx:id_by_text_contains "out of Range";
    std:assert_eq id.0.0.1 "DBGID_TEXT_HEADER";

    hx:query_state[];
    t:click_text_contains "Ok" :left;

    hx:query_state[];
    !id = hx:id_by_text_contains "out of Range";
    std:assert is_none[id];
};

std:push tests "drag_empty_exist_adj_new" => {
    hx:set_cell $i(1, 0) ${ node_id = "sin" => 3 };

    t:matrix_wait {
        h:drag_hex_from_to $i(0, 0) $i(1, 0) :left;
        t:menu_click_text "Mod"  :left;
        t:menu_click_text "Ad"   :left;
    };

    !(a, b) = $[
        hx:get_cell $i(0, 0),
        hx:get_cell $i(1, 0),
    ];
    std:assert_eq a.node_id.0 "Ad";
    std:assert_eq b.node_id.0 "Sin";

    std:assert_eq b.ports.1 "freq";
    std:assert_eq a.ports.4 "sig";
};

std:push tests "drag_empty_exist_adj_new_join_out" => {
    hx:set_cell $i(1, 0) ${ node_id = "sin" => 3 };

    t:matrix_wait {
        h:drag_hex_from_to $i(0, 0) $i(1, 0) :left;
        t:menu_click_text "Mod"  :left;
        t:menu_click_text "Ad"   :left;
    };

    t:matrix_wait {
        h:drag_hex_from_to $i(0, 1) $i(1, 0) :left;
        t:menu_click_text "Mod"  :left;
        t:menu_click_text "Ad"   :left;
        t:menu_click_text "eoet" :left;
        t:menu_click_text "det"  :left;
    };

    !(a, b) = $[
        hx:get_cell $i(0, 1),
        hx:get_cell $i(1, 0),
    ];
    std:assert_eq a.node_id.0 "Ad";
    std:assert_eq b.node_id.0 "Sin";

    std:assert_eq a.ports.3 "eoet";
    std:assert_eq b.ports.1 "freq";
    std:assert_eq b.ports.2 "det";
};

std:push tests "drag_empty_exist_adj_new_io" => {
    hx:set_cell $i(1, 0) ${ node_id = "sin" => 3 };

    t:matrix_wait {
        h:drag_hex_from_to $i(0, 0) $i(1, 0) :right;
        t:menu_click_text "Mod"  :left;
        t:menu_click_text "Ad"   :left;
        t:menu_click_text "sig"  :left;
        t:menu_click_text "freq" :left;
    };

    !(a, b) = $[
        hx:get_cell $i(0, 0),
        hx:get_cell $i(1, 0),
    ];
    std:assert_eq a.node_id.0 "Ad";
    std:assert_eq b.node_id.0 "Sin";

    std:assert_eq b.ports.1 "freq";
    std:assert_eq a.ports.4 "sig";
};

std:push tests "drag_empty_exist_copy" => {
    hx:set_cell $i(1, 0) ${
        node_id = "sin" => 3,
        ports   = $["freq", "det", "det", "sig", "sig", "sig"],
    };

    t:matrix_wait {
        h:drag_hex_from_to $i(3, 3) $i(1, 0) :left;
    };

    !a = hx:get_cell $i(3, 3);

    std:assert_eq a.node_id.0 "Sin";
    std:assert_eq a.node_id.1 3;
    std:assert_str_eq a.ports $[$n,$n,$n,$n,$n,$n];
};

std:push tests "drag_exist_exist_copy" => {
    hx:set_cell $i(1, 0) ${
        node_id = "ad" => 0,
        ports   = $[],
    };

    hx:set_cell $i(3, 3) ${
        node_id = "tseq" => 0,
        ports   = $[],
    };

    # first check if above:
    t:matrix_wait {
        h:drag_hex_from_to $i(1, 0) $i(3, 3) :left;
    };

    !a = hx:get_cell $i(3, 2);
    std:assert_eq a.node_id.0 "Ad";
    std:assert_eq a.node_id.1 0;
    std:assert_str_eq a.ports $[$n,$n,$n,$n,$n,$n];

    # second check if one left
    t:matrix_wait {
        h:drag_hex_from_to $i(1, 0) $i(3, 3) :left;
    };

    !a = hx:get_cell $i(2, 3);
    std:assert_eq a.node_id.0 "Ad";
    std:assert_eq a.node_id.1 0;

    # second check if one more left
    t:matrix_wait {
        h:drag_hex_from_to $i(1, 0) $i(3, 3) :left;
    };

    !a = hx:get_cell $i(2, 4);
    std:assert_eq a.node_id.0 "Ad";
    std:assert_eq a.node_id.1 0;
};

std:push tests "drag_exist_exist_new" => {
    hx:set_cell $i(1, 0) ${
        node_id = "ad" => 0,
        ports   = $[],
    };

    hx:set_cell $i(3, 3) ${
        node_id = "tseq" => 0,
        ports   = $[],
    };

    # first check if above:
    t:matrix_wait {
        h:drag_hex_from_to $i(1, 0) $i(3, 3) :right;
    };

    !a = hx:get_cell $i(3, 2);
    std:assert_eq a.node_id.0 "Ad";
    std:assert_eq a.node_id.1 1;
    std:assert_str_eq a.ports $[$n,$n,$n,$n,$n,$n];

    # second check if one left
    t:matrix_wait {
        h:drag_hex_from_to $i(1, 0) $i(3, 3) :right;
    };

    !a = hx:get_cell $i(2, 3);
    std:assert_eq a.node_id.0 "Ad";
    std:assert_eq a.node_id.1 2;

    # second check if one more left
    t:matrix_wait {
        h:drag_hex_from_to $i(1, 0) $i(3, 3) :right;
    };

    !a = hx:get_cell $i(2, 4);
    std:assert_eq a.node_id.0 "Ad";
    std:assert_eq a.node_id.1 3;
};

tests
