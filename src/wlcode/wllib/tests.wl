!@wlambda;
!@import std;
!@import ui;
!@import hx;

!inside_rect = {!(target, test) = @;
         test.x >= target.x
    &and test.y >= target.y
    &and test.x <= (target.x + target.2)
    &and test.y <= (target.y + target.3)
};

!with_first = {!(list, filt, do) = @;
    iter l list {
        if (filt l) {
            return ~ $o(do l);
        };
    };
    $o()
};

!do_click = {
    !pos = _1.pos + $f(1.0, 1.0);
    std:displayln ">>> click(LMB)@" pos;
    _.mouse_press_at pos :left;
    _.mouse_release_at pos :left;
};

!do_hover= {
    !pos = _1.pos + $f(1.0, 1.0);
    std:displayln ">>> hover@" pos;
    _.mouse_to pos;
};

!click_on_source_label = {!(td, source, label) = @;
    with_first
        td.list_labels[]
        { _.source == source &and _.label == label }
        { do_click _ td; }
};

!dump_labels = {!(td) = @;
    iter l td.list_labels[] {
        std:displayln l;
    };
};

!@export install = {
    !test = ui:test_script "knob_hover_help_desc";
#    test.add_step :matrix_setup {!(td, labels) = @;
#        !matrix = hx:get_main_matrix_handle[];
#        matrix.clear[];
#    };
#    test.add_step :sleep {|| std:thread:sleep :ms => 500 };
    test.add_step :click_cell {!(td, labels) = @;
        !res = $S(*:{source=cell_name, label=Amp}) labels;
        do_click td res.0;
        $t
    };
#    test.add_step :sleep {|| std:thread:sleep :ms => 500 };
    test.add_step :hover_knob {!(td, labels) = @;
        do_hover td
            ($S(*:{tag=knob, source=value, label=*060*}) labels)
            .0;
        $t
    };
#    test.add_step :sleep {|| std:thread:sleep :ms => 500 };
    test.add_step :check_desc {!(td, labels) = @;
#        dump_labels td;
        !doc = ($S(*:{label=*Amp\ gain*}) labels).0;
        std:assert doc;
        $t
    };
    ui:install_test test;
};
