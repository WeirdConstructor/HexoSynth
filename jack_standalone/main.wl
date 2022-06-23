!@import ui;
!@import hx;
!@import node_id;
!@import styling wllib:styling;

!:global loaded_tests = $[
    $[
        { std:displayln "STEP 1" @; },
        { std:displayln "STEP 2" },
    ],
    $[
        { std:displayln "XXX STEP 1" },
        { std:displayln "XXX STEP 2" },
    ],
];

!default_style = ui:style[];

!set_button_active_colors = {!(style_map) = @;
    style_map.bg_color     = ui:UI_LBL_BG_CLR;
    style_map.border_color = ui:UI_HLIGHT_CLR;
    style_map
};
!set_button_colors = {!(style_map) = @;
    style_map.bg_color     = ui:UI_LBL_BG_CLR;
    style_map.border_color = ui:UI_ACCENT_CLR;
    style_map
};

!set_panel_colors = {!(style_map) = @;
    style_map.bg_color     = ui:UI_BG3_CLR;
    style_map.border_color = ui:UI_BORDER_CLR;
    style_map
};

!lbl = ui:txt "Test123";

!matrix = hx:get_main_matrix_handle[];

!build_dsp_node_picker = {!(style) = @;
    !parent = ui:widget style;
    parent.change_layout ${
        layout_type = :column,
    };

    !button_bar = ui:widget style;
    button_bar.change_layout ${
        layout_type = :row,
        height = :pixels => 30,
    };
    !stack_container = ui:widget ~ style.clone_set ${
        border = 1,
        border_color = ui:UI_ACCENT_CLR,
        bg_color     = ui:UI_BG3_CLR,
    };
    stack_container.set_ctrl :rect $n;

    parent.add button_bar;
    parent.add stack_container;

    !cat_map = node_id:ui_category_node_id_map[];
    std:displayln cat_map;
    !all_pages = $[];
    !all_tabs  = $[];
    iter cat node_id:ui_category_list[] {
        if cat == :none {
            next[];
        };

        !tab_btn = styling:new_widget :tab_hor;
        tab_btn.set_ctrl :button (ui:txt cat);
        button_bar.add tab_btn;
        std:push all_tabs tab_btn;

        !cat_node_page = ui:widget style;
        cat_node_page.hide[];
        cat_node_page.change_layout ${
            layout_type = :column,
        };
        tab_btn.reg :click {!(wid) = @;
            iter pg all_pages { pg.hide[] };
            iter bt all_tabs { styling:restyle bt :tab_hor };
            styling:restyle wid :tab_hor :button_active;
            cat_node_page.show[];
        };
        std:push all_pages cat_node_page;
        stack_container.add cat_node_page;

        !row_style = style.clone_set ${
            pad_top = 2,
            pad_bottom = 6,
        };
        !row = ui:widget row_style;
        !row_layout = ${
            layout_type = :row,
            height = :percent => 20,
        };
        row.change_layout row_layout;
        !row_count = 0;

        !btn_style = style.clone_set ~ set_button_colors ${
            border_style = $[:hex, 10],
            border = 2,
            shadow_offs = $f(3, 3),
        };

        !btn_layout = ${
            left   = :pixels => 5,
            right  = :pixels => 5,
            width  = :stretch => 1,
            height = :percent => 100,
        };

        !drag_txt = ui:txt "?";
        !drag_btn = ui:widget ~ btn_style.clone_set ${
            border_style = $[:hex, 20],
        };
        drag_btn.set_ctrl :label drag_txt;
        drag_btn.set_pos $f(0, 0, 90, (2.0/3.0) * 90);

        iter node (cat cat_map) {
            if row_count >= 5 {
                cat_node_page.add row;
                .row = ui:widget row_style;
                row.change_layout row_layout;
                .row_count = 0;
            };

            !node_id_widget = ui:widget btn_style;
            node_id_widget.set_ctrl :button (ui:txt ~ node_id:label node);
            node_id_widget.change_layout btn_layout;
            node_id_widget.set_drag_widget drag_btn;
            !node_drag_lbl = node.0;
            node_id_widget.reg :drag {
                drag_txt.set node_drag_lbl;
                $[:node_type, ${ node = node_drag_lbl }]
            };
            row.add node_id_widget;
            .row_count += 1;
        };

        if row_count > 0 {
            iter i row_count => 5 {
                !dummy = ui:widget style;
                row.add dummy;
                dummy.change_layout btn_layout;
            };
            cat_node_page.add row;
        };
    };

    all_pages.0.show[];
    styling:restyle all_tabs.0 :tab_hor :button_active;

    parent
};

!new_slide_panel = {!(style, panel_layout, child) = @;
    !slide_panel = ui:widget style;
    slide_panel.change_layout panel_layout;
    slide_panel.change_layout ${
        layout_type = :row,
    };

    !close_btn_style = style.clone_set ${
        border = 2,
        border_style = $[:bevel, $f(0, 10, 0, 10)],
        border_color = ui:UI_ACCENT_CLR,
        bg_color     = ui:UI_BG2_CLR,
    };
    !slide_btn = ui:widget close_btn_style;
    slide_btn.change_layout ${
        width  = :pixels => 30,
        top    = :stretch => 1,
        bottom = :stretch => 1,
        height = :percent => 25,
        left   = :pixels => -2,
    };
    !close_btn_text = ui:txt "<";
    slide_btn.set_ctrl :button close_btn_text;
    slide_btn.reg :click {
        if child.is_visible[] {
            child.hide[];
            close_btn_text.set ">";
        } {
            child.show[];
            close_btn_text.set "<";
        };
    };

    slide_panel.add child;
    slide_panel.add slide_btn;

    slide_panel
};

!setup_grid_widget = {!(style, matrix, click_cb) = @;
    !grid_model = matrix.create_grid_model[];
    !grid = ui:widget style;
    grid.set_ctrl :grid grid_model;

    grid.reg :click {
        std:displayln "GRID CLICK:" @;
        grid_model.set_focus_cell $i(@.1.x, @.1.y);
        click_cb[];
    };

    grid.reg :hex_drag {
        std:displayln "GRID DRAG:" @;
    };

    grid.reg :drop_query {
        std:displayln "DROP QUERY:" @;
        $t
    };
    grid.reg :drop {!(wid, drop_data) = @;
        !pos = $i(drop_data.x, drop_data.y);
        !cell = matrix.get pos;
        !new_node_id =
            matrix.get_unused_instance_node_id
                $p(drop_data.data.1.node, 0);
        cell.node_id = new_node_id;
        matrix.set pos cell;
        matrix.sync[];
    };

    grid.change_layout ${
        position_type = :self,
    };

    !panel_style = style.clone_set ${
        border       = 2,
        border_style = $[:rect],
    };

    !add_node_panel_inner = ui:widget panel_style;
    add_node_panel_inner.add ~ build_dsp_node_picker style;

    !slide_panel_layout = ${
        top    = :stretch => 1.0,
        width  = :percent => 60,
        height = :pixels  => 200,
        min_width = :pixels => 400,
    };
    !add_node_panel =
        new_slide_panel
            style
            slide_panel_layout
            add_node_panel_inner;

    !grid_panel = ui:widget style;

    grid_panel.add grid;
    grid_panel.add add_node_panel;

    grid_panel
};

!root = ui:widget default_style;

root.change_layout ${ layout_type = :row };

!new_misc_panel = {!(style) = @;
    !panel = ui:widget style;
    panel.set_ctrl :rect $n;

    panel.change_layout ${
        position_type = :self,
        left = :stretch => 1.0,
        width = :percent => 30,
        min_width = :pixels => 200,
    };

    panel
};

!misc_panel = new_misc_panel default_style;

!grid = setup_grid_widget default_style matrix {
    if misc_panel.is_visible[] { misc_panel.hide[]; } { misc_panel.show[] };
};
grid.change_layout ${
    height = :stretch => 1.0,
};

grid.add misc_panel;

!left_panel = ui:widget default_style;
left_panel.change_layout ${
    layout_type = :column,
    width       = :percent => 30,
    min_width   = :pixels  => 300
};
#left_panel.set_ctrl :rect $n;


!param_panel = ui:widget ~ default_style.clone_set ${ };
param_panel.set_ctrl :rect $n;
param_panel.change_layout ${
    height     = :stretch => 2.0,
    min_height = :pixels => 300,
};
!text_panel = ui:widget ~ default_style.clone_set ${ };
text_panel.set_ctrl :rect $n;
text_panel.change_layout ${
    height     = :stretch => 1.0,
    min_height = :pixels => 200,
};

!signal_panel = ui:widget ~ default_style.clone_set ${ };
signal_panel.set_ctrl :rect $n;
signal_panel.change_layout ${
    height     = :stretch => 1.0,
    min_height = :pixels => 300,
};
#param_panel.change_layout ${
#    left = :pixels => 0,
#    right = :pixels => 0,
#};

left_panel.add param_panel;
left_panel.add text_panel;
left_panel.add signal_panel;

root.add left_panel;
root.add grid;

#style.set ${
#    bg_color   = std:v:hex2rgba_f "222",
#    color      = $f(1.0, 1.0, 0.0),
#    font_size  = 24,
#    text_align = :left,
#    pad_left   = 20,
#    border_style = $[:bevel, $f(5.0, 10.0, 20.0, 2.0)],
#};
#
#!btn = ui:widget style;
#btn.set_ctrl :button lbl;
#
#!btn2 = ui:widget style;
#btn2.set_ctrl :button (ui:txt "wurst");
#
#!matrix = hx:get_main_matrix_handle[];
#!freq_s = 440.0;
#!sin_freq = node_id:inp_param $p(:sin, 0) :freq;
#lbl.set ~ sin_freq.format ~ sin_freq.norm freq_s;
#
#btn.reg :click {
#    std:displayln "CLICK!" @;
#    lbl.set ~ sin_freq.format ~ sin_freq.norm freq_s;
#    matrix.set_param sin_freq sin_freq.norm[freq_s];
#    matrix.set_param $p($p(:amp, 0), :att) 1.0;
#    root.remove_child btn2;
#    root.change_layout ${
#        layout_type = :row
#    };
#    .freq_s *= 1.25;
#};
#
#!ent = ui:widget style;
##ent.set_ctrl :entry lbl;
#
##ent.reg :changed {
##    std:displayln "CHANGED" @;
##};
#
#std:displayln "FOO";
#
#iter y 0 => 10 {
#    iter x 0 => 10 {
#        std:displayln ~ matrix.get $i(x, y);
#    };
#};
#
#matrix.set_param $p($p(:amp, 0), :att) 0.0;
#std:displayln ~ node_id:param_list $p(:amp, 0);
#
#!knob_model = matrix.create_hex_knob_dummy_model[];
#!knob = ui:widget style;
#knob.set_ctrl :knob knob_model;
#
#!knob_freq_model = matrix.create_hex_knob_model sin_freq;
#!knob_freq = ui:widget style;
#knob_freq.set_ctrl :knob knob_freq_model;
#
#
#root.add btn;
#root.add ent;
#root.add knob;
#root.add knob_freq;
#root.add btn2;
#root.add grid;

$[root]
