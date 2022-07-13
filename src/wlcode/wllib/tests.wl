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

!do_click_rmb = {
    !pos = _1.pos + $f(1.0, 1.0);
    std:displayln ">>> click(RMB)@" pos;
    _.mouse_press_at pos :right;
    _.mouse_release_at pos :right;
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

!matrix_cell_label = {!(labels, pos) = @;
    !sel_str = $F $q(*:{{path=*.matrix_grid, label=hexcell_{}_{}}}) pos.0 pos.1;
    !sel = std:selector sel_str;
    (sel labels).0
};

!is_inside = {!(rect, pos) = @;
    std:displayln "INS " rect pos;
         pos.x >= rect.0
    &and pos.x <= (rect.0 + rect.2)
    &and pos.y >= rect.1
    &and pos.y <= (rect.1 + rect.3)
};

!pval = {!(param_id) = @;
    !matrix = hx:get_main_matrix_handle[];
    matrix.get_param param_id
};

!@export install = {!(test_match) = @;
    !add_test = {!(name, setup_fun) = @;
        if is_none[test_match]
           &or (std:pattern test_match) <& name {
            !test = ui:test_script name;
            setup_fun test;
            ui:install_test test;
        };
    };

    add_test "knob_hover_help_desc" {!(test) = @;
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
    };

    add_test "param_panel_update_on_matrix_change" {!(test) = @;
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
            !lbls = $S(*:{path=*param_panel*param_label}/label) labels;
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
            !lbls = $S(*:{path=*param_panel*param_label}/label) labels;
            std:assert_str_eq (std:sort lbls) $["atv","mode","offs"];

        };
    };

    add_test "cluster_move_focus_change_works" {!(test) = @;
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

    #    test.add_step :sleep {|| std:thread:sleep :ms => 1000 };

        test.add_step :check_bosc_help {!(td, labels) = @;
    #        dump_labels td;
            !res = $S(*:{ctrl=Ctrl\:\:Label, label=wtype}) labels;
            std:assert_str_eq res.0.tag "param_label" "Found wtype param";
        };

    #    test.add_step :sleep {|| std:thread:sleep :ms => 1000 };

        test.add_step :drag_bosc_start {!(td, labels) = @;
            do_drag td bosc_pos;
        };
        test.add_step :drag_bosc_end {!(td, labels) = @;
            !res = matrix_cell_label labels $i(7, 2);
            .bosc_pos = res;
            do_drop td res;
        };
        test.add_step :check_bosc_help_still_there {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:Label, label=wtype}) labels;
            std:assert_str_eq res.0.tag "param_label" "(Still) found wtype param";

            do_drag_rmb td bosc_pos;

            .bosc_pos = matrix_cell_label labels $i(7, 4);
        };
        test.add_step :check_bosc_help_still_there {!(td, labels) = @;
            do_drop_rmb td bosc_pos;
        };
        test.add_step :check_bosc_help_still_there {!(td, labels) = @;
    #        dump_labels td;
            !res = $S(*:{ctrl=Ctrl\:\:Label, label=wtype}) labels;
            std:assert_str_eq res.0.tag "param_label" "(Still) found wtype param";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=BOsc}) labels;
            std:assert_str_eq res.0.logic_pos $i(7, 4) "Confirm new BOsc pos";
        };
    };

    add_test  "node help button shows help" {!(test) = @;
        test.add_step :init {||
            matrix_init  $i(0, 0) :TR ${chain=$[]};
        };
        test.add_step :goto_ntom_tab {!(td, labels) = @;
            !res = $S(*:{path=*.tab_hor, label=NtoM}) labels;
            do_click td res.0;
        };
        test.add_step :hover_mux9 {!(td, labels) = @;
            !res = $S(*:{path=*.pick_node_btn, label=Mux9}) labels;
            do_hover td res.0;
        };
        test.add_step :check_mux9_help {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:WichText, label=9\ Ch.\ Mul*}) labels;
            std:assert_str_eq
                res.0.source
                "text"
                "Found the small help text on screen";
        };
        test.add_step :click_node_help_btn {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=\?}) labels;
            do_click td res.0;
        };
        test.add_step :click_node_help_btn {!(td, labels) = @;
            !res = $S(*:{
                ctrl=Ctrl\:\:WichText,
                path=*.main_help_wichtext,
                label=Mux9\ -\ 9\ Ch*
            }) labels;
            std:assert_str_eq
                res.0.source
                "text"
                "Found the long help text on screen";
        };
        test.add_step :close_help_text {!(td, labels) = @;
            # file:///home/weicon/devel/rust/hexosynth/jack_standalone/target/doc/src/keyboard_types/key.rs.html#957-1260
            unwrap ~ td.key_press :Escape;
        };
        test.add_step :check_help_text_away {!(td, labels) = @;
            !res = $S(*:{
                ctrl=Ctrl\:\:WichText,
                path=*.main_help_wichtext,
                label=Mux9\ -\ 9\ Ch*
            }) labels;
            std:assert_str_eq res.0.source $n "Help text no longer open";
    #        dump_labels td;
        };
    };

    add_test "create node and remove node" {!(test) = @;
        test.add_step :init {!(td, labels) = @;
            matrix_init  $i(0, 0) :TR ${chain=$[]};
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=IOUtil}) labels;
            do_click td res.0;
        };
        test.add_step :drag_out_to_matrix {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=Out}) labels;
            do_drag td res.0;
        };
        test.add_step :drop_out_on_matrix {!(td, labels) = @;
            do_drop td ~ matrix_cell_label labels $i(3, 3);
        };
        test.add_step :open_cell_context_menu {!(td, labels) = @;
            !res = $S(*:{path=*.matrix_grid, label=Out}) labels;
            std:assert_eq res.0.logic_pos $i(3, 3) "No out cell existing anymore";
            do_click_rmb td res.0;
        };
        test.add_step :click_on_remove {!(td, labels) = @;
            !res = $S(*:{path=*popup*, label=Remove}) labels;
            do_click td res.0;
        };
        test.add_step :verify_out_cell_gone {!(td, labels) = @;
            !res = $S(*:{path=*.matrix_grid, label=Out}) labels;
            std:assert_eq res $n "No out cell existing anymore";
        };
    };

    add_test "hover picker" {!(test) = @;
        test.add_step :goto_ioutil {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=IOUtil}) labels;
            do_click td res.0;
        };
        test.add_step :reset_help_text_to_out_node {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=Out}) labels;
            do_hover td res.0;
        };
        test.add_step :hover_fbwr_node_btn {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=FbWr}) labels;
            do_hover td res.0;
        };
        test.add_step :check_fbwr_desc {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:WichText, label=Feedback*Writer}) labels;
    #        dump_labels td;
            std:assert_eq res.0.source "text" "FbWr description text is displayed";
        };
    };

    add_test "connect nodes" {!(test) = @;
        test.add_step :init {||
            matrix_init  $i(0, 3) :B ${chain=$[
                $[:tslfo, $n],
                $[$n,  :cqnt, $n],
            ]};
        };
        test.add_step :drag {!(td, labels) = @;
            do_drag td ~ matrix_cell_label labels $i(0, 3);
        };
        test.add_step :drop {!(td, labels) = @;
            do_drop td ~ matrix_cell_label labels $i(0, 4);
        };
        test.add_step :check_not_connected {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=oct}) labels;
            std:assert is_none[res.0] "Did not find 'oct' label";
        };
        test.add_step :drag_out_label {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:Connector, label=sig}) labels;
            do_drag td res.0;
        };
        test.add_step :drag_out_label {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:Connector, label=oct}) labels;
            do_drop td res.0;
        };
        test.add_step :check_connected {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=oct}) labels;
            std:assert is_some[res.0] "Found 'oct' label";
        };
    };

    add_test "connect nodes" {!(test) = @;
        test.add_step :init {||
            matrix_init  $i(0, 3) :B ${chain=$[
                $[:tslfo, $n],
                $[$n,  :cqnt, $n],
            ]};
        };
        test.add_step :drag {!(td, labels) = @;
            do_drag td ~ matrix_cell_label labels $i(0, 3);
        };
        test.add_step :drop {!(td, labels) = @;
            do_drop td ~ matrix_cell_label labels $i(0, 4);
        };
        test.add_step :check_not_connected {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=oct}) labels;
            std:assert is_none[res.0] "Did not find 'oct' label";
        };
        test.add_step :drag_out_label {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:Connector, label=sig}) labels;
            do_drag td res.0;
        };
        test.add_step :drag_out_label {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:Connector, label=oct}) labels;
            do_drop td res.0;
        };
        test.add_step :check_connected {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=oct}) labels;
            std:assert is_some[res.0] "Found 'oct' label";
        };
    };

    add_test "check mode button functionality" {!(test) = @;
        test.add_step :init {||
            matrix_init  $i(2, 2) :B ${chain=$[ $[:cqnt, $n], ]};
        };
        test.add_step :focus_cqnt {!(td, labels) = @;
            do_click td ~ matrix_cell_label labels $i(2, 2);
        };

        !more_area = $n;
        test.add_step :click_minus_oct_lbl {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=-0}) labels;
            .more_area = $f(
                res.0.pos.0,
                res.0.pos.1 - 50.0,
                res.0.pos.2,
                res.0.pos.3 + 100.0
            );
            do_click td res.0;
        };
        test.add_step :click_minus2_popup_item {!(td, labels) = @;
            dump_labels td;
            !res = $S(*:{tag=mode_selector_item, label=-2}) labels;
            do_click td res.0;
        };
        test.add_step :check_omin_value {!(td, labels) = @;
            std:assert_eq
                (pval $p($p(:cqnt, 0), :omin)).i[]
                2 "Octave minimum is -2";
        };
        test.add_step :click_more {!(td, labels) = @;
            !res =
                filter { is_inside more_area _.pos }
                    ~ $S(*:{tag=mode_button_more, label=+}) labels;

            do_click td res.0;
        };
        test.add_step :check_omin_value_2 {!(td, labels) = @;
            std:assert_eq
                (pval $p($p(:cqnt, 0), :omin)).i[]
                3 "Octave minimum is -3";
        };
        test.add_step :click_less {!(td, labels) = @;
            !res =
                filter { is_inside more_area _.pos }
                    ~ $S(*:{tag=mode_button_less, label=-}) labels;
            do_click td res.0;
            do_click td res.0;
        };
        test.add_step :check_omin_value_2 {!(td, labels) = @;
            std:assert_eq
                (pval $p($p(:cqnt, 0), :omin)).i[]
                1 "Octave minimum is -1";
        };
    };
};
# dump_labels td;
