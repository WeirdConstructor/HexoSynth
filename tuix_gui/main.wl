!@import hx = hx;

!COLORS = ${};

iter line (("\n" => 0) hx:hexo_consts_rs) {
    if line &> $r/$*?const\ (^UI_$+$S)$*?hxclr!\(0x(^$+[^\)])\)/ {
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

!:global init = {!(ui) = @;
    ui.add_theme
        ~ replace_colors_in_text
        ~ std:io:file:read_text "main_style.css";

    !matrix = hx:get_main_matrix_handle[];
    matrix.set $i(1, 1) ${ node_id = $p("sin", 1) };
    std:displayln ~ matrix.get $i(1, 1);

    !grid = ui.new_hexgrid 0 ${
        position = "self",
        on_click = {!(ui, pos, btn) = @;
            std:displayln "CLICK CELL:" pos btn;

            matrix.set pos ${ node_id = $p("sin", 1) };
            unwrap ~ matrix.sync[];
        },
        on_cell_drag = {!(ui, pos, pos2, btn) = @;
            std:displayln "DRAG CELL:" pos "=>" pos2 btn;
        },
    };

#    !test_model = hx:create_test_hex_grid_model[];
#    ui.emit_to 0 grid $p(:hexgrid:set_model, test_model);
    std:displayln "A";
    !matrix_model = matrix.create_grid_model[];
    std:displayln "B";
    ui.emit_to 0 grid $p(:hexgrid:set_model, matrix_model);
    std:displayln "C";

    !col = ui.new_col 0 ${ position = "self" };

    !par = ui.new_row col "headbar";

    !i = 0;
    !btn = $n;

    .btn = ui.new_button par "Test" {
        .i += 1;

        std:displayln "CLICK:" i;


        _.set_text btn ~ $F "Counter: {}" i;
        _.redraw[];
    };

    ui.new_hexknob par;

    !par2 = ui.new_row col;

    ui.new_pattern_editor par2;
};
