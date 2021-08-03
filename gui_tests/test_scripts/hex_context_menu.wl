!@import t = wlambda_lib:test_lib;
!@import hx;
!@import h = wlambda_lib:hex_lib;

!tests = $[];

std:push tests "hex_menu_adj_random" => {
    hx:set_cell $i(2, 2) ${
        node_id = "ad" => 0,
        ports   = $[],
    };

    t:matrix_wait {
        t:click_on_hex $i(2, 2) :right;
        t:menu_click_text "Rand>";
        t:menu_click_text "Rand";
    };

    !o = t:get_all_adj $i(2, 2);
    #d# std:displayln o;
    std:assert_eq len[o] 1;

    iter _ 0 => 5 {
        t:matrix_wait {
            t:click_on_hex $i(2, 2) :right;
            t:menu_click_text "Rand>";
            t:menu_click_text "Rand";
        };
    };

    !o = t:get_all_adj $i(2, 2);
    #d# std:displayln o;
    std:assert_eq len[o] 6;
};
