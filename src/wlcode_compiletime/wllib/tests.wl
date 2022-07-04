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

!do_drag_rmb = {
    !pos = _1.pos + $f(1.0, 1.0);
    std:displayln ">>> pick(RMB)@" pos;
    _.mouse_press_at pos :right;
};

!do_drop_rmb = {
    !pos = _1.pos + $f(1.0, 1.0);
    std:displayln ">>> drop(RMB)@" pos;
    _.mouse_release_at pos :right;
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
        std:assert_str_eq (std:sort lbls) $["atv","mode","offs"];

    };
    ui:install_test test;

    .test = ui:test_script "cluster_move_focus_change_works";
    test.add_step :init {||
        matrix_init  $i(0, 3) :TR ${chain=$[
            $[:tslfo, :sig],
            $[:inp,  :cqnt, :sig],
            $[:freq, :bosc, :sig],
            $[:in_l, :pverb, :sig_l],
            $[:ch1, :out, $n],
        ], params = $[
            $[:time => 4000.0],
            $n,
            $[:wtype => 2],
            $[:size   => 0.2,
              :predly => 40.0,
              :dcy    => 0.3],
            $[:gain => 0.1,
              :mono => 1],
        ]};
    };
    !bosc_pos = $n;
    test.add_step :click_bosc {!(td, labels) = @;
        !res = $S(*:{source=cell_name, label=BOsc}) labels;
        .bosc_pos = res.0;
        do_click td bosc_pos;
    };
    test.add_step :check_bosc_help {!(td, labels) = @;
        !res = $S(*:{ctrl=Ctrl\:\:Label, label=wtype}) labels;
        std:assert_str_eq res.0.tag "knob_label" "Found wtype param";
    };
    test.add_step :drag_bosc_start {!(td, labels) = @;
        do_drag td bosc_pos;
    };
    test.add_step :drag_bosc_end {!(td, labels) = @;
        !res = $S(*:{path=*.matrix_grid, label=hexcell_7_2}) labels;
        .bosc_pos = res.0;
        do_drop td res.0;
    };
    test.add_step :check_bosc_help_still_there {!(td, labels) = @;
        !res = $S(*:{ctrl=Ctrl\:\:Label, label=wtype}) labels;
        std:assert_str_eq res.0.tag "knob_label" "(Still) found wtype param";

        do_drag_rmb td bosc_pos;

        !res = $S(*:{path=*.matrix_grid, label=hexcell_7_4}) labels;
        .bosc_pos = res.0;
    };
    test.add_step :check_bosc_help_still_there {!(td, labels) = @;
        do_drop_rmb td bosc_pos;
    };
    test.add_step :check_bosc_help_still_there {!(td, labels) = @;
#        dump_labels td;
        !res = $S(*:{ctrl=Ctrl\:\:Label, label=wtype}) labels;
        std:assert_str_eq res.0.tag "knob_label" "(Still) found wtype param";

        !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=BOsc}) labels;
        std:assert_str_eq res.0.logic_pos $i(7, 4) "Confirm new BOsc pos";
    };
    ui:install_test test;
};
