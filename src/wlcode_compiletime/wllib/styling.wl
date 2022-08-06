!@wlambda;
!@import std;
!@import ui;

!:global style  = ${};
!:global layout = ${};

layout.root = ${
    layout_type = :row,
};

style.panel = ${
    border       = 2,
    border_style = $[:rect],
};

layout.misc_panel = ${
    position_type = :self,
    left          = :stretch => 1.0,
    width         = :percent => 30,
    min_width     = :pixels => 200,
};

layout.main_panel = ${
    layout_type = :column,
    width       = :percent => 30,
    min_width   = :pixels  => 300
};

layout.picker_slide_panel = ${
    top         = :stretch => 1.0,
    width       = :percent => 50,
    height      = :pixels  => 200,
    min_width   = :pixels => 400,
    layout_type = :row,
};

layout.picker_btn_bar = ${
    width = :pixels => 40,
    layout_type = :column,
};


#style.right_util_panel_cont =
#    ${ border_color = $f(255.0, 0.0, 0.0), border = 3, typ = :rect, border_style = $[:rect] };

#style.right_slide_panel =
#    ${ border = 3, typ = :rect, border_style = $[:rect] };

layout.right_slide_panel = ${
    layout_type = :row,
    position_type = :self,
    left  = :stretch => 1,
    width = :auto,
    right = :pixels => 0,
};

layout.grid_panel = ${
    layout_type = :column,
};


style.close_hor_slide_left_panel_btn = ${
    border       = 2,
    border_style = $[:bevel, $f(0, 10, 0, 10)],
    border_color = ui:UI_ACCENT_CLR,
    bg_color     = ui:UI_BG2_CLR,
};
layout.close_hor_slide_left_panel_btn = ${
    width  = :pixels => 30,
    top    = :stretch => 1,
    bottom = :stretch => 1,
    height = :percent => 25,
    left   = :pixels => -2,
};

layout.right_util_panel = ${
    position_type = :self,
    left          = :stretch => 1,
    width         = :percent => 20,
    min_width     = :pixels => 270,
};

style.close_hor_slide_right_panel_btn = ${
    border       = 2,
    border_style = $[:bevel, $f(10, 0, 10, 0)],
    border_color = ui:UI_ACCENT_CLR,
    bg_color     = ui:UI_BG2_CLR,
};
layout.close_hor_slide_right_panel_btn = ${
    width  = :pixels => 30,
    bottom = :pixels => 50,
    height = :pixels => 100,
    top    = :stretch => 1,
    left   = :stretch => 1,
    right  = :pixels => -2,
};

style.matrix_grid = ${ border = 0 };
layout.matrix_grid = ${
    height = :stretch => 1,
};

style.label = ${
    bg_color = ui:UI_LBL_BG_CLR,
    border   = 0,
};
style.button = ${
    bg_color     = ui:UI_LBL_BG_CLR,
    border_color = ui:UI_ACCENT_CLR,
};
style.button_active = ${
    parent = :button,
    border_color = ui:UI_HLIGHT_CLR,
};
style.button_label = ${
    parent = :button,
};
layout.button_label = ${
    height = :pixels => 30,
};
style.button_big = ${
    parent = :button,
    font_size = 18,
};

style.button_float_menu = ${
    parent = :button,
    border_style = $[:bevel, $f(0, 0, 0, 10)],
    shadow_offs = $f(3, 3),
};
layout.top_float_menu = ${
    position_type = :self,
    layout_type = :row,
    width  = :pixels => 320,
    height = :pixels => 30,
    left   = :pixels => 0,
    right  = :stretch => 1,
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

style.connector = ${
    bg_color = ui:UI_LBL_BG_CLR,
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
layout.pick_node_bg_panel = ${
    layout_type = :row,
};

layout.knob_row = ${
    layout_type = :row,
    max_height  = :pixels => 130,
    height      = :stretch => 1,
};

layout.param_container = ${
    height = :stretch => 1,
};
style.param_container = ${
#    typ    = :rect,
    border = 2,
    border_color = ui:UI_ACCENT_CLR,
};
style.knob = ${
    border = 0,
};
style.param_label = ${
    border = 0,
    bg_color = ui:UI_ACCENT_BG1_CLR,
    color = ui:UI_PRIM_CLR,
};
layout.param_label = ${
    height = :pixels => 20,
    left   = :pixels => 2,
    right  = :pixels => 2,
};

layout.mode_btn_cont = ${
    layout_type = :column,
    height = :pixels => 100,
    top = :stretch => 1,
};
style.mode_button_more = ${
    parent = :button,
    border_style = $[:bevel, $f(15, 15, 0, 0)],
};
layout.mode_button_updown = ${
    width = :pixels => 40,
    left = :stretch => 1,
    right = :stretch => 1,
    height = :stretch => 1,
};
layout.mode_button_more = ${
    parent = :mode_button_updown,
};
style.mode_button_less = ${
    parent = :button,
    border_style = $[:bevel, $f(0, 0, 15, 15)],
};
layout.mode_button_less = ${
    parent = :mode_button_updown,
};
style.mode_button = ${ parent = :button };
layout.mode_button = ${
    height = :stretch => 1,
};

style.param_trig_button = ${
    parent = :button,
    border_style = $[:hex, 13.0],
};
layout.param_trig_button = ${
    left = :pixels => 4,
    top = :stretch => 1,
    bottom = :stretch => 1,
    height = :pixels => 80,
    right = :pixels => 4,
};

style.param_sample_button = ${
    parent = :button,
    border_style = $[:bevel, $f(5, 5, 5, 5)],
};

layout.popup_menu = ${
    position_type = :self,
    layout_type   = :column,
    height        = :auto,
    width         = :pixels => 200,
    visible       = $f,
};
style.popup_menu_item = ${
    parent = :button,
};
layout.popup_menu_item = ${
    height = :pixels => 27,
};

layout.mode_selector_popup = ${ parent = :popup_menu, };

style.mode_selector_item = ${ parent = :popup_menu_item, };
layout.mode_selector_item = ${ parent = :popup_menu_item, };

style.cell_context_item = ${ parent = :popup_menu_item, };
layout.cell_context_item = ${ parent = :popup_menu_item, };

style.wichtext = ${
    bg_color = ui:UI_ACCENT_BG1_CLR,
    border = 0,
    pad_left = 4,
    pad_right = 4,
    pad_top = 4,
    pad_bottom = 4,
};

style.main_help_wichtext = ${
    bg_color   = ui:UI_ACCENT_BG1_CLR,
    border     = 3,
    pad_left   = 10,
    pad_right  = 10,
    pad_top    = 10,
    pad_bottom = 10,
};
layout.main_help_wichtext = ${
    top = :pixels => 33,
};

style.sample_list_wichtext = ${
    parent = :wichtext,
};

style.help_btn = ${
    parent = :button,
    border_style = $[:bevel, $f(6, 6, 6, 6)],
};
layout.help_btn = ${
    width  = :pixels => 35,
    height = :pixels => 35,
};

style.top_right_help_btn = ${
    parent = :help_btn,
};
layout.top_right_help_btn = ${
    parent = :help_btn,
    position_type = :self,
    left = :stretch => 1,
    right = :pixels => 0,
};

style.help_text_panel = ${
};
layout.help_text_panel = ${
    height     = :stretch => 1.0,
    min_height = :pixels => 170,
};

layout.keys = ${
    max_height = :pixels => 180,
};

style.node_graph = ${
    bg_color = ui:UI_ACCENT_BG1_CLR,
};
layout.node_graph = ${
    max_height = :pixels => 180,
};

style.pattern_editor_label = ${
    parent = :label,
};
layout.pattern_editor_label = ${
    height = :pixels => 35,
};
layout.pattern_editor = ${
    width = :pixels => 350,
};
style.pattern_editor = ${
    bg_color     = ui:UI_LBL_BG_CLR,
    border_color = ui:UI_ACCENT_CLR,
#    pad_left     = 4,
#    pad_right    = 4,
#    pad_top      = 4,
#    pad_bottom   = 4,
};

layout.cell_channel_monitor_cont = ${
    height = :auto,
};
style.cell_channel_monitor_cont = ${
    pad_bottom = 2,
};
style.cell_channel_monitor_label = ${
    parent = :label,
    border = 0,
    font_size = 12,
};
layout.cell_channel_monitor_label = ${
    height = :pixels => 13,
};
style.cell_channel_monitor = ${
    bg_color = ui:UI_LBL_BG_CLR,
    font_size = 11,
};
layout.cell_channel_monitor = ${
    height = :pixels => 60,
    width = :pixels => 160,
};

style.cell_channel_monitor_out = ${
    parent = :cell_channel_monitor,
    color = ui:STD_COLORS.17,
};
layout.cell_channel_monitor_out = ${
    parent = :cell_channel_monitor,
};

style.cell_channel_monitor_in = ${
    parent = :cell_channel_monitor,
    color = ui:STD_COLORS.8,
};
layout.cell_channel_monitor_in = ${
    parent = :cell_channel_monitor,
};

layout.monitor_panel = ${
    layout_type = :row,
};

layout.monitor_column = ${
    layout_type = :column,
    left = :stretch => 1,
    right = :stretch => 1,
};

layout.dialog_popup_button_bar = ${
    layout_type = :row,
    height = :pixels => 40,
};

style.scope_panel = ${
    parent = :panel,
};
layout.scope_panel = ${
    parent = :panel,
    layout_type = :column,
    height = :pixels => 300,
};

style.scope = ${
    bg_color = ui:UI_LBL_BG_CLR,
    border_color = ui:UI_ACCENT_CLR,
    color = ui:STD_COLORS.11,
    font_size = 12,
    graph = ${
        graph_line = 2.0,
        vline1 = 2.0,
        vline2 = 2.0,
        hline = 1.5,
        vline1_color = ui:STD_COLORS.2,
        vline2_color = ui:STD_COLORS.7,
        hline_color = ui:STD_COLORS.13,
    },
};

style.blockcode = ${
    bg_color     = ui:UI_ACCENT_BG1_CLR,
    border_color = ui:UI_SELECT_CLR,
    blockcode = ${
        with_markers = $false,
        grid_marker_color = ui:UI_ACCENT_DARK_CLR,
        block_bg_hover_color = ui:UI_ACCENT_CLR,
        block_bg_color = ui:UI_ACCENT_BG2_CLR,
        port_select_color = ui:UI_SELECT_CLR,
    }
};
layout.blockcode = ${
    visible = $f,
};

!apply_class = $n;
.apply_class = {!(class, style_map, layout_map, set_ctrl) = @;
    !st = style.(class);

    if is_some[st] {
        if is_some[st.parent] {
            apply_class st.parent style_map $n set_ctrl;
        };

        iter kv st {
            if kv.1 == "parent" { next[]; };
            if kv.1 == "typ" {
                match kv.0
                    :rect => { .*set_ctrl = $[:rect, $n] };
                next[];
            };

            style_map.(kv.1) = kv.0;
        };
    };

    if is_some[layout.(class)] {
        !ly = layout.(class);
        if is_some[ly.parent] {
            apply_class ly.parent $n layout_map set_ctrl;
        };

        iter kv ly {
            if kv.1 == "parent" { next[]; };

            layout_map.(kv.1) = kv.0;
        };
    }
};

!default_style = ui:style[];

!new_widget = {
    !layout = ${};
    !style  = ${};
    !set_ctrl = $&& $n;
    iter class @ {
        apply_class class style layout set_ctrl;
    };

    !wid = ui:widget ~ default_style.clone_set style;
    if len[layout] > 0 {
        wid.change_layout layout;
    };

    if is_some[$*set_ctrl] {
        !(t, a) = $*set_ctrl;
        wid.set_ctrl t a;
    };

    wid.set_tag @.0;

    wid
};

!@export new_widget = new_widget;

!@export new_tagged_widget = {
    !wid = new_widget[[($p(1, -1) @)]];
    wid.set_tag @.0;
    wid
};

!@export new_rect = {
    !wid = new_widget[[@]];
    wid.set_ctrl :rect $n;
    wid
};

!@export new_button_with_label = {!(class, label, cb) = @;
    !wid = new_widget class;
    wid.set_ctrl :button (ui:txt label);
    wid.reg :click cb;
    wid
};

!@export restyle = {
    !wid = @.0;
    !layout = ${};
    !style  = ${};
    iter idx 1 => len[@] {
        iter class @ {
            apply_class class style layout $n;
        };
    };
    if len[style] > 0 {
        wid.set_style ~ default_style.clone_set style;
    };
    if len[layout] > 0 {
        wid.change_layout layout;
    };
};

!@export apply_color_idx_border = {!(wid, idx) = @;
    !st = wid.style[];
    st.set ${ border_color = ui:STD_COLORS.(idx) };
    wid.set_style st;
};
