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

!do_drag = {
    !pos = _1.pos + $f(1.0, 1.0);
    std:displayln ">>> pick(LMB)@" pos;
    _.mouse_press_at pos :left;
};

!do_drop = {
    !pos = _1.pos + $f(1.0, 1.0);
    std:displayln ">>> drop(LMB)@" pos;
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

!matrix_init = {!(pos, dir, chain) = @;
    !matrix = hx:get_main_matrix_handle[];
    matrix.clear[];
    matrix.place_chain pos dir chain;
    matrix.sync[];
};

!@export install = {
    !test = ui:test_script "knob_hover_help_desc";
    test.add_step :init {||
        matrix_init  $i(0,1) :TR ${chain=$[
            $[:sin, :sig],
            $[:inp, :amp, :sig],
            $[:ch1, :out, $n],
        ], params = $[
            $n,
            $[:gain => 0.06],
            $n
        ]};
    };
#    test.add_step :sleep {|| std:thread:sleep :ms => 1000 };
    test.add_step :click_cell {!(td, labels) = @;
        !res = $S(*:{source=cell_name, label=Amp}) labels;
        do_click td res.0;
    };
#    test.add_step :sleep {|| std:thread:sleep :ms => 1000 };
    test.add_step :hover_knob {!(td, labels) = @;
        do_hover td
            ($S(*:{tag=knob, source=value, label=*060*}) labels)
            .0;
    };
#    test.add_step :sleep {|| std:thread:sleep :ms => 1000 };
    test.add_step :check_desc {!(td, labels) = @;
#        dump_labels td;
        !doc = ($S(*:{label=*Amp\ gain*}) labels).0;
        std:assert doc;
    };
#    test.add_step :sleep {|| std:thread:sleep :ms => 10000 };
    ui:install_test test;


    .test = ui:test_script "param_panel_update_on_matrix_change";
    !amp_node_info = $n;
    test.add_step :init {||
        matrix_init  $i(0,1) :TR ${chain=$[
            $[:sin, :sig],
            $[:inp, :amp, :sig],
            $[:ch1, :out, $n],
        ]};
    };
    test.add_step :click_amp {!(td, labels) = @;
        !res = $S(*:{source=cell_name, label=Amp}) labels;
        .amp_node_info = res.0;
        do_click td res.0;
    };
    test.add_step :check_amp_labels {!(td, labels) = @;
        !lbls = $S(*:{path=*param_panel*knob_label}/label) labels;
        std:assert_str_eq (std:sort lbls) $["att","gain","inp","neg_att"];
    };
    test.add_step :start_drag_new_node {!(td, labels) = @;
        !res = $S(*:{path=*pick_node_btn, label=Noise}) labels;
        do_drag td res.0;
    };
    test.add_step :drop_new_node {!(td, labels) = @;
        do_drop td amp_node_info;
    };
    test.add_step :check_noise_labels {!(td, labels) = @;
        !lbls = $S(*:{path=*param_panel*knob_label}/label) labels;
        std:assert_str_eq (std:sort lbls) $["atv","offs"];

    };
    ui:install_test test;
};
