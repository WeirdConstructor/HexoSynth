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

    !par = ui.new_row 0;

    !i = 0;
    !btn = $n;

    .btn = ui.new_button par "Test" {
        .i += 1;

        std:displayln "CLICK:" i;

        _.set_text btn ~ $F "Counter: {}" i;
        _.redraw[];
    };

    ui.new_hexknob par;
};
