# Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
# This file is a part of HexoSynth. Released under GPL-3.0-or-later.
# See README.md and COPYING for details.

!@import hx      = hx;
!@import node_id = node_id;

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

!CUR_NODE_TYPE = :sin => 0;

!create_node_id_selector = {!(ui, parent) = @;
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

    !tab_cont_ids = ui.new_tabs parent tabs ${
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
                ui.new_button
                    tc
                    node_id:label[nid]
                    {||
                        .CUR_NODE_TYPE = mnid;

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

!:global init = {!(ui) = @;
    ui.add_theme
        ~ replace_colors_in_text
        ~ std:io:file:read_text "main_style.css";

    !matrix = hx:get_main_matrix_handle[];
    matrix.set $i(1, 1) ${ node_id = CUR_NODE_TYPE };
    std:displayln ~ matrix.get $i(1, 1);

    !grid = ui.new_hexgrid 0 ${
        position = "self",
        on_click = {!(ui, pos, btn) = @;
            std:displayln "CLICK CELL:" pos btn;

            match btn
                :left => {
                    !new_node_id =
                        matrix.get_unused_instance_node_id CUR_NODE_TYPE;
                    matrix.set pos ${ node_id = new_node_id };
                    unwrap ~ matrix.sync[];
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
        on_cell_drag = {!(ui, pos, pos2, btn) = @;

            if btn == :left {
                !cluster = hx:new_cluster[];
                cluster.add_cluster_at matrix pos;

                !pth = hx:dir_path_from_to pos pos2;

                checked_matrix_change matrix {!(matrix) = @;
                    cluster.remove_cells matrix;
                    _? :err ~ cluster.move_cluster_cells_dir_path pth;
                    _? :err ~ cluster.place matrix;
                };
            };

            std:displayln "DRAG CELL:" pos "=>" pos2 btn;
        },
    };

    std:displayln "A";
    !matrix_model = matrix.create_grid_model[];
    std:displayln "B";
    ui.emit_to 0 grid $p(:hexgrid:set_model, matrix_model);
    std:displayln "C";

    !panel = ui.new_elem 0 ${ class = "knob_panel" };

    !param_id = node_id:inp_param :sin => 0 "freq";
#    !dmy = matrix.create_hex_knob_dummy_model[];
    std:displayln :DMY: param_id;
    !dmy = matrix.create_hex_knob_model[param_id];

    !pf = ui.new_hexknob panel dmy;

    create_node_id_selector ui 0;

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


#    !test_model = hx:create_test_hex_grid_model[];
#    ui.emit_to 0 grid $p(:hexgrid:set_model, test_model);
