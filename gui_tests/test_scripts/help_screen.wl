# Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
# This file is a part of HexoSynth. Released under GPL-3.0-or-later.
# See README.md and COPYING for details.

# Open/Close the Help screen

!@import t = wlambda_lib:test_lib;
!@import hx;

!tests = $[];

std:push tests "open_close_f1_f1" => {
    t:key :F1;
    hx:query_state[];
    std:assert_eq
        hx:id_by_text_contains["Parameter Knobs"].0.0.1
        "DBGID_TEXT_HEADER";

    t:key :F1;
    hx:query_state[];

    std:assert_eq
        hx:id_by_text_contains["Parameter Knobs"]
        $none;
};

std:push tests "open_close_f1_esc" => {
    t:key :F1;
    hx:query_state[];
    std:assert_eq
        hx:id_by_text_contains["Parameter Knobs"].0.0.1
        "DBGID_TEXT_HEADER";

    t:key :Escape;
    hx:query_state[];

    std:assert_eq
        hx:id_by_text_contains["Parameter Knobs"]
        $none;
};

tests
