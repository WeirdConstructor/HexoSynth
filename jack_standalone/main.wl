!@import ui;
!@import hx;
!@import node_id;

!style = ui:style[];

!lbl = ui:txt "Test123";

!root = ui:widget style;

style.set ${
    bg_color   = std:v:hex2rgba_f "222",
    color      = $f(1.0, 1.0, 0.0),
    font_size  = 24,
    text_align = :left,
    pad_left   = 20,
};

!btn = ui:widget style;
btn.set_ctrl :button lbl;

btn.reg :click {
    std:displayln "CLICK!" @;
};

!ent = ui:widget style;
#ent.set_ctrl :entry lbl;

#ent.reg :changed {
#    std:displayln "CHANGED" @;
#};

std:displayln "FOO";

!matrix = hx:get_main_matrix_handle[];

iter y 0 => 10 {
    iter x 0 => 10 {
        std:displayln ~ matrix.get $i(x, y);
    };
};

#matrix.

root.add btn;
root.add ent;

$[root]
