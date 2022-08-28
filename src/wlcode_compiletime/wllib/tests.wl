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

!matrix_place = {!(pos, dir, chain) = @;
    !matrix = hx:get_main_matrix_handle[];
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

!connections_at = {!(pos) = @;
    !matrix = hx:get_main_matrix_handle[];
    matrix.get_connections pos
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
                $[:amp, :inp, :sig],
                $[:out, :ch1, $n],
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
                $[:amp, :inp, :sig],
                $[:out, :ch1, $n],
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
            matrix_place $i(1, 0) :TR ${chain=$[$[:noise, :sig]]};
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
                $[:cqnt, :inp, :sig],
                $[:bosc, :freq, :sig],
                $[:pverb, :in_l, :sig_l],
                $[:out, :ch1, $n],
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
            !res = $S(*:{
                ctrl=Ctrl\:\:Button,
                path=*main_panel*,
                label=\?
            }) labels;
            do_click td res.0;
        };
        test.add_step :check_help_text_there {!(td, labels) = @;
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
            !res = $S(*:{path=*popup*, label=Remove\ Cell}) labels;
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
            std:assert_eq res.0.source "text" "FbWr description text is displayed";
        };
    };

    add_test "connect nodes" {!(test) = @;
        test.add_step :init {||
            matrix_init  $i(0, 3) :B ${chain=$[
                $[:tslfo],
                $[:cqnt],
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
                $[:tslfo],
                $[:cqnt],
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
            matrix_init  $i(2, 2) :B ${chain=$[ $[:cqnt], ]};
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

    add_test "check mode button shows help" {!(test) = @;
        test.add_step :init {||
            matrix_init  $i(2, 2) :B ${chain=$[ $[:cqnt], ]};
        };
        test.add_step :focus_cqnt {!(td, labels) = @;
            do_click td ~ matrix_cell_label labels $i(2, 2);
        };
        test.add_step :check_octave_keys_are_shown {!(td, labels) = @;
            # TODO!
        };
        test.add_step :check_no_doc {!(td, labels) = @;
            !res = $S(*:{path=*help*wichtext, label=*Quantizer*}) labels;
            std:assert_str_eq
                res.0.source
                "text"
                "Found the small help text on screen";
        };
        test.add_step :hover_mode_button {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=-0}) labels;
            do_hover td res.0;
        };
        test.add_step :check_no_doc {!(td, labels) = @;
            !res = $S(*:{path=*help*wichtext, label=*omin*}) labels;
            std:assert_str_eq
                res.0.source
                "text"
                "Found the small help text with omin on screen";
        };
    };

    add_test "check trig parameter is a button" {!(test) = @;
        test.add_step :init {||
            matrix_init  $i(2, 2) :B ${chain=$[ $[:ad], ]};
        };
        test.add_step :focus_cqnt {!(td, labels) = @;
            do_click td ~ matrix_cell_label labels $i(2, 2);
        };
        test.add_step :hover_mode_button {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=trig}) labels;
            do_hover td res.0;
        };
        test.add_step :check_no_doc {!(td, labels) = @;
            !res = $S(*:{path=*help*wichtext, label=*trig*}) labels;
            std:assert_str_eq res.0.source "text" "Found the small help text with trig on screen";
        };
    };

    add_test "node picker LMB click" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(2, 2) :B ${chain=$[ ]};
        };
        test.add_step :goto_ntom_tab {!(td, labels) = @;
            !res = $S(*:{path=*.tab_hor, label=NtoM}) labels;
            do_click td res.0;
        };
        test.add_step :click_mix3 {!(td, labels) = @;
            !res = $S(*:{path=*.pick_node_btn, label=Mix3}) labels;
            do_click td res.0;
        };
        test.add_step :find_mix3 {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Mix3}) labels;
            std:assert_str_eq res.0.source "cell_name" "Found Mix3 node on matrix";
        };
        test.add_step :goto_ctrl_tab {!(td, labels) = @;
            !res = $S(*:{path=*.tab_hor, label=Ctrl}) labels;
            do_click td res.0;
        };
        test.add_step :click_mix3 {!(td, labels) = @;
            !res = $S(*:{path=*.pick_node_btn, label=Map}) labels;
            do_click td res.0;
        };
        test.add_step :find_mix3 {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Map}) labels;
            std:assert_str_eq
                res.0.source
                "cell_name"
                "Found Map node on matrix";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=inp}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'inp' input label";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=sig}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'sig' output label";
        };
    };

    add_test "node picker RMB click" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(2, 2) :B ${chain=$[ ]};
        };
        test.add_step :goto_ntom_tab {!(td, labels) = @;
            !res = $S(*:{path=*.tab_hor, label=NtoM}) labels;
            do_click td res.0;
        };
        test.add_step :click_mix3 {!(td, labels) = @;
            !res = $S(*:{path=*.pick_node_btn, label=Mix3}) labels;
            do_click_rmb td res.0;
        };
        test.add_step :find_mix3 {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Mix3}) labels;
            std:assert_str_eq res.0.source "cell_name" "Found Mix3 node on matrix";
        };
        test.add_step :goto_ctrl_tab {!(td, labels) = @;
            !res = $S(*:{path=*.tab_hor, label=Ctrl}) labels;
            do_click td res.0;
        };
        test.add_step :click_mix3 {!(td, labels) = @;
            !res = $S(*:{path=*.pick_node_btn, label=Map}) labels;
            do_click_rmb td res.0;
        };
        test.add_step :find_mix3 {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Map}) labels;
            std:assert_str_eq
                res.0.source
                "cell_name"
                "Found Map node on matrix";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=ch1}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'ch1' input label";
        };
    };

    add_test "node picker drag onto existing unconnected" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(2, 2) :B ${chain=$[ $[:sin] ]};
        };
        test.add_step :goto_ntom_tab {!(td, labels) = @;
            !res = $S(*:{path=*.tab_hor, label=NtoM}) labels;
            do_click td res.0;
        };
        test.add_step :drag_mix3 {!(td, labels) = @;
            !res = $S(*:{path=*.pick_node_btn, label=Mix3}) labels;
            do_drag td res.0;
        };
        test.add_step :drop_mix3 {!(td, labels) = @;
            do_drop td ~ matrix_cell_label labels $i(2, 2);
        };
        test.add_step :check_connection {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Sin}) labels;
            std:assert_eq len[res] 1 "Found one Sin nodes";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Mix3}) labels;
            std:assert_eq len[res] 1 "Found one Mix3 nodes";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=sig}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'sig' output label";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=freq}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'freq' input label";
        };
    };

    add_test "matrix move single cell adjacent connection" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(1, 1) :B ${chain=$[
                $[:sin, :sig],
                $[:amp, :inp, :sig],
                $[:out, :ch1, $n],
            ]};
        };
        test.add_step :drag_amp_cell {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Amp}) labels;
            do_drag_rmb td res.0;

            std:assert_eq len[connections_at $i(1, 2)] 2 "Two connections before movement";
        };
        test.add_step :drop_amp_cell {!(td, labels) = @;
            !res = matrix_cell_label labels $i(2, 2);
            do_drop_rmb td res;
        };
        test.add_step :check_amp_connections {!(td, labels) = @;
            std:assert_eq len[connections_at $i(2, 2)] 1 "One connection after movement";

            # drag one further:
            !res = matrix_cell_label labels $i(2, 2);
            do_drag_rmb td res;
        };
        test.add_step :drop_to_non_adjacent {!(td, labels) = @;
            !res = matrix_cell_label labels $i(3, 3);
            do_drop_rmb td res;
        };
        test.add_step :check_no_amp_connections {!(td, labels) = @;
            !connections = connections_at $i(3, 3);
            std:assert is_some[connections];
            std:assert_eq len[connections] 0 "No connections after movement";
        };
    };

    add_test "matrix move single cell adjacent two connections" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(1, 1) :BR ${chain=$[
                $[:sin, :sig],
                $[:amp, :inp, :sig],
                $[:out, :ch1, $n],
            ]};
        };
        test.add_step :drag_out_cell {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Out}) labels;
            do_drag_rmb td res.0;
        };
        test.add_step :drop_out_cell {!(td, labels) = @;
            !res = matrix_cell_label labels $i(3, 1);
            do_drop_rmb td res;
        };
        test.add_step :check_move_precond {!(td, labels) = @;
            std:assert_eq len[connections_at $i(2, 2)] 2 "Two Amp connections before movement";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Amp}) labels;
            do_drag_rmb td res.0;
        };
        test.add_step :drop_amp_cell {!(td, labels) = @;
            !res = matrix_cell_label labels $i(2, 1);
            do_drop_rmb td res;
        };
        test.add_step :check_move_postcond {!(td, labels) = @;
            std:assert_eq len[connections_at $i(2, 1)] 2 "Two Amp connections after movement";
        };
    };

    add_test "matrix move single cell adjacent preserve edges" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(1, 1) :BR ${chain=$[
                $[:sin, :sig],
                $[:amp, :inp, :sig],
                $[:out, :ch1, $n],
            ]};
        };
        test.add_step :drag_amp_cell {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Amp}) labels;
            do_drag_rmb td res.0;
        };
        test.add_step :drop_amp_cell {!(td, labels) = @;
            !res = matrix_cell_label labels $i(0, 1);
            do_drop_rmb td res;
        };
        test.add_step :check_edge_labels {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=sig}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'sig' output label";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=inp}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'inp' input label";
        };
    };

    add_test "matrix move single cell adjacent connection 2" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(1, 1) :B ${chain=$[
                $[:sin, :sig],
                $[:amp, :inp, :sig],
                $[:out, :ch1, $n],
            ]};
        };
        test.add_step :drag_amp_cell {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Sin}) labels;
            do_drag_rmb td res.0;

            std:assert_eq len[connections_at $i(1, 2)] 2 "Two connections before movement";
        };
        test.add_step :drop_amp_cell {!(td, labels) = @;
            !res = matrix_cell_label labels $i(0, 3);
            do_drop_rmb td res;
        };
        test.add_step :check_amp_connections {!(td, labels) = @;
            std:assert_eq len[connections_at $i(1, 2)] 2 "Two connection after movement";
        };
    };

    add_test "DSP chain splitting" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(1, 1) :B ${chain=$[
                $[:sin, :sig],
                $[:amp, :inp, :sig],
                $[:out, :ch1, $n],
            ]};
        };
        test.add_step :check_precond {!(td, labels) = @;
            std:assert_eq len[connections_at $i(1, 2)] 2 "Two connections before split";
        };
        test.add_step :drag_split_down {!(td, labels) = @;
            !res = matrix_cell_label labels $i(1, 2);
            do_drag_rmb td res;
        };
        test.add_step :drop_split_down {!(td, labels) = @;
            !res = matrix_cell_label labels $i(1, 3);
            do_drop_rmb td res;
        };
        test.add_step :check_postcond {!(td, labels) = @;
            std:assert_eq len[connections_at $i(1, 2)] 1 "One connections after split";
        };
        test.add_step :drag_split_up {!(td, labels) = @;
            !res = matrix_cell_label labels $i(1, 2);
            do_drag_rmb td res;
        };
        test.add_step :drop_split_up {!(td, labels) = @;
            !res = matrix_cell_label labels $i(1, 1);
            do_drop_rmb td res;
        };
        test.add_step :check_postcond2 {!(td, labels) = @;
            std:assert_eq len[connections_at $i(1, 2)] 0 "No connections after split";
        };
    };

    add_test "DSP chain splitting blocked" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(1, 1) :B ${chain=$[
                $[:sin, :sig],
                $[:amp, :inp, :sig],
                $[:out, :ch1, $n],
            ]};
            matrix_place $i(1, 4) :B ${chain=$[ $[:sin, :sig] ]};
        };
        test.add_step :check_precond {!(td, labels) = @;
            std:assert_eq len[connections_at $i(1, 2)] 2 "Two connections before split";
        };
        test.add_step :drag_split_down {!(td, labels) = @;
            !res = matrix_cell_label labels $i(1, 2);
            do_drag_rmb td res;
        };
        test.add_step :drop_split_down {!(td, labels) = @;
            !res = matrix_cell_label labels $i(1, 3);
            do_drop_rmb td res;
        };
        test.add_step :check_postcond {!(td, labels) = @;
            std:assert_eq len[connections_at $i(1, 2)] 2 "Still two connections after split";
        };
    };

    add_test "matrix drag linked copy to empty position" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(1, 1) :BR ${chain=$[ $[:amp, :inp, :sig], ]};
        };
        test.add_step :drag_from_empty {!(td, labels) = @;
            do_drag td ~ matrix_cell_label labels $i(2, 1);
        };
        test.add_step :drop_on_filled {!(td, labels) = @;
            do_drop td ~ matrix_cell_label labels $i(1, 1);
        };
        test.add_step :check_two_amps {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Amp}) labels;
            std:assert_eq len[res] 2 "Found two Amp nodes";
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, source=cell_num, label=0}) labels;
            std:assert_eq len[res] 2 "Found two 0 nodes";
        };
    };

    add_test "matrix drag linked copy default connected" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(1, 1) :BR ${chain=$[ $[:sin], ]};
            matrix_place $i(3, 3) :BR ${chain=$[ $[:amp], ]};
        };
        test.add_step :drag_from_sin {!(td, labels) = @;
            do_drag td ~ matrix_cell_label labels $i(1, 1);
        };
        test.add_step :drop_on_amp {!(td, labels) = @;
            do_drop td ~ matrix_cell_label labels $i(3, 3);
        };
        test.add_step :check_two_amps {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Sin}) labels;
            std:assert_eq len[res] 2 "Found two Sin nodes";
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, source=cell_num, label=0}) labels;
            std:assert_eq len[res] 3 "Found three 0 nodes";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=sig}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'sig' output label";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=inp}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'inp' input label";
        };
    };

    add_test "matrix drag instance default connected" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(1, 1) :BR ${chain=$[ $[:sin], ]};
            matrix_place $i(3, 3) :BR ${chain=$[ $[:amp], ]};
        };
        test.add_step :drag_from_sin {!(td, labels) = @;
            do_drag_rmb td ~ matrix_cell_label labels $i(1, 1);
        };
        test.add_step :drop_on_amp {!(td, labels) = @;
            do_drop_rmb td ~ matrix_cell_label labels $i(3, 3);
        };
        test.add_step :check_two_amps {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Sin}) labels;
            std:assert_eq len[res] 2 "Found two Sin nodes";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, source=cell_num, label=0}) labels;
            std:assert_eq len[res] 2 "Found two 0 nodes";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, source=cell_num, label=1}) labels;
            std:assert_eq len[res] 1 "Found one 1 nodes";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=sig}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'sig' output label";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=inp}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'inp' input label";
        };
    };

    add_test "matrix drag instance pre connected" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(1, 1) :BR ${chain=$[ $[:sin], ]};
            matrix_place $i(3, 3) :TR ${chain=$[ $[:ad, :atk, :eoet], ]};
        };
        test.add_step :drag_from_sin {!(td, labels) = @;
            do_drag_rmb td ~ matrix_cell_label labels $i(1, 1);
        };
        test.add_step :drop_on_amp {!(td, labels) = @;
            do_drop_rmb td ~ matrix_cell_label labels $i(3, 3);
        };
        test.add_step :check_two_amps {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Sin}) labels;
            std:assert_eq len[res] 2 "Found two Sin nodes";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=atk}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'atk' output label";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=sig}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'sig' input label";
        };
    };

    add_test "matrix drag linked copy pre connected" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(1, 1) :BR ${chain=$[ $[:sin], ]};
            matrix_place $i(3, 3) :TR ${chain=$[ $[:ad, :atk, :eoet], ]};
        };
        test.add_step :drag_from_sin {!(td, labels) = @;
            do_drag td ~ matrix_cell_label labels $i(1, 1);
        };
        test.add_step :drop_on_amp {!(td, labels) = @;
            do_drop td ~ matrix_cell_label labels $i(3, 3);
        };
        test.add_step :check_two_amps {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Sin}) labels;
            std:assert_eq len[res] 2 "Found two Sin nodes";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=atk}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'atk' output label";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=sig}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'sig' input label";
        };
    };

    add_test "node picker drag onto existing pre connected" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(3, 3) :TR ${chain=$[ $[:ad, :atk, :eoet], ]};
        };
        test.add_step :goto_ntom_tab {!(td, labels) = @;
            !res = $S(*:{path=*.tab_hor, label=NtoM}) labels;
            do_click td res.0;
        };
        test.add_step :drag_mix3 {!(td, labels) = @;
            !res = $S(*:{path=*.pick_node_btn, label=Mix3}) labels;
            do_drag td res.0;
        };
        test.add_step :drop_mix3 {!(td, labels) = @;
            do_drop td ~ matrix_cell_label labels $i(3, 3);
        };
        test.add_step :check_connection {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Ad}) labels;
            std:assert_eq len[res] 1 "Found one Ad nodes";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=Mix3}) labels;
            std:assert_eq len[res] 1 "Found one Mix3 nodes";
            std:assert_eq res.0.logic_pos $i(2, 4) "Position of Mix3 correct";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=sig}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'sig' output label";

            !res = $S(*:{ctrl=Ctrl\:\:HexGrid, label=atk}) labels;
            std:assert_str_eq res.0.tag "matrix_grid" "Found 'atk' input label";
        };
    };

    add_test "midip learn" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(1, 1) :B ${chain=$[
                $[:midip, :gate],
                $[:out, :ch1, $n],
            ]};
        };
        test.add_step :click_midip {!(td, labels) = @;
            !res = $S(*:{ctrl=*HexGrid, label=MidiP}) labels;
            do_click td res.0;
        };
        test.add_step :click_learn {!(td, labels) = @;
            # Finding no "13" button:
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=13}) labels;
            std:assert is_none[res];

            !res = $S(*:{ctrl=*Button, label=*Learn*}) labels;
            do_click td res.0;
        };
        test.add_step :send_midi {!(td, labels) = @;
            !matrix = hx:get_main_matrix_handle[];
            matrix.inject_midi_event ${
                channel = 13,
                note = 69,
                vel = 1.0,
                type = :note_on,
            };
        };
        test.add_step :sleep {|| std:thread:sleep :ms => 100 };
        test.add_step :check_chan_set {!(td, labels) = @;
            # Finding "13" button now:
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=13}) labels;
            std:assert is_some[res];
        };
    };

    add_test "midicc learn" {!(test) = @;
        test.add_step :init {||
            matrix_init $i(1, 1) :B ${chain=$[
                $[:midicc, :sig1],
                $[:out, :ch1, $n],
            ]};
        };
        test.add_step :click_midip {!(td, labels) = @;
            !res = $S(*:{ctrl=*HexGrid, label=MidiCC}) labels;
            do_click td res.0;
        };
        test.add_step :click_learn1 {!(td, labels) = @;
            # Finding no "13" button and no 55/56/57 button:
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=13}) labels;
            std:assert is_none[res];
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=55}) labels;
            std:assert is_none[res];
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=56}) labels;
            std:assert is_none[res];
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=57}) labels;
            std:assert is_none[res];

            !res = $S(*:{ctrl=*Button, label=*Learn*1*}) labels;
            do_click td res.0;
        };
        test.add_step :send_midi {!(td, labels) = @;
            !matrix = hx:get_main_matrix_handle[];
            matrix.inject_midi_event ${
                channel = 13,
                cc = 55,
                vel = 1.0,
                type = :cc,
            };
        };
        test.add_step :sleep {|| std:thread:sleep :ms => 100 };
        test.add_step :check_chan_set {!(td, labels) = @;
            # Finding "13" and "55" button now:
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=13}) labels;
            std:assert is_some[res];
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=55}) labels;
            std:assert is_some[res];
        };
        test.add_step :click_learn2 {!(td, labels) = @;
            !res = $S(*:{ctrl=*Button, label=*Learn*2*}) labels;
            do_click td res.0;
        };
        test.add_step :send_midi2 {!(td, labels) = @;
            !matrix = hx:get_main_matrix_handle[];
            matrix.inject_midi_event ${
                channel = 13,
                cc = 56,
                vel = 1.0,
                type = :cc,
            };
        };
        test.add_step :sleep2 {|| std:thread:sleep :ms => 100 };
        test.add_step :check_cc_set2 {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=56}) labels;
            std:assert is_some[res];
        };
        test.add_step :click_learn3 {!(td, labels) = @;
            !res = $S(*:{ctrl=*Button, label=*Learn*3*}) labels;
            do_click td res.0;
        };
        test.add_step :send_midi2 {!(td, labels) = @;
            !matrix = hx:get_main_matrix_handle[];
            matrix.inject_midi_event ${
                channel = 13,
                cc = 57,
                vel = 1.0,
                type = :cc,
            };
        };
        test.add_step :sleep2 {|| std:thread:sleep :ms => 100 };
        test.add_step :check_cc_set2 {!(td, labels) = @;
            !res = $S(*:{ctrl=Ctrl\:\:Button, label=57}) labels;
            std:assert is_some[res];
        };
    };
};
# dump_labels td;
# test.add_step :sleep {|| std:thread:sleep :ms => 1000 };
