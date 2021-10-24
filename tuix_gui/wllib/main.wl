# Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
# This file is a part of HexoSynth. Released under GPL-3.0-or-later.
# See README.md and COPYING for details.

!@import hx;
!@import node_id;
!@import wpp;
!@import vizia;
!@import observable;

!@import wid_params   = params_widget;
!@import wid_settings = settings_widget;
!@import wid_connect  = connect_widget;

!NODE_ID_CATEGORIES = node_id:ui_category_node_id_map[];

std:displayln NODE_ID_CATEGORIES;

!COLORS = ${};

iter line (("\n" => 0) hx:hexo_consts_rs) {
    if line &> $r/*const\ (^UI_$+$S)*hxclr!\(0x(^$+[^\)])\)/ {
        COLORS.($\.1) = "#" $\.2;
    };
};

!replace_colors_in_text = {!(text) = @;
    iter kv COLORS {
        #d# std:displayln "REPLACE" kv;
        .text = (kv.1 => kv.0) text;
    };
    text
};

!load_theme = {
    vizia:add_theme
        ~ replace_colors_in_text
        ~ wpp:run_macro_lang[${}, std:io:file:read_text "main_style.css"];
};


!create_node_id_selector = {!(parent, select_node_cb) = @;
    !tabs = $[];

    !cat_list = node_id:ui_category_list[];

    iter cat cat_list {
        if cat == :none { next[]; };

        std:push tabs ${
            name = "tab_" std:str:to_lowercase[cat],
            title = cat,
            cont = ${
                class       = "ui_category_tab_cont",
                layout_type = :grid,
                grid_rows   = $@vec ($iter 0 => 5) {|| $+ 1 => :s },
                grid_cols   = $@vec ($iter 0 => 4) {|| $+ 1 => :s },
            },
        };
    };

    !tab_cont_ids = vizia:new_tabs parent tabs ${
        tab      = ${ class = $["tab", "tabx"], },
        tab_view = ${ class = "ui_category_tab_view" },
    };

    !i = 0;
    iter tc tab_cont_ids {
        !cat = (i + 1) cat_list;
        !j = 0;

        iter nid NODE_ID_CATEGORIES.(cat) {
            !row_i = j / 4;
            !col_i = j - (4 * row_i);
            .j += 1;

            !mnid = nid;
            !btn =
                vizia:new_button
                    tc
                    node_id:label[nid]
                    {||
                        select_node_cb mnid;

                        std:displayln "pbyidx" ~ node_id:param_by_idx mnid 0;
                        std:displayln "inp_p"  ~ node_id:inp_param mnid "inp";
                        std:displayln "plist"  ~ node_id:param_list mnid;
                        std:displayln "in2i"   ~ node_id:inp_name2idx mnid "inp";
                        std:displayln "ii2n"   ~ node_id:inp_idx2name mnid 0;
                        std:displayln "on2i"   ~ node_id:out_name2idx mnid "sig";
                        std:displayln "oi2n"   ~ node_id:out_idx2name mnid 0;
                        std:displayln "node_info" ~ node_id:info mnid;
                    }
                    ${
                        class = "node_btn",
                        row = row_i,
                        col = col_i,
                    };
        };

        .i += 1;
    };
};

!checked_matrix_change = {!(matrix, cb) = @;
    matrix.save_snapshot[];

    !res = block :err {
        _? :err cb[matrix];
        _? :err matrix.check[];
    };

    if is_err[res] {
        matrix.restore_snapshot[];
    } {
        if is_err[matrix.sync[]] ~
            matrix.restore_snapshot[];
    };
};

#!:global TEST_WID = $n;

!STATE = ${
    _data = ${
        m                = $n,       # The matrix (connection to the audio thread)
        grid_model       = $n,       # HexGrid data model handle
        place_node_type  = :sin => 0,# Currently selected/placeable node type
        focus = ${                   # Focused/Selected HexGrid cell
            pos     = $n,            #  - X/Y pos
            cell    = $n,            #  - Matrix cell
            node_id = $n,            #  - NodeId of that cell
        },
        widgets = ${                 # Stores handles to some custom widgets
            params = $n,             # Stores the ParamsWidget object/structure
        },
    },
    _proto = ${
        init = {!(matrix) = @;
            $data.m          = matrix;
            $data.grid_model = matrix.create_grid_model[];
            $data.widgets.params =
                wid_params:ParamsWidget.new[matrix];
        },
        build_main_gui = {!(grid) = @;
            wid_settings:init_global_settings_popup[];
            wid_connect:init_global_connect_popup $data.m;
            $data.widgets.params.build 0;

            vizia:emit_to 0 grid $p(:hexgrid:set_model, $data.grid_model);

            !self = $w& $self;
            create_node_id_selector 0 { self.set_place_node_type _ };

#            !panel = vizia:new_elem 0 ${
#                class = "con_panel",
#                width    = 380 => :px,
#                height   = 220 => :px,
#                left     = 200,
#                top      = 200,
#                position = :self,
#            };
#            !con = vizia:new_connector panel ${
#                on_change = { std:displayln "ONCHANGE:" @; },
#                on_hover  = { std:displayln "ONHOVER:" @; },
#            };
#            vizia:emit_to 0 con $[
#                :connector:set_items,
#                $[
#                    #  name,      assignable or "unused"
#                    $p("out1",    $t),
#                    $p("output2", $t),
#                    $p("o3",      $f),
#                    $p("o4",      $f),
#                    $p("o5",      $t),
#                ],
#                $[
#                    $p("i1",      $t),
#                    $p("input2",  $t),
#                    $p("inpttt3", $t),
#                    $p("i4",      $t),
#                    $p("i5",      $f),
#                    $p("i6",      $f),
#                    $p("i8",      $f),
#                ]
#            ];

            vizia:new_button 0 "reload" {
                load_theme[];
                vizia:redraw[];
            } ${ width = 90 => :px, height = 20 => :px };
        },
        set_focus = {!(pos) = @;
            !focus = $data.focus;
            focus.pos     = pos;
            focus.cell    = $data.m.get pos;
            focus.node_id = $data.focus.cell.node_id;

            $data.grid_model.set_focus_cell pos;
            $data.widgets.params.set_node_id $data.focus.node_id;
            vizia:redraw[];
        },
        set_place_node_type = {!(typ) = @;
            $data.place_node_type = typ;
        },
        do_place_at = {!(pos) = @;
            !new_node_id =
                $data.m.get_unused_instance_node_id
                    $data.place_node_type;
            $data.m.set pos ${ node_id = new_node_id };
            unwrap ~ $data.m.sync[];
        },
        move_cluster = {!(pos_a, pos_b) = @;
            !cluster = hx:new_cluster[];
            cluster.add_cluster_at $data.m pos_a;

            !pth = hx:dir_path_from_to pos_a pos_b;

            checked_matrix_change $data.m {!(matrix) = @;
                cluster.remove_cells matrix;
                _? :err ~ cluster.move_cluster_cells_dir_path pth;
                _? :err ~ cluster.place matrix;
            };
        },
    },
};

#!oct_keys = $n;
#!g_mask = 0b11001;

!:global init = {
    load_theme[];

    !matrix = hx:get_main_matrix_handle[];
    STATE.init matrix;

    matrix.set_param $p(:amp => 0, "att") 0.0;

    !grid = vizia:new_hexgrid 0 ${
        position = "self",
        on_click = {!(pos, btn) = @;
            std:displayln "CLICK CELL:" pos btn;

            match btn
                :left => {
                    !cell = matrix.get pos;
                    if cell.node_id.0 == "nop" {
                        .cell = $n;
                    };

                    std:displayln "CELL CONT:" cell;

                    if is_some[cell] {
                        STATE.set_focus pos;
                    } {
                        STATE.do_place_at pos;
                    };
                }
                :right => {
                    !cluster = hx:new_cluster[];
                    cluster.add_cluster_at matrix pos;
                    std:displayln cluster.cell_list[];

                    checked_matrix_change matrix {!(matrix) = @;
                        cluster.remove_cells matrix;
                        _? :err ~ cluster.move_cluster_cells_dir_path $[:B];
                        _? :err ~ cluster.place matrix;
                    };

                    !d = hx:dir :T;
                    !d2 = d.flip[];
                    std:displayln
                        d d2
                        d.is_input[] d2.is_input[]
                        d2.as_edge[];
                };
        },
        on_cell_drag = {!(pos, pos2, btn) = @;

            !adj_dir = hx:pos_are_adjacent pos pos2;
            if is_some[adj_dir] {
                if btn == :left {
                    !cell_out = matrix.get pos;
                    !cell_in  = matrix.get pos2;

                    if adj_dir.is_input[] {
                        .(cell_in, cell_out) = $p(cell_out, cell_in);
                    };

                    if      cell_out.node_id.0 != "nop"
                       &and cell_in.node_id.0  != "nop" {

                        std:displayln "DRAG ADJ:"
                            adj_dir.is_output[]
                            cell_out
                            cell_in;
                        wid_connect:connect cell_out cell_in;
                        return $n;
                    }
                } {

                };
            };

            if btn == :left {
                STATE.move_cluster pos pos2;
            };
        },
    };

    STATE.build_main_gui[grid];

#    !dummy_settings = $[
#        0 => "Off",
#        1 => "On",
#        2 => "LowPass",
#        3 => "HighPass",
#    ];


#    !panel = vizia:new_elem 0 ${ class = "knob_panel" };

#    !param_id = node_id:inp_param :sin => 0 "freq";
#    !dmy = matrix.create_hex_knob_dummy_model[];
#    std:displayln :DMY: param_id;
#    !dmy = matrix.create_hex_knob_model[param_id];

#    !my_wid = wid_settings:SettingsWidget.new dummy_settings;
#    .TEST_WID = my_wid;
#    my_wid.build panel;
#    my_wid.listen :changed {!(ev, idx) = @;
#        if idx == 3 {
#            .TEST_WID = $n;
#        };
#    };

#    !keys = vizia:new_octave_keys panel ${
#        on_change = {!(mask) = @;
#            .g_mask = mask;
#        }
#    };
#
#    .oct_keys = keys;

#    !cva = vizia:new_cv_array panel ${
#        on_change = {!(array) = @;
#            std:displayln array;
#        }
#    };
#
#    !pf = vizia:new_hexknob panel dmy;

#    !buf =
#        hx:new_sample_buf_from
#            $[0.1, 0.2, 0.3, 0.4];
#
#    std:displayln "BUF:" buf;
#
#    std:displayln "0:" buf.0;
#    std:displayln "1:" buf.1;
#    std:displayln "2:" buf.2;
#
#    buf.1 = 23.42;
#    std:displayln "1:" buf.1;
#


#    !col = ui.new_col 0 ${ position = "self" };
#
#    !par = ui.new_row col "headbar";
#
#    !i = 0;
#    !btn = $n;
#
#    .btn = ui.new_button par "Test" {
#        .i += 1;
#
#        std:displayln "CLICK:" i;
#
#
#        _.set_text btn ~ $F "Counter: {}" i;
#        _.redraw[];
#    };
#
#    ui.new_hexknob par;
#
#    !par2 = ui.new_row col;
#
#    ui.new_pattern_editor par2;
#
#    !tab_cont_ids = ui.new_tabs par2 $[
#            ${ name = "first",  title = "First",      cont = ${ class = "tab_cont" } },
#            ${ name = "second", title = "Second Tab", cont = ${ class = "tab_cont" } },
#            ${ name = "third",  title = "Third Tab",  cont = ${ class = "tab_cont" } },
#        ] ${
#            tab_class = "tab",
#            tab_view = ${ }, # attribs
#        };
#
#    !count = 0;
#    !create_first = {!(ui, parent) = @;
#        !first = $n;
#        .first =
#            ui.new_button
#                parent
#                ("Test First " str[count])
#                { _.remove first; };
#        .count += 1;
#    };
#
#    create_first ui tab_cont_ids.0;
#
#    ui.new_button tab_cont_ids.1 "Test Second" {!(ui) = @;
#        create_first ui tab_cont_ids.0;
#    };
};

!:global idle = {
    if is_some[_] > 0 {
        std:displayln "IDLE with a change!";
    };

#    vizia:emit_to 0 oct_keys
#        $p(:octave_keys:set_mask, g_mask);

    iter change _ {
        match change
            $p(:matrix_param, param_id) => {
                vizia:redraw[];
#                std:displayln "PARAM:" $\.param_id.as_parts[];
#                std:displayln "PARAM:" $\.param_id.name[];
#                std:displayln "PARAM:" $\.param_id.default_value[];
#                std:displayln ~ (hx:to_atom 10);
#                std:displayln ~ (hx:to_atom 10.3);
#                std:displayln ~ (hx:to_atom "foobar");
#                std:displayln ~ (hx:to_atom 0 => 10).micro_sample[];
            }
            {
                std:displayln " * matrix change: " change;
            };
    };
};

#    !test_model = hx:create_test_hex_grid_model[];
#    ui.emit_to 0 grid $p(:hexgrid:set_model, test_model);
