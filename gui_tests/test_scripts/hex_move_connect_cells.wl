!@import t = wlambda_lib:test_lib;
!@import hx;
!@import h = wlambda_lib:hex_lib;

hx:query_state[];

# Test 1B:  Connect with node with just 1 input to one with just 1 output
# Test 1C: Connect one with no output with one with 1 input
# Test 2B:  Connect with node with just 1 output to one with just 1 input
# Test 3B:  Connect with node with just 1 input to one with multiple outputs
# Test 4B:  Connect with node with just 1 output to one with multiple inputs
# Test 5B:  Connect with node with multiple output to one with multiple inputs

# Test 1B:  Connect with node with just 1 input 
h:reset10x10[];

hx:set_cell $i(1, 0) ${ node_id = "fbrd" => 0 };
hx:set_cell $i(2, 1) ${ node_id = "fbwr" => 0 };

t:matrix_wait {
    h:drag_hex_from_to $i(2, 1) $i(1, 0) :left;
};

!cell_a = hx:get_cell $i(2, 1);
!cell_b = hx:get_cell $i(1, 0);
std:assert_eq cell_b.ports.4 "sig";
std:assert_eq cell_a.ports.1 "inp";

# Test 1C: Connect one with no output with one with 1 input
h:reset10x10[];

hx:set_cell $i(1, 0) ${ node_id = "fbwr" => 0 };
hx:set_cell $i(2, 1) ${ node_id = "fbrd" => 0 };

t:matrix_wait {
    h:drag_hex_from_to $i(2, 1) $i(1, 0) :left;
};

!cell_a = hx:get_cell $i(2, 1);
!cell_b = hx:get_cell $i(1, 0);
std:assert_eq cell_b.ports.4 $n;
std:assert_eq cell_a.ports.1 "atv";

# Test 2B:  Connect with node with just 1 output to one with just 1 input
h:reset10x10[];

hx:set_cell $i(1, 0) ${ node_id = "fbrd" => 0 };
hx:set_cell $i(2, 1) ${ node_id = "fbwr" => 0 };

t:matrix_wait {
    h:drag_hex_from_to $i(1, 0) $i(2, 1) :left;
};

!cell_a = hx:get_cell $i(2, 1);
!cell_b = hx:get_cell $i(1, 0);
std:assert_eq cell_b.ports.4 "sig";
std:assert_eq cell_a.ports.1 "inp";

# Test 3B:  Connect with node with just 1 input to one with multiple outputs
h:reset10x10[];

hx:set_cell $i(1, 0) ${ node_id = "ad" => 0 };
hx:set_cell $i(2, 1) ${ node_id = "fbrd" => 0 };

t:matrix_wait {
    h:drag_hex_from_to $i(1, 0) $i(2, 1) :left;
    t:menu_click_text "eoet" :left;
};

!cell_a = hx:get_cell $i(2, 1);
!cell_b = hx:get_cell $i(1, 0);
std:assert_eq cell_b.ports.4 "eoet";
std:assert_eq cell_a.ports.1 "atv";

# Test 4B:  Connect with node with just 1 output to one with multiple inputs
h:reset10x10[];

hx:set_cell $i(1, 0) ${ node_id = "sin" => 0 };
hx:set_cell $i(2, 1) ${ node_id = "sin" => 1 };

t:matrix_wait {
    h:drag_hex_from_to $i(1, 0) $i(2, 1) :left;
    t:menu_click_text "det" :left;
};

!cell_a = hx:get_cell $i(2, 1);
!cell_b = hx:get_cell $i(1, 0);
std:assert_eq cell_b.ports.4 "sig";
std:assert_eq cell_a.ports.1 "det";

# Test 5B:  Connect with node with multiple output to one with multiple inputs
h:reset10x10[];

hx:set_cell $i(1, 0) ${ node_id = "ad" => 0 };
hx:set_cell $i(2, 1) ${ node_id = "sin" => 0 };

t:matrix_wait {
    h:drag_hex_from_to $i(1, 0) $i(2, 1) :left;
    t:menu_click_text "eoet" :left;
    t:menu_click_text "det" :left;
};

!cell_a = hx:get_cell $i(2, 1);
!cell_b = hx:get_cell $i(1, 0);
std:assert_eq cell_b.ports.4 "eoet";
std:assert_eq cell_a.ports.1 "det";
