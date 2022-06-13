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

!matrix = hx:get_main_matrix_handle[];
!freq_s = 440.0;
!sin_freq = node_id:inp_param $p(:sin, 0) :freq;
lbl.set ~ sin_freq.format ~ sin_freq.norm freq_s;

btn.reg :click {
    std:displayln "CLICK!" @;
    lbl.set ~ sin_freq.format ~ sin_freq.norm freq_s;
    matrix.set_param sin_freq sin_freq.norm[freq_s];
    matrix.set_param $p($p(:amp, 0), :att) 1.0;
    .freq_s *= 1.25;
};

!ent = ui:widget style;
#ent.set_ctrl :entry lbl;

#ent.reg :changed {
#    std:displayln "CHANGED" @;
#};

std:displayln "FOO";

iter y 0 => 10 {
    iter x 0 => 10 {
        std:displayln ~ matrix.get $i(x, y);
    };
};

matrix.set_param $p($p(:amp, 0), :att) 0.0;
std:displayln ~ node_id:param_list $p(:amp, 0);

!knob_model = matrix.create_hex_knob_dummy_model[];
!knob = ui:widget style;
knob.set_ctrl :knob knob_model;

!knob_freq_model = matrix.create_hex_knob_model sin_freq;
!knob_freq = ui:widget style;
knob_freq.set_ctrl :knob knob_freq_model;

root.add btn;
root.add ent;
root.add knob;
root.add knob_freq;

$[root]
