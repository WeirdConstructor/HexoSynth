!COLORS = ${};

iter line (("\n" => 0) hexo_consts_rs) {
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

    std:displayln ~ ui.matrix[].get 1 1;

    !grid = ui.new_hexgrid 0 60.0 ${ position = "self" };

    ui.emit_to grid grid $[:hexgrid:set_test_model];

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
