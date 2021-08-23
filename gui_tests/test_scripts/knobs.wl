# Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
# This file is a part of HexoSynth. Released under GPL-3.0-or-later.
# See README.md and COPYING for details.

# Test the Knob UI

!@import t = wlambda_lib:test_lib;
!@import hx;

!tests = $[];

std:push tests "mouse_wheel_coarse" => {
    t:matrix_wait {
        hx:set_cell $i(2, 2) ${
            node_id = "ad" => 0,
            ports   = $[],
        };
    };

    t:click_on_hex $i(2, 2) :left;

    hx:query_state[];

    t:move_text_contains "3.00ms";

    hx:mouse_wheel :up;
    hx:mouse_wheel :up;

    hx:query_state[];
    !id = (hx:id_by_text "24.00ms").0;

    std:assert_eq id.0.0 $p(0,2);
    std:assert_eq id.0.1 "DBGID_KNOB_VALUE";

    hx:mouse_wheel :down;
    hx:mouse_wheel :down;
    hx:mouse_wheel :down;

    hx:query_state[];
    !id = (hx:id_by_text " 0.00ms").0;

    std:assert_eq id.0.0 $p(0,2);
    std:assert_eq id.0.1 "DBGID_KNOB_VALUE";
};

std:push tests "mouse_wheel_fine" => {
    t:matrix_wait {
        hx:set_cell $i(2, 2) ${
            node_id = "ad" => 0,
            ports   = $[],
        };
    };

    t:click_on_hex $i(2, 2) :left;

    hx:query_state[];

    t:move_text_contains "dcy";

    hx:mouse_wheel :up;
    hx:mouse_wheel :up;

    hx:query_state[];
    !id = (hx:id_by_text "12.10ms").0;

    std:assert_eq id.0.0 $p(0,3);
    std:assert_eq id.0.1 "DBGID_KNOB_VALUE";

    hx:mouse_wheel :down;
    hx:mouse_wheel :down;
    hx:mouse_wheel :down;

    hx:query_state[];
    !id = (hx:id_by_text " 9.00ms").0;

    std:assert_eq id.0.0 $p(0,3);
    std:assert_eq id.0.1 "DBGID_KNOB_VALUE";
};

tests
