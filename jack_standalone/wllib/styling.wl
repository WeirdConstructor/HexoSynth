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
