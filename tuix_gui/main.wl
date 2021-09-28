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
            cont = ${ class = "ui_category_tab_cont" },
        };
    };

    !tab_cont_ids = ui.new_tabs parent tabs ${
        tab      = ${ class = $["tab", "tabx"] },
        tab_view = ${ class = "ui_category_tab_view" },
    };

    !i = 0;
    iter tc tab_cont_ids {
        !row = ui.new_row tc;
        !cat = (i + 1) cat_list;

        iter nid NODE_ID_CATEGORIES.(cat) {
            !mnid = nid;
            !btn =
                ui.new_button
                    row
                    node_id:label[nid]
                    {|| .CUR_NODE_TYPE = mnid; }
                    ${ class = "node_btn" };
        };

        .i += 1;
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

            matrix.set pos ${ node_id = CUR_NODE_TYPE };
            unwrap ~ matrix.sync[];
        },
        on_cell_drag = {!(ui, pos, pos2, btn) = @;
            std:displayln "DRAG CELL:" pos "=>" pos2 btn;
        },
    };

    std:displayln "A";
    !matrix_model = matrix.create_grid_model[];
    std:displayln "B";
    ui.emit_to 0 grid $p(:hexgrid:set_model, matrix_model);
    std:displayln "C";

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
