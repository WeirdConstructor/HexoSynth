!@import t = wlambda_lib:test_lib;
!@import hx;
!@import h = wlambda_lib:hex_lib;

!tests = $[];

# Test 1B:  Connect with node with just 1 input to one with just 1 output
# Test 1C: Connect one with no output with one with 1 input
# Test 2B:  Connect with node with just 1 output to one with just 1 input
# Test 3B:  Connect with node with just 1 input to one with multiple outputs
# Test 4B:  Connect with node with just 1 output to one with multiple inputs
# Test 5B:  Connect with node with multiple output to one with multiple inputs

# Test 1B:  Connect with node with just 1 input 
std:push tests "test_1b_con_just_1_input" => {
    hx:set_cell $i(1, 0) ${ node_id = "fbrd" => 0 };
    hx:set_cell $i(2, 1) ${ node_id = "fbwr" => 0 };

    t:matrix_wait {
        h:drag_hex_from_to $i(2, 1) $i(1, 0) :left;
    };

    !cell_a = hx:get_cell $i(2, 1);
    !cell_b = hx:get_cell $i(1, 0);
    std:assert_eq cell_b.ports.4 "sig";
    std:assert_eq cell_a.ports.1 "inp";
};

std:push tests "test_1c_con_no_out_1_in" => {
    hx:set_cell $i(1, 0) ${ node_id = "fbwr" => 0 };
    hx:set_cell $i(2, 1) ${ node_id = "fbrd" => 0 };

    t:matrix_wait {
        h:drag_hex_from_to $i(2, 1) $i(1, 0) :left;
    };

    !cell_a = hx:get_cell $i(2, 1);
    !cell_b = hx:get_cell $i(1, 0);
    std:assert_eq cell_b.ports.4 $n;
    std:assert_eq cell_a.ports.1 "atv";
};

std:push tests "test_2b_con_1_out_1_in" => {
    hx:set_cell $i(1, 0) ${ node_id = "fbrd" => 0 };
    hx:set_cell $i(2, 1) ${ node_id = "fbwr" => 0 };

    t:matrix_wait {
        h:drag_hex_from_to $i(1, 0) $i(2, 1) :left;
    };

    !cell_a = hx:get_cell $i(2, 1);
    !cell_b = hx:get_cell $i(1, 0);
    std:assert_eq cell_b.ports.4 "sig";
    std:assert_eq cell_a.ports.1 "inp";
};

std:push tests "test_3b_con_1_in_mult_out" => {
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
};

std:push tests "test_4b_con_1_out_mult_in" => {
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
};

std:push tests "test_5b_con_mult_out_mult_in" => {
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
};

tests
