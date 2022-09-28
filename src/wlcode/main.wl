!@wlambda;
!@import std;
!@import ui;
!@import hx;
!@import node_id;
!@import styling wllib:styling;
!@import editor wllib:editor;
!@import tests wllib:tests;
!@import texts wllib:texts;
!@import popup_debug_browser wllib:popup_debug_browser;

!IN_TEST_MODE = $false;
!GLOBAL_CLICK_CB = $none;
!LAST_LABELS = $[];

!@export init = {
    !test = std:sys:env:var "HEXOSYNTH_TEST";
    if not[is_err[test]] &and is_some[test] {
        .IN_TEST_MODE = $true;
        !test_match = if test == "1" { $n } { test };
        tests:install test_match;
    }
};

!default_style = ui:style[];

!matrix = hx:get_main_matrix_handle[];

!editor = editor:EditorClass.new matrix;

#editor.reg :set_focus {!(cell) = @;
#    !info = node_id:info cell.node_id;
#    !plist = node_id:param_list cell.node_id;
#};

!build_dsp_node_picker = {
    !parent = styling:new_widget :node_picker;
    parent.change_layout ${
        layout_type = :column,
    };

    !button_bar = styling:new_widget :button_bar;

    !picker_btn_bar = styling:new_widget :picker_btn_bar;
    !picker_help_btn = styling:new_button_with_label :help_btn "?" {
        editor.handle_picker_help_btn[];
    };
    picker_btn_bar.add picker_help_btn;

    !bg_panel = styling:new_widget :pick_node_bg_panel;
    bg_panel.set_ctrl :rect $n;

    !stack_container = styling:new_widget :pick_node_bg_panel;

    bg_panel.add picker_btn_bar;
    bg_panel.add stack_container;

    parent.add button_bar;
    parent.add bg_panel;

    !cat_map = node_id:ui_category_node_id_map[];
    !all_pages = $[];
    !all_tabs  = $[];
    iter cat node_id:ui_category_list[] {
        !cat_name      = cat.0;
        !cat_color_idx = cat.1;
        if cat_name == :none {
            next[];
        };

        !tab_btn = styling:new_widget :tab_hor;
        styling:apply_color_idx_border tab_btn cat_color_idx;
        tab_btn.set_ctrl :button (ui:txt cat_name);

        button_bar.add tab_btn;
        std:push all_tabs $[tab_btn, tab_btn.style[]];

        !cat_node_page = styling:new_widget :cat_node_page;
        cat_node_page.hide[];
        cat_node_page.change_layout ${
            layout_type = :column,
        };
        tab_btn.reg :click {!(wid) = @;
            iter pg all_pages { pg.hide[] };
            iter bt all_tabs { bt.0.set_style bt.1 };
            styling:restyle wid :tab_hor :button_active;
            cat_node_page.show[];
        };
        std:push all_pages cat_node_page;
        stack_container.add cat_node_page;

        !row = styling:new_widget :pick_node_row;
        !row_count = 0;

        !drag_txt = ui:txt "?";
        !drag_btn = styling:new_widget :pick_node_drag_btn;
        styling:apply_color_idx_border drag_btn cat_color_idx;
        drag_btn.set_ctrl :label drag_txt;
        drag_btn.set_pos $f(0, 0, 90, (2.0/3.0) * 90);

        iter node (cat_name cat_map) {
            if row_count >= 5 {
                cat_node_page.add row;
                .row = styling:new_widget :pick_node_row;
                .row_count = 0;
            };

            !node_id_widget = styling:new_widget :pick_node_btn;
            styling:apply_color_idx_border node_id_widget cat_color_idx;
            node_id_widget.set_ctrl :button (ui:txt ~ node_id:label node);
            node_id_widget.set_drag_widget drag_btn;
            !node_drag = node;
            node_id_widget.reg :drag {
                drag_txt.set ~ node_id:label node_drag;
                $[:node_type, ${ node = node_drag }]
            };
            node_id_widget.reg :click {!(wid, ev) = @;
                if ev == :left &or ev == :right {
                    editor.handle_picker_node_id_click node_drag ev;
                };
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
    styling:restyle all_tabs.0.0 :tab_hor :button_active;

    parent.enable_cache[];

    parent
};

!new_slide_panel = {!(class, dir, child) = @;
    !slide_panel = styling:new_widget class;

    !close_btn_class =
        if dir == :left
            { :close_hor_slide_left_panel_btn }
            { :close_hor_slide_right_panel_btn };

    !(open_txt, closed_txt) =
        if dir == :left
            { $p("<", ">") }
            { $p(">", "<") };

    !slide_btn = styling:new_widget close_btn_class;
    !close_btn_text = ui:txt open_txt;
    slide_btn.set_ctrl :button close_btn_text;
    slide_btn.reg :click {
        if child.is_visible[] {
            child.hide[];
            close_btn_text.set closed_txt;
        } {
            child.show[];
            close_btn_text.set open_txt;
        };
    };

    if dir == :left {
        slide_panel.add child;
        slide_panel.add slide_btn;
    } {
        slide_panel.add slide_btn;
        slide_panel.add child;
    };

    slide_panel
};

!new_param_container = {!(wid, label_text) = @;
    !cont = styling:new_widget :param_container;
    !lbl = styling:new_widget :param_label;
    lbl.set_ctrl :label (ui:txt label_text);

    cont.add wid;
    cont.add lbl;
    cont
};

!new_hex_knob = {|2<3| !(model, label_text, on_hover) = @;
    !knob = styling:new_widget :knob;
    knob.set_ctrl :knob model;
    if is_some[on_hover] {
        knob.reg :hover on_hover;
    };
    new_param_container knob label_text
};

!cell_context_popup = styling:new_widget :popup_menu;
cell_context_popup.auto_hide[];

!add_context_menu_item = {!(menu, label, help, callback) = @;
    !btn = styling:new_widget :cell_context_item;
    btn.set_ctrl :button (ui:txt label);
    btn.reg :hover {
        editor.handle_hover :hover_help help;
    };
    btn.reg :click {
        callback[];
        menu.hide[];
    };

    menu.add btn;
};

iter ctx_item editor.get_cell_context_menu_items[] {
    !ctx_item = ctx_item;

    if is_none[ctx_item.2] {
        panic ~ $F"Missing help for context menu item: {}" ctx_item.0;
    };

    add_context_menu_item cell_context_popup ctx_item.1 ctx_item.2 {
        editor.handle_context_menu_action ctx_item.0;
    };
};

!matrix_context_popup = styling:new_widget :popup_menu;
matrix_context_popup.auto_hide[];

iter ctx_item editor.get_matrix_context_menu_items[] {
    !ctx_item = ctx_item;

    if is_none[ctx_item.2] {
        panic ~ $F"Missing help for context menu item: {}" ctx_item.0;
    };

    add_context_menu_item matrix_context_popup ctx_item.1 ctx_item.2 {
        editor.handle_context_menu_action ctx_item.0;
    };
};


!setup_grid_widget = {!(matrix) = @;
    !grid = styling:new_widget :matrix_grid;
    grid.set_ctrl :grid editor.get_grid_model[];

    grid.reg :click {!(widget, event) = @;
        match event.button
            :left => {
                editor.set_focus_cell $i(@.1.x, @.1.y);
            }
            :right => {
                editor.set_context_cell_pos $i(event.x, event.y);
                if editor.context_cell_is_empty[] {
                    matrix_context_popup.popup_at_mouse[];
                } {
                    cell_context_popup.popup_at_mouse[];
                };
            };
    };

    grid.reg :center_pos {!(wid, ev) = @;
        editor.set_grid_center $i(ev.x, ev.y);
    };

    grid.reg :hex_drag {!(wid, ev) = @;
        !src = $i(ev.x_src, ev.y_src);
        !dst = $i(ev.x_dst, ev.y_dst);
        editor.handle_drag_gesture src dst ev.button;
    };

    grid.reg :drop_query {
        $t
    };
    grid.reg :drop {!(wid, drop_data) = @;
        editor.spawn_new_instance_at
            drop_data.data.1.node
            $i(drop_data.x, drop_data.y);
    };

    grid
};

!root     = styling:new_widget :root;
!root_mid = styling:new_widget :root;
!popup_layer = styling:new_widget :popup_layer;

!grid = setup_grid_widget matrix;

!app_panel = styling:new_widget :app_panel;

app_panel.change_layout ${
    height = :stretch => 1.0,
};

!CODE_SIZES = $[0, 25, 50, 75];
!CODE_SIZE_IDX = 1;

!blockcode = styling:new_widget :blockcode;

!on_code_menu_toggle = {
    !size_perc = CODE_SIZES.(CODE_SIZE_IDX);
    .CODE_SIZE_IDX += 1;
    .CODE_SIZE_IDX %= len CODE_SIZES;

    if size_perc > 0 {
        blockcode.change_layout ${
            visible = $t,
            height = :percent => size_perc,
        };
    } {
        blockcode.change_layout ${
            visible = $f,
        };
    };
};

!fun = matrix.get_block_function 0;
blockcode.set_ctrl :blockcode fun;

!blockcode_picker_popup = styling:new_widget :blockcode_picker_popup;
blockcode_picker_popup.change_layout ${
    position_type = :self,
    width         = :pixels => 700,
    height        = :pixels => 300,
    top           = :stretch => 1,
    bottom        = :stretch => 1,
    left          = :stretch => 1,
    right         = :stretch => 1,
    visible       = $f,
};
blockcode_picker_popup.auto_hide[];
blockcode_picker_popup.set_ctrl :rect $n;

!blockcode_context_popup = styling:new_widget :blockcode_picker_popup;
blockcode_context_popup.change_layout ${
    position_type = :self,
    width         = :pixels => 700,
    height        = :pixels => 300,
    top           = :stretch => 1,
    bottom        = :stretch => 1,
    left          = :stretch => 1,
    right         = :stretch => 1,
    visible       = $f,
};
blockcode_context_popup.auto_hide[];
blockcode_context_popup.set_ctrl :rect $n;

!entry_popup = styling:new_widget :entry_popup;
entry_popup.change_layout ${
    position_type = :self,
    width         = :pixels => 100,
    height        = :pixels => 40,
    top           = :stretch => 1,
    bottom        = :stretch => 1,
    left          = :stretch => 1,
    right         = :stretch => 1,
    visible       = $f,
};
entry_popup.auto_hide[];
entry_popup.set_ctrl :rect $n;

!value_entry = styling:new_widget :value_entry;
!value_tf = ui:txt_field[];
value_tf.set "?";
value_entry.set_ctrl :entry value_tf;
entry_popup.add value_entry;

!ENTRY_ACTION = $n;

value_entry.reg :enter {!(wid, ev) = @;
    std:displayln "ENTER:" @;
    if is_some[ENTRY_ACTION] {
        ENTRY_ACTION[ev];
    };
    entry_popup.hide[];
};

!popup_entry = {!(cb) = @;
    entry_popup.show[];
};

!lang = fun.language[];
#d# std:displayln lang.get_type_list[];

!blockcode_click_pos = $n;

iter typ
    (std:sort { std:cmp:str:asc (_.category "/" _.name) (_1.category "/" _1.name) }
        lang.get_type_list[]) {
    !typ = typ;
    !bc_pick_btn = styling:new_widget :blockcode_pick_btn;
    bc_pick_btn.set_ctrl :button (ui:txt ~ $F"({}) {}" ($p(0, 4) typ.category) typ.name);
    bc_pick_btn.reg :click {
        if typ.user_input == :identifier &or typ.user_input == :float {
            .ENTRY_ACTION = {!(txt) = @;
                fun.instanciate_at
                    blockcode_click_pos.id
                    $i(blockcode_click_pos.x, blockcode_click_pos.y)
                    typ.name
                    txt;
                fun.recalculate_area_sizes[];
            };
            blockcode_picker_popup.hide[];
            value_tf.set "";
            entry_popup.popup_at_mouse_offs $f(-50, -20);
            value_entry.activate[];
        } {
            fun.instanciate_at
                blockcode_click_pos.id
                $i(blockcode_click_pos.x, blockcode_click_pos.y)
                typ.name
                $n;
            fun.recalculate_area_sizes[];
            blockcode_picker_popup.hide[];
            std:displayln "PICK:" typ;
        }
    };
    blockcode_picker_popup.add bc_pick_btn;
};

!bc_context_actions = $[
    $p("split ->", {
        fun.split_block_chain_after
            blockcode_click_pos.id
            blockcode_click_pos.x
            blockcode_click_pos.y
            "->";
    }),
    $p("split", {
        fun.split_block_chain_after
            blockcode_click_pos.id
            blockcode_click_pos.x
            blockcode_click_pos.y
            $n;
    }),
    $p("delete", {
        fun.remove_at
            blockcode_click_pos.id
            blockcode_click_pos.x
            blockcode_click_pos.y;
        fun.recalculate_area_sizes[];
    }),
];

iter action bc_context_actions {
    !(label, cb) = action;

    !bc_del_btn = styling:new_widget :blockcode_pick_btn;
    bc_del_btn.set_ctrl :button (ui:txt label);
    bc_del_btn.reg :click {
        cb[];
        blockcode_context_popup.hide[];
    };
    blockcode_context_popup.add bc_del_btn;
};

blockcode.reg :click {!(wid, pos) = @;
    std:displayln "POSOSO" pos;
    .blockcode_click_pos = pos.at;
    if is_none[pos.at.row] {
        blockcode_picker_popup.popup_at_mouse[];
    } {
        if pos.btn == :right {
            fun.shift_port
                pos.at.id pos.at.x pos.at.y pos.at.row pos.at.col == 1;
        } {
            blockcode_context_popup.popup_at_mouse[];
        };
    };
};

blockcode.reg :drag {!(wid, pos) = @;
    std:displayln "dRAG" pos;
    if is_none[pos.to] {
    } { # has destination:
        if is_some[pos.at.row] {
            if is_none[pos.to.row] {
                if pos.btn == :left {
                    fun.move_block_chain_from_to
                        pos.at.id pos.at.x pos.at.y
                        pos.to.id pos.to.x pos.to.y;
                } {
                    fun.move_block_from_to
                        pos.at.id pos.at.x pos.at.y
                        pos.to.id pos.to.x pos.to.y;
                };
            }
        } {
            if is_some[pos.to.row] {
                fun.clone_block_from_to
                    pos.to.id pos.to.x pos.to.y
                    pos.at.id pos.at.x pos.at.y;
            } {
                if pos.btn == :left {
                    fun.instanciate_at
                        pos.to.id
                        $i(pos.to.x, pos.to.y)
                        "->"
                        $n;
                };
            };
        };
    };

    fun.recalculate_area_sizes[];
};

app_panel.add grid;
app_panel.add blockcode;

!add_node_panel_inner = styling:new_widget :panel;
add_node_panel_inner.add ~ build_dsp_node_picker[];

!add_node_panel =
    new_slide_panel
        :picker_slide_panel
        :left
        add_node_panel_inner;

!left_panel_dummy = styling:new_widget :main_panel;
root_mid.add left_panel_dummy;

!right_container = styling:new_widget :right_mid_cont;
right_container.add add_node_panel;

!top_menu_button_bar = styling:new_widget :top_float_menu;
top_menu_button_bar.enable_cache[];

!top_menu_actions = $[
    $["Help", texts:top_menu_texts.help, :help],
    $["About", texts:top_menu_texts.about, :about],
    $["MIDI", texts:top_menu_texts.midi, :midi],
    $["Save", texts:top_menu_texts.save, :save],
    $["Load", texts:top_menu_texts.load, :load],
    $["Demo", texts:top_menu_texts.demo, :init],
    $["Code", texts:top_menu_texts.code, on_code_menu_toggle],
    $["_C", texts:top_menu_texts.colors, {
        editor.show_color_info[];
        if IN_TEST_MODE {
            .GLOBAL_CLICK_CB = {!(ev) = @;
                !found = $false;
                !position_labels = $@vec iter lbl LAST_LABELS {
                    if ev.x >= lbl.wid_pos.x
                       &and ev.y >= lbl.wid_pos.y
                       &and ev.x <= (lbl.wid_pos.x + lbl.wid_pos.2)
                       &and ev.y <= (lbl.wid_pos.y + lbl.wid_pos.3) {
                        !lblcopy = ${};
                        iter kv lbl {
                            if kv.1 != "pos" &and kv.1 != "wid_pos" {
                                lblcopy.(kv.1) = kv.0;
                            }
                        };
                        $+ lblcopy;
                        .found = $true;
                    };
                };

                if found {
                    editor.emit :show_dbg_help position_labels;
                };

                .GLOBAL_CLICK_CB = $n;
            };
        }
    }],
];

iter action top_menu_actions {
    !(lbl, desc_text, editor_cmd) = action;

    !btn = styling:new_button_with_label :button_float_menu action.0 {
        if is_fun[editor_cmd] {
            editor_cmd[];
        } {
            editor.handle_top_menu_click editor_cmd;
        }
    };
    btn.reg :hover {
        editor.show_markdown_desc desc_text;
    };
    top_menu_button_bar.add btn;
};

!popup_test = $n;
!tmp_btn = styling:new_button_with_label :button_float_menu "Test" {
    popup_test.popup_at_mouse[];
};

top_menu_button_bar.add tmp_btn;

right_container.add top_menu_button_bar;


root_mid.add right_container;

!right_panel_container = styling:new_widget :right_util_panel_cont;
!right_panel =
    new_slide_panel
        :right_slide_panel
        :right
        right_panel_container;

#!tracker_help = styling:new_button_with_label :top_right_help_btn "?" {
#    editor.handle_tracker_help_btn[];
#};

!PanelTopLabel = ${
    new = {!(label) = @;
        ${
            _proto = $self,
            _data = ${
                txt = ui:txt label,
                styling = :right_panel_top_label,
                btn_styling = :top_right_help_btn,
            },
        }
    },
    get_widget = {
        if $data.widget {
            return $data.widget;
        };

        !widget = styling:new_widget $data.styling;
        widget.set_ctrl :label $data.txt;

        !_data = $data;
        !help_button = styling:new_button_with_label $data.btn_styling "?" {
            if _data.callback { _data.callback[]; }
        };
        widget.add help_button;

        $data.widget = widget;
        widget
    },
    set_help_callback = { $data.callback = _ },
    set_text = {!(txt) = @;
        $data.txt.set txt;
    },
};

!patedit = styling:new_widget :pattern_editor;

!patedit_lbl_obj = PanelTopLabel.new "TSeq 0";
patedit_lbl_obj.set_help_callback {
    editor.handle_tracker_help_btn[];
};

#!patedit_label = styling:new_widget :right_panel_top_label;
#!patedit_label_data = ui:txt "TSeq 0";
#patedit_label.set_ctrl :label patedit_label_data;
#
#patedit_label.add tracker_help;

!patdata = matrix.create_pattern_data_model 0;
!fbdummy = ui:create_pattern_feedback_dummy[];
patedit.set_ctrl :pattern_editor $[6, patdata, fbdummy];

editor.reg :pattern_editor_set_data {!(tracker_id, data) = _;
    patedit_lbl_obj.set_text ($F"TSeq {}" tracker_id);
    if is_none[data.2] {
        data.2 = ui:create_pattern_feedback_dummy[];
    };
    patedit.set_ctrl :pattern_editor data;
};
editor.set_active_tracker $p(:tseq, 0);

!patedit_container = styling:new_widget :pattern_editor_container;
patedit_container.add patedit_lbl_obj.get_widget[];
patedit_container.add patedit;
patedit_container.hide[];

!ext_param_container = styling:new_widget :ext_param_container;

!extparam_lbl_obj = PanelTopLabel.new "External Parameters";
extparam_lbl_obj.set_help_callback {
    editor.handle_ext_param_help_btn[];
};
ext_param_container.add extparam_lbl_obj.get_widget[];

ext_param_container.hide[];
ext_param_container.enable_cache[];

iter row $[
    $[
        $p(:A1, "ExtA1"),
        $p(:A2, "ExtA2"),
        $p(:A3, "ExtA3"),
        $p(:B1, "ExtB1"),
    ],
    $[
        $p(:C1, "ExtC1"),
        $p(:C2, "ExtC2"),
        $p(:C3, "ExtC3"),
        $p(:B2, "ExtB2"),
    ],
    $[
        $p(:D1, "ExtD1"),
        $p(:D2, "ExtD2"),
        $p(:D3, "ExtD3"),
        $p(:B3, "ExtB3"),
    ],
    $[
        $p(:E1, "ExtE1"),
        $p(:E2, "ExtE2"),
        $p(:E3, "ExtE3"),
    ],
    $[
        $p(:F1, "ExtF1"),
        $p(:F2, "ExtF2"),
        $p(:F3, "ExtF3"),
    ],
] {
    !knob_row = styling:new_widget :knob_row;
    iter knob_info row {
        knob_row.add ~ new_hex_knob (ui:create_ext_param_model knob_info.0) knob_info.1;
    };
#    knob_row.enable_cache[];
    ext_param_container.add knob_row;
};

!intro_help_container = styling:new_widget :intro_help_container;

!intro_help = styling:new_widget :intro_help_wichtext;
!wtd_intro_help = ui:wichtext_simple_data_store[];
wtd_intro_help.set_text ~ ui:mkd2wt texts:intro 50;
intro_help.set_ctrl :wichtext wtd_intro_help;
intro_help_container.add intro_help;
intro_help.reg "click" {!(wid, ev) = @;
    if is_some[ev.cmd $p(0, "Help")] {
        editor.show_help[];
    };
};

!right_pnl_button_bar = styling:new_widget :button_bar;
right_pnl_button_bar.add ~ styling:new_button_with_label :tab_hor "Seq" {
    patedit_container.show[];
    ext_param_container.hide[];
    intro_help_container.hide[];
};
right_pnl_button_bar.add ~ styling:new_button_with_label :tab_hor "Ext" {
    patedit_container.hide[];
    ext_param_container.show[];
    intro_help_container.hide[];
};
right_pnl_button_bar.add ~ styling:new_button_with_label :tab_hor "Intro" {
    patedit_container.hide[];
    ext_param_container.hide[];
    intro_help_container.show[];
};

right_panel_container.add right_pnl_button_bar;
right_panel_container.add patedit_container;
right_panel_container.add ext_param_container;
right_panel_container.add intro_help_container;

# TODO: Add toggle/tab for this:

!scope_handle = matrix.get_scope_handle $p(:scope, 0);

editor.reg :change_focus {!(cell) = @;
    if cell.node_id.0 == "scope" {
        scope_handle.set_node_id cell.node_id;
    };
};

!scope = styling:new_widget :scope;
scope.set_ctrl :scope $[scope_handle];

!scope_panel = styling:new_rect :scope_panel;
scope_panel.add scope;

!scope_size_big = $t;
!scope_size_btn = styling:new_widget :top_right_help_btn;
!scope_size_btn_lbl = ui:txt "-";
scope_size_btn.set_ctrl :button scope_size_btn_lbl;
scope_size_btn.reg :click {
    if scope_size_big {
        scope_size_btn_lbl.set "+";
        scope_panel.change_layout ${
            height = :pixels => 100,
        };
    } {
        scope_size_btn_lbl.set "-";
        scope_panel.change_layout ${
            height = :pixels => 300,
        };
    };
    .scope_size_big = not scope_size_big;
};

scope_panel.add scope_size_btn;

right_panel_container.add scope_panel;

root_mid.add right_panel;


!left_panel = styling:new_widget :main_panel;

!param_panel = styling:new_widget :param_panel;
param_panel.enable_cache[];
param_panel.set_ctrl :rect $n;
param_panel.change_layout ${
    height     = :stretch => 2.0,
    min_height = :pixels => 300,
};

!text_panel = styling:new_widget :help_text_panel;
text_panel.set_ctrl :rect $n;

!node_help_btn = styling:new_button_with_label :top_right_help_btn "?" {
    editor.handle_node_help_btn[];
};


!wt = styling:new_widget :wichtext;
!wtd = ui:wichtext_simple_data_store[];
wt.set_ctrl :wichtext wtd;

editor.reg :update_status_help_text {!(new_text) = @;
    wtd.set_text new_text;
};

wt.add node_help_btn;
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

!help_wichtext = styling:new_widget :main_help_wichtext;
!wtd_help = ui:wichtext_simple_data_store[];
wtd_help.set_text "Help!";
help_wichtext.set_ctrl :wichtext wtd_help;
help_wichtext.auto_hide[];

editor.reg :show_main_help {!(text) = @;
    wtd_help.set_text text;
    help_wichtext.show[];
};

!debug_panel = popup_debug_browser:DebugPanel.new[];
editor.reg :show_dbg_help {!(labels) = @;
    debug_panel.show labels;
};

!midi_log_wichtext = styling:new_widget :midi_log_wichtext;
!midi_log_text = ui:wichtext_simple_data_store[];
midi_log_text.set_text "Midi Log!";
midi_log_wichtext.set_ctrl :wichtext midi_log_text;
midi_log_wichtext.auto_hide[];

editor.reg :show_midi_log {
    midi_log_text.set_text ~ editor.get_midi_log_text[];
    midi_log_wichtext.show[];
};

editor.reg :update_midi_log {
    midi_log_text.set_text ~ editor.get_midi_log_text[];
};

!sample_list_popup = styling:new_widget :sample_list_popup;
sample_list_popup.change_layout ${
    position_type = :self,
    width         = :pixels  => 500,
    min_width     = :pixels  => 500,
    height        = :percent => 70,
    left          = :stretch => 1,
    right         = :stretch => 1,
    visible       = $f,
};
sample_list_popup.auto_hide[];
sample_list_popup.set_ctrl :rect $n;

!dialog_popup = styling:new_widget :dialog_popup;
dialog_popup.change_layout ${
    position_type = :self,
    width         = :pixels => 700,
    height        = :pixels => 300,
    top           = :stretch => 1,
    bottom        = :stretch => 1,
    left          = :stretch => 1,
    right         = :stretch => 1,
    visible       = $f,
};
dialog_popup.auto_hide[];
dialog_popup.set_ctrl :rect $n;

!dialog_wichtext = styling:new_widget :wichtext;
!dialog_wtd = ui:wichtext_simple_data_store[];
dialog_wichtext.set_ctrl :wichtext dialog_wtd;

editor.reg :dialog_query {!(mode, text, ok_cb) = @;
    dialog_wtd.set_text text;
    match mode
        :yes_cancel => {
            dialog_popup.remove_childs[];
            !row = styling:new_widget :dialog_popup_button_bar;
            !btn1 = styling:new_button_with_label :button_big "✔ Yes" {
                ok_cb[];
                dialog_popup.hide[];
            };
            !btn2 = styling:new_button_with_label :button_big "✘ Cancel" {
                dialog_popup.hide[];
            };
            row.add btn1;
            row.add btn2;
            dialog_popup.add dialog_wichtext;
            dialog_popup.add row;
            dialog_popup.show[];
        };
};

!DirIndexer = ${
    new = {#!() = @;
        ${
            _proto = $self,
            _data = ${ cur_dir = ".", },
        }
    },
    get_wav_files = {
        $@vec std:fs:read_dir $data.cur_dir {!(ent) = @;
            if ent.type == :f &and ent.name &> $r/*.wav$$/ {
                $+ ent.path
            };
            $f
        }
    },
    wichtext_file_list = {
        !paths = $self.get_wav_files[];
        $@string iter p paths {
            $+ "[a:" +> p +> "]";
            $+ "\n";
        }
    },
};

!WAV_INDEXER = DirIndexer.new[];

!wtd_slist = ui:wichtext_simple_data_store[];
wtd_slist.set_text WAV_INDEXER.wichtext_file_list[];

!current_sample_atom = $n;
!sample_list_wichtext = styling:new_widget :sample_list_wichtext;
sample_list_wichtext.set_ctrl :wichtext wtd_slist;
sample_list_wichtext.reg :click {!(wid, ev) = @;
#    std:displayln :click @;
    matrix.set_param current_sample_atom $p(:audio_sample, ev.cmd);
    sample_list_popup.hide[];
    editor.emit :update_param_ui;
};

sample_list_popup.add sample_list_wichtext;

editor.reg :open_sample_selector {!(param) = @;
    std:displayln param;
    .current_sample_atom = param;
    sample_list_popup.popup_at_mouse[];
};

##################################################################

!file_selector_popup = styling:new_widget :file_selector_popup;
file_selector_popup.auto_hide[];
file_selector_popup.set_ctrl :rect $n;

!FileSelector = ${
    new = {
        !root_dirs = hx:get_directories_samples[];

        ${
            _proto = $self,
            _data = ${
                file_type = :sample,
                root_dirs = root_dirs,
                cur_path = root_dirs.0,
                directories = $[],
                files = $[],
                file_list_data = ui:list_data[],
                dir_list_data = ui:list_data[],
                path_stack = $[],
            },
        }
    },
    build = {
        !grp = styling:new_widget :file_dialog;

        !dir_list = styling:new_widget :dir_list;
        dir_list.set_ctrl :list $data.dir_list_data;

        !file_list = styling:new_widget :file_list;
        file_list.set_ctrl :list_selector $data.file_list_data;

        grp.add dir_list;
        grp.add file_list;

        !self = $self;
        dir_list.reg :select {!(wid, idx) = @;
            if idx == 0 {
                self.navigate_parent[];
                return $n;
            };

            !list_idx = idx - 1;
            self.navigate_dir_index list_idx;
        };

        grp
    },
    navigate_parent = {
        if len[$data.path_stack] > 0 {
            $data.cur_path = std:pop $data.path_stack;
            $self.update[];
        };
    },
    navigate_dir_index = {!(index) = @;
        !dir = $data.directories.(index);

        std:push $data.path_stack $data.cur_path;
        $data.cur_path = dir.0;
        $self.update[];
    },
    update = {
        !dirs = $[];
        !files = $[];
        !_ = std:fs:read_dir $data.cur_path {!(ent) = @;
            std:displayln "ENT:" ent;
            match ent.type
                :f => {
                    if ent.name &> $r/*.wav$$/ {
                        std:push files $p(ent.path, ent.name);
                    };
                }
                :d => {
                    std:push dirs $p(ent.path, ent.name);
                };
            $f
        };
        $data.directories = dirs;
        $data.files = files;

        $data.file_list_data.clear[];
        iter f files \$data.file_list_data.push f.1;

        $data.dir_list_data.clear[];
        $data.dir_list_data.push ".. (parent)";
        iter d dirs \$data.dir_list_data.push ~ d.1 "/";
    },
};

!fsel = FileSelector.new[];
fsel.update[];

#!list = styling:new_widget :file_list;
#list.set_ctrl :list $["a", "b", "c", "d"];

std:displayln "PATCHES:" hx:get_directory_patches[];
std:displayln "SAMPLES:" hx:get_directories_samples[];

#list.reg :select { std:displayln "SELECT:" @; };

file_selector_popup.add fsel.build[];

.popup_test = file_selector_popup;

file_selector_popup.popup_at_mouse[];


##################################################################

#sample_list_wichtext.popup_at_mouse[];

popup_layer.add connector_popup;
popup_layer.add mode_selector_popup;
popup_layer.add cell_context_popup;
popup_layer.add matrix_context_popup;
popup_layer.add file_selector_popup;
popup_layer.add help_wichtext;
popup_layer.add debug_panel.build[];
popup_layer.add midi_log_wichtext;
popup_layer.add sample_list_popup;
popup_layer.add dialog_popup;
popup_layer.add blockcode_picker_popup;
popup_layer.add blockcode_context_popup;
popup_layer.add entry_popup;

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

        !param = input_param;
        !param_wid =
            match input_param.name[]
                "trig" => {
                    !trig_btn = styling:new_widget :param_trig_button;
                    trig_btn.set_ctrl :button (ui:txt param.name[]);
                    trig_btn.reg :press {
                        editor.handle_param_trig_btn param :press;
                    };
                    trig_btn.reg :release {
                        editor.handle_param_trig_btn param :release;
                    };
                    trig_btn.reg :hover {
                        editor.handle_hover :param_knob param;
                    };
                    new_param_container trig_btn param.name[];
                }
                {
                    new_hex_knob
                        matrix.create_hex_knob_model[param]
                        param.name[]
                        { editor.handle_hover :param_knob param; };
                };

        knob_row.add param_wid;
        .row_fill += 1;
    };

    iter atom plist.atoms {
        !cur_atom = atom;
        !cont = styling:new_widget :param_container;

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
                :sample => {
                    !sample_btn = styling:new_widget :param_sample_button;
                    !init = matrix.get_param atom;
                    !sample_name = init.audio_sample_name[];
                    sample_btn.set_ctrl :button (ui:txt ~ $p(len[sample_name] - 10, 10) sample_name);
                    !my_atom = atom;
                    sample_btn.reg :click {
                        editor.emit :open_sample_selector my_atom;
                    };
                    sample_btn.reg :hover {
                        editor.handle_hover :param_knob my_atom;
                    };
                    sample_btn
                }
                {
                    !wid = styling:new_widget :atom_wid;
                    wid.set_ctrl :label (ui:txt atom.atom_ui[]);
                    wid
                };

        if is_none[wid] {
            next[];
        };


        !lbl = styling:new_widget :param_label;
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

    !extras = editor.get_current_extra_btns[];
    if extras {
        .knob_row = styling:new_widget :knob_row;
        iter i $i(0, 5) {
            !cont = styling:new_widget :param_container;
            if extras.(i) {
                !(label, cb) = extras.(i);
                !btn = styling:new_widget :param_trig_button;
                btn.set_ctrl :button (ui:txt label);
                btn.reg :click cb;
                cont.add btn;
            };
            knob_row.add cont;
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
left_panel.enable_cache[];


!MONITOR_LABELS = $[
    ui:txt "I1",
    ui:txt "I2",
    ui:txt "I3",
    ui:txt "O1",
    ui:txt "O2",
    ui:txt "O3",
];

editor.reg :update_monitor_labels {!(cell_labels) = @;
    MONITOR_LABELS.0.set cell_labels.t;
    MONITOR_LABELS.1.set cell_labels.tl;
    MONITOR_LABELS.2.set cell_labels.bl;
    MONITOR_LABELS.3.set cell_labels.tr;
    MONITOR_LABELS.4.set cell_labels.br;
    MONITOR_LABELS.5.set cell_labels.b;
};

!create_monitor_widget = {!(index) = @;
    !graph_cont = styling:new_widget :cell_channel_monitor_cont;

    !graph_monitor_style = if index > 2 { :cell_channel_monitor_out } { :cell_channel_monitor_in };

    !graph_mm = styling:new_widget graph_monitor_style;
    !moni_model = matrix.create_graph_minmax_monitor index;
    graph_mm.set_ctrl :graph_minmax $[hx:MONITOR_MINMAX_SAMPLES, moni_model];

    !graph_lbl = styling:new_widget :cell_channel_monitor_label;
    graph_lbl.set_ctrl :label MONITOR_LABELS.(index);

    graph_cont.add graph_mm;
    graph_cont.add graph_lbl;

    graph_cont
};

!moni_panel = styling:new_widget :monitor_panel;
moni_panel.enable_cache[];

!moni_col_inputs  = styling:new_widget :monitor_column;
moni_col_inputs.add ~ create_monitor_widget 0;
moni_col_inputs.add ~ create_monitor_widget 1;
moni_col_inputs.add ~ create_monitor_widget 2;

!moni_col_outputs = styling:new_widget :monitor_column;
moni_col_outputs.add ~ create_monitor_widget 3;
moni_col_outputs.add ~ create_monitor_widget 4;
moni_col_outputs.add ~ create_monitor_widget 5;

moni_panel.add moni_col_inputs;
moni_panel.add moni_col_outputs;

signal_panel.add moni_panel;

root.add left_panel;
root.add app_panel;

!@export on_frame = {!(matrix_records) = @;
    editor.check_pattern_data[];
    # TODO: FIXME:
    unwrap ~ matrix.check_block_function 0;
    matrix.handle_graph_events[];

    iter r matrix_records {
        #d# std:displayln "REC:" r;
        match r
            $p(:matrix_graph, _?) => {
                editor.handle_matrix_graph_change[];
            }
            $p(:midi_event, ev) => {
                editor.handle_midi_event $\.ev;
            };
    };
};

!@export on_click = {!(event) = @;
    if is_some[GLOBAL_CLICK_CB] {
        GLOBAL_CLICK_CB[event];
    };
};

!@export on_driver = {!(driver) = @;
    if is_some[GLOBAL_CLICK_CB] {
        .LAST_LABELS = $@vec iter l driver.list_labels[] {
            $+ l;
        };
    };
};

!@export root = $[root, root_mid, popup_layer];
