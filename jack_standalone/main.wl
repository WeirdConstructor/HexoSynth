!@import ui;

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

std:displayln "FOO";

root.add btn;

$[root]
