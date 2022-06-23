!@wlambda;
!@import std;
!@import ui;

!:global style  = ${};
!:global layout = ${};

style.button = ${
    bg_color     = ui:UI_LBL_BG_CLR,
    border_color = ui:UI_ACCENT_CLR,
};
style.button_active = ${
    parent = :button,
    border_color = ui:UI_HLIGHT_CLR,
};
style.tab_hor = ${
    parent       = :button,
    border       = 2,
    border_style = $[:bevel, $f(5, 5, 0, 0)],
};
layout.tab_hor = ${
    left   = :pixels => 1,
    right  = :pixels => 1,
};


style.pick_node_btn = ${
    parent = :button,
    border_style = $[:hex, 10],
    border = 2,
    shadow_offs = $f(3, 3),
};
style.pick_node_drag_btn = ${
    parent = :pick_node_btn,
    border_style = $[:hex, 20],
};
layout.pick_node_btn = ${
    left   = :pixels => 5,
    right  = :pixels => 5,
    width  = :stretch => 1,
    height = :percent => 100,
};

style.pick_node_row = ${
    pad_top = 2,
    pad_bottom = 6,
};
layout.pick_node_row = ${
    layout_type = :row,
    height = :percent => 20,
};

layout.button_bar = ${
    layout_type = :row,
    height = :pixels => 30,
};

style.pick_node_bg_panel = ${
    border = 1,
    border_color = ui:UI_ACCENT_CLR,
    bg_color     = ui:UI_BG3_CLR,
};

!apply_class = $n;
.apply_class = {!(class, style_map, layout_map) = @;
    !st = style.(class);

    if is_some[st] {
        if is_some[st.parent] {
            apply_class st.parent style_map $n;
        };

        iter kv st {
            if kv.1 == "parent" { next[]; };

            style_map.(kv.1) = kv.0;
        };
    };

    if is_some[layout.(class)] {
        !ly = layout.(class);
        if is_some[ly.parent] {
            apply_class ly.parent $n layout_map;
        };

        iter kv ly {
            if kv.1 == "parent" { next[]; };

            layout_map.(kv.1) = kv.0;
        };
    }
};

!default_style = ui:style[];

!@export new_widget = {
    !layout = ${};
    !style  = ${};
    iter class @ {
        apply_class class style layout;
    };
    std:displayln "NEW WID" @ style;
    !wid = ui:widget ~ default_style.clone_set style;
    if len[layout] > 0 {
        wid.change_layout layout;
    };
    wid
};

!@export restyle = {
    !wid = @.0;
    !layout = ${};
    !style  = ${};
    iter idx 1 => len[@] {
        iter class @ {
            apply_class class style layout;
        };
    };
    if len[style] > 0 {
        wid.set_style ~ default_style.clone_set style;
    };
    if len[layout] > 0 {
        wid.change_layout layout;
    };
};
