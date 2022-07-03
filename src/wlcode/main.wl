!@wlambda;
!@import std;
!@import ui;
!@import hx;
!@import node_id;
!@import styling wllib:styling;
!@import editor wllib:editor;
!@import tests wllib:tests;

tests:install[];

!default_style = ui:style[];

!matrix = hx:get_main_matrix_handle[];

!editor = editor:EditorClass.new matrix;

editor.reg :set_focus {!(cell) = @;
    std:displayln "SET FOCUS:" cell;
    !info = node_id:info cell.node_id;
    std:displayln "INFO:" info;
    !plist = node_id:param_list cell.node_id;
    std:displayln "PARAMS:" plist;

};

!build_dsp_node_picker = {
    !parent = styling:new_widget :node_picker;
    parent.change_layout ${
        layout_type = :column,
    };

    !button_bar = styling:new_widget :button_bar;
    !stack_container = styling:new_widget :pick_node_bg_panel;
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

        !cat_node_page = styling:new_widget :cat_node_page;
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

        !row = styling:new_widget :pick_node_row;
        !row_count = 0;

        !drag_txt = ui:txt "?";
        !drag_btn = styling:new_widget :pick_node_drag_btn;
        drag_btn.set_ctrl :label drag_txt;
        drag_btn.set_pos $f(0, 0, 90, (2.0/3.0) * 90);

        iter node (cat cat_map) {
            if row_count >= 5 {
                cat_node_page.add row;
                .row = styling:new_widget :pick_node_row;
                .row_count = 0;
            };

            !node_id_widget = styling:new_widget :pick_node_btn;
            node_id_widget.set_ctrl :button (ui:txt ~ node_id:label node);
            node_id_widget.set_drag_widget drag_btn;
            !node_drag = node;
            node_id_widget.reg :drag {
                drag_txt.set ~ node_id:label node_drag;
                $[:node_type, ${ node = node_drag }]
            };
            node_id_widget.reg :hover {
                editor.handle_hover :node_picker node_drag;
            };
            row.add node_id_widget;
            .row_count += 1;
        };

        if row_count > 0 {
            iter i row_count => 5 {
                !dummy = styling:new_widget :pick_node_btn;
                row.add dummy;
            };
            cat_node_page.add row;
        };
    };

    all_pages.0.show[];
    styling:restyle all_tabs.0 :tab_hor :button_active;

    parent
};

!new_slide_panel = {!(class, child) = @;
    !slide_panel = styling:new_widget class;

    !slide_btn = styling:new_widget :close_hor_slide_panel_btn;
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

!setup_grid_widget = {!(matrix, click_cb) = @;
    !grid = styling:new_widget :matrix_grid;
    grid.set_ctrl :grid editor.get_grid_model[];

    grid.reg :click {
        std:displayln "GRID CLICK:" @;
        editor.set_focus_cell $i(@.1.x, @.1.y);
        click_cb[];
    };

    grid.reg :hex_drag {!(wid, ev) = @;
        !src = $i(ev.x_src, ev.y_src);
        !dst = $i(ev.x_dst, ev.y_dst);
        editor.handle_drag_gesture src dst ev.button;
    };

    grid.reg :drop_query {
        std:displayln "DROP QUERY:" @;
        $t
    };
    grid.reg :drop {!(wid, drop_data) = @;
        editor.place_new_instance_at
            drop_data.data.1.node
            $i(drop_data.x, drop_data.y);
    };

#    grid_panel.add grid;
#    grid_panel.add add_node_panel;

    grid
};

!root     = styling:new_widget :root;
!root_mid = styling:new_widget :root;
!popup_layer = styling:new_widget :popup_layer;

!new_misc_panel = {
    !panel = styling:new_rect :misc_panel;
    panel
};

!misc_panel = new_misc_panel[];
misc_panel.hide[];

!grid = setup_grid_widget matrix {
#    if misc_panel.is_visible[] { misc_panel.hide[]; } { misc_panel.show[] };
};
grid.change_layout ${
    height = :stretch => 1.0,
};

grid.add misc_panel;

!add_node_panel_inner = styling:new_widget :panel;
add_node_panel_inner.add ~ build_dsp_node_picker[];

!add_node_panel =
    new_slide_panel
        :picker_slide_panel
        add_node_panel_inner;

!left_panel_dummy = styling:new_widget :main_panel;
root_mid.add left_panel_dummy;
root_mid.add add_node_panel;


!left_panel = styling:new_widget :main_panel;

!param_panel = styling:new_widget :param_panel;
param_panel.set_ctrl :rect $n;
param_panel.change_layout ${
    height     = :stretch => 2.0,
    min_height = :pixels => 300,
};

!text_panel = styling:new_widget :help_text_panel;
text_panel.set_ctrl :rect $n;
text_panel.change_layout ${
    height     = :stretch => 1.0,
    min_height = :pixels => 200,
};

!wt = styling:new_widget :wichtext;
!wtd = ui:wichtext_simple_data_store[];
wt.set_ctrl :wichtext wtd;

editor.reg :update_status_help_text {!(new_text) = @;
    wtd.set_text new_text;
};

text_panel.add wt;

!signal_panel = styling:new_widget :signal_panel;
signal_panel.set_ctrl :rect $n;
signal_panel.change_layout ${
    height     = :stretch => 1.0,
    min_height = :pixels => 300,
};

!connector_popup = styling:new_widget :panel;

!con = styling:new_widget :connector;
!con_data = ui:connector_data[];
con.set_ctrl :connector con_data;
!connect_cb = $n;
con.reg :change {
    std:displayln "NEW CON:" con_data.get_connection[];
    connector_popup.hide[];
    connect_cb con_data.get_connection[];
};

connector_popup.set_ctrl :rect $n;
connector_popup.auto_hide[];
connector_popup.add con;

!clear_con = styling:new_button_with_label :button_label "Clear" {
    connector_popup.hide[];
    connect_cb $n;
};
connector_popup.add clear_con;

connector_popup.change_layout ${
    position_type = :self,
    layout_type   = :column,
    height        = :pixels => 300,
    width         = :pixels => 300,
    visible       = $f,
};

!mode_selector_popup = styling:new_widget :mode_selector_popup;
mode_selector_popup.auto_hide[];

popup_layer.add connector_popup;
popup_layer.add mode_selector_popup;

!create_mode_button = {!(val_list, init_idx, change_cb, hover_cb) = @;
    !val_idx = init_idx;
    !val_lbl = ui:txt "";

    !value_inc = {
        .val_idx += 1;
        .val_idx = val_idx % len[val_list];
        val_lbl.set val_list.(val_idx).0;
        change_cb val_idx val_list.(val_idx);
    };
    !value_dec = {
        .val_idx -= 1;
        if val_idx < 0 { .val_idx = len[val_list] - 1; };
        val_lbl.set val_list.(val_idx).0;
        change_cb val_idx val_list.(val_idx);
    };
    !value_set = {
        .val_idx = _;
        val_lbl.set val_list.(val_idx).0;
        change_cb val_idx val_list.(val_idx);
    };
    value_set val_idx;

    !mode_cont = styling:new_widget :mode_btn_cont;

    !mode_less_btn = styling:new_widget :mode_button_less;
    mode_less_btn.set_ctrl :button (ui:txt "-");
    mode_less_btn.reg :click value_dec;
    mode_less_btn.reg :hover hover_cb;

    !mode_more_btn = styling:new_widget :mode_button_more;
    mode_more_btn.set_ctrl :button (ui:txt "+");
    mode_more_btn.reg :click value_inc;
    mode_more_btn.reg :hover hover_cb;

    !mode_btn = styling:new_widget :mode_button;
    mode_btn.set_ctrl :button val_lbl;
    mode_btn.reg :hover hover_cb;

    mode_btn.reg :click {
        mode_selector_popup.remove_childs[];

        !i = 0;
        iter v val_list {
            !btn = styling:new_widget :mode_selector_item;
            btn.set_ctrl :button (ui:txt v.0);

            !idx = i;
            btn.reg :click {
                value_set idx; mode_selector_popup.hide[];
            };

            mode_selector_popup.add btn;
            .i += 1;
        };

        mode_selector_popup.popup_at_mouse[];
    };

    mode_cont.add mode_more_btn;
    mode_cont.add mode_btn;
    mode_cont.add mode_less_btn;

    mode_cont
};

editor.reg :setup_edit_connection {
    !(src_cell, dst_cell,
      output_port_list,
      input_port_list,
      con, con_cb) = @;

    .connect_cb = con_cb;

    std:displayln "INPUT LIST:" input_port_list;
    std:displayln "oUTP LIST:" output_port_list;

    con_data.clear[];
    iter out output_port_list {
        con_data.add_output out $t;
    };

    iter inp input_port_list {
        con_data.add_input inp.0 inp.1;
    };

    if is_some[con] {
        con_data.set_connection con;
    } {
        con_data.clear_connection[];
    };

    connector_popup.popup_at_mouse[];
    std:displayln "SETUP EDIT CON:" con;
};

editor.reg :update_param_ui {
    param_panel.remove_childs[];
    !plist = editor.get_current_param_list[];

    !knob_row = styling:new_widget :knob_row;
    !extra_widgets = $[];

    !grph_model = editor.get_current_graph_fun[];
    if is_some[grph_model] {
        !grph = styling:new_widget :node_graph;
        grph.set_ctrl :graph $[128, $f, grph_model];
        std:push extra_widgets grph
    };

    !row_fill = 0;

    iter input_param plist.inputs {
        if row_fill > 4 {
            param_panel.add knob_row;
            .knob_row = styling:new_widget :knob_row;
            .row_fill = 0;
        };

        !cont = styling:new_widget :param_container;

        !knob = styling:new_widget :knob;
        !knob_model = matrix.create_hex_knob_model input_param;
        knob.set_ctrl :knob knob_model;
        !in_param = input_param;
        knob.reg :hover {
            editor.handle_hover :param_knob in_param;
        };

        !lbl = styling:new_widget :knob_label;
        lbl.set_ctrl :label (ui:txt input_param.name[]);

        cont.add knob;
        cont.add lbl;
        knob_row.add cont;
        .row_fill += 1;
    };

    iter atom plist.atoms {
        !cur_atom = atom;
        !cont = styling:new_widget :param_container;

        std:displayln "FOFO" atom atom.atom_ui[];
        !wid =
            match atom.atom_ui[]
                :mode => {
                    !(min, max) = atom.setting_min_max[];
                    !init = (matrix.get_param atom).i[];
                    !list = $@vec
                        iter s min => (max + 1) {
                            $+ $p((atom.format s), s);
                        };
                    !my_atom = atom;
                    create_mode_button list init {
                        matrix.set_param cur_atom _1.1;
                    } {
                        editor.handle_hover :param_knob my_atom;
                    };
                }
                :keys => {
                    !wid = styling:new_widget :keys;
                    !model = matrix.create_octave_keys_model atom;
                    wid.set_ctrl :octave_keys model;
                    !my_atom = atom;
                    wid.reg :hover {
                        editor.handle_hover :param_knob my_atom;
                    };
                    std:push extra_widgets wid;
                    $n
                }
                {
                    !wid = styling:new_widget :atom_wid;
                    wid.set_ctrl :label (ui:txt atom.atom_ui[]);
                    wid
                };

        if is_none[wid] {
            next[];
        };


        !lbl = styling:new_widget :knob_label;
        lbl.set_ctrl :label (ui:txt atom.name[]);

        cont.add wid;
        cont.add lbl;
        knob_row.add cont;
        .row_fill += 1;
    };

    if row_fill > 0 {
        while row_fill < 5 {
            !cont = styling:new_widget :param_container;
            knob_row.add cont;
            .row_fill += 1;
        };
        param_panel.add knob_row;
    };

    iter exwid extra_widgets {
        std:displayln "ADD:" exwid;
        param_panel.add exwid;
    };
};

#param_panel.change_layout ${
#    left = :pixels => 0,
#    right = :pixels => 0,
#};

left_panel.add param_panel;
left_panel.add text_panel;
left_panel.add signal_panel;

!color_btn = styling:new_button_with_label :button_label "Colors" {
    editor.show_color_info[];
};
signal_panel.add color_btn;
!save_btn = styling:new_button_with_label :button_label "Save Init" {
    matrix.save_patch "init.hxy";
};
signal_panel.add save_btn;
!load_btn = styling:new_button_with_label :button_label "Load Init" {
    matrix.load_patch "init.hxy";
};
signal_panel.add load_btn;

root.add left_panel;
root.add grid;

!@export on_frame = {!(matrix_records) = @;
    iter r matrix_records {
        std:displayln "REC:" r;
        match r
            $p(:matrix_graph, _?) => {
                editor.handle_matrix_graph_change[];
                std:displayln "GRAPH UPDATE";
            };
    };
};

!@export root = $[root, root_mid, popup_layer];
