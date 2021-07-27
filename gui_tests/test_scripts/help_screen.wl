# Open/Close the Help screen

!@import t = wlambda_lib:test_lib;
!@import hx;

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
