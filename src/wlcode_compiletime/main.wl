!@wlambda;
!@import std;
!@import ui;
!@import hx;
!@import node_id;
!@import styling wllib:styling;
!@import editor wllib:editor;
!@import tests wllib:tests;

!@export init = {!(ARGV) = @;
    std:displayln "ARGV:" ARGV;
    !do_test = $f;
    !test_match = $n;
    iter arg ARGV {
        match arg
            $r/$^test$$/        => { .do_test = $t; }
            (x $r/$^-(^$*?)$$/) => { .test_match = $\.x.1; }
    };

    if do_test {
        tests:install test_match;
    };
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
    std:displayln "CAT MAP:" cat_map;
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

!cell_context_popup = styling:new_widget :popup_menu;
cell_context_popup.auto_hide[];

!add_context_menu_item = {!(menu, label, callback) = @;
    !btn = styling:new_widget :cell_context_item;
    btn.set_ctrl :button (ui:txt label);
    btn.reg :click {
        callback[];
        menu.hide[];
    };

    menu.add btn;
};

!CONTEXT_CELL = $n;

add_context_menu_item cell_context_popup "Remove" {
    editor.remove_cell CONTEXT_CELL.pos;
};

!setup_grid_widget = {!(matrix, click_cb) = @;
    !grid = styling:new_widget :matrix_grid;
    grid.set_ctrl :grid editor.get_grid_model[];

    grid.reg :click {!(widget, event) = @;
        match event.button
            :left => {
                editor.set_focus_cell $i(@.1.x, @.1.y);
                click_cb[];
            }
            :right => {
                .CONTEXT_CELL = editor.get_context_cell $i(event.x, event.y);
                cell_context_popup.popup_at_mouse[];
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
        :left
        add_node_panel_inner;

!left_panel_dummy = styling:new_widget :main_panel;
root_mid.add left_panel_dummy;

!right_container = styling:new_widget :right_mid_cont;
right_container.add add_node_panel;

!top_menu_button_bar = styling:new_widget :top_float_menu;


!help_button = styling:new_button_with_label :button_float_menu "Help" {
    editor.handle_top_menu_click :help;
};
top_menu_button_bar.add help_button;

!about_button = styling:new_button_with_label :button_float_menu "About" {
    editor.handle_top_menu_click :about;
};
top_menu_button_bar.add about_button;

!save_btn = styling:new_button_with_label :button_float_menu "Save" {
    matrix.save_patch "init.hxy";
};
top_menu_button_bar.add save_btn;
!load_btn = styling:new_button_with_label :button_float_menu "Load" {
    matrix.load_patch "init.hxy";
};
top_menu_button_bar.add load_btn;

!color_btn = styling:new_button_with_label :button_float_menu "_C" {
    editor.show_color_info[];
};
top_menu_button_bar.add color_btn;


right_container.add top_menu_button_bar;

root_mid.add right_container;

!right_panel_container =
    styling:new_widget :right_util_panel_cont;
!right_panel =
    new_slide_panel
        :right_slide_panel
        :right
        right_panel_container;

!tracker_help = styling:new_button_with_label :top_right_help_btn "?" {
    editor.handle_tracker_help_btn[];
};

!patedit = styling:new_widget :pattern_editor;
!patedit_label = styling:new_widget :pattern_editor_label;
!patedit_label_data = ui:txt "TSeq 0";
patedit_label.set_ctrl :label patedit_label_data;

patedit_label.add tracker_help;

!patdata = matrix.create_pattern_data_model 0;
!fbdummy = ui:create_pattern_feedback_dummy[];
patedit.set_ctrl :pattern_editor $[6, patdata, fbdummy];

editor.reg :pattern_editor_set_data {!(tracker_id, data) = _;
    patedit_label_data.set ($F"TSeq {}" tracker_id);
    if is_none[data.2] {
        data.2 = ui:create_pattern_feedback_dummy[];
    };
    patedit.set_ctrl :pattern_editor data;
};
editor.set_active_tracker $p(:tseq, 0);

!patedit_container = styling:new_widget :pattern_editor_container;
patedit_container.add patedit_label;
patedit_container.add patedit;

right_panel_container.add patedit_container;

root_mid.add right_panel;


!left_panel = styling:new_widget :main_panel;

!param_panel = styling:new_widget :param_panel;
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
help_wichtext.change_layout ${
    position_type = :self,
    width         = :pixels  => 720,
    min_width     = :pixels  => 720,
    height        = :stretch => 1,
    left          = :stretch => 1,
    right         = :stretch => 1,
    visible       = $f,
};
!wtd_help = ui:wichtext_simple_data_store[];
wtd_help.set_text "Help!";
help_wichtext.set_ctrl :wichtext wtd_help;
help_wichtext.auto_hide[];

editor.reg :show_main_help {!(text) = @;
    wtd_help.set_text text;
    help_wichtext.show[];
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

#sample_list_wichtext.popup_at_mouse[];

popup_layer.add connector_popup;
popup_layer.add mode_selector_popup;
popup_layer.add cell_context_popup;
popup_layer.add help_wichtext;
popup_layer.add sample_list_popup;

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
                    trig_btn
                }
                {
                    !knob = styling:new_widget :knob;
                    !knob_model = matrix.create_hex_knob_model input_param;
                    knob.set_ctrl :knob knob_model;
                    knob.reg :hover {
                        editor.handle_hover :param_knob param;
                    };
                    knob
                };

        !lbl = styling:new_widget :param_label;
        lbl.set_ctrl :label (ui:txt input_param.name[]);

        cont.add param_wid;
        cont.add lbl;
        knob_row.add cont;
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

    !graph_mm = styling:new_widget :cell_channel_monitor;
    !moni_model = matrix.create_graph_minmax_monitor index;
    graph_mm.set_ctrl :graph_minmax $[hx:MONITOR_MINMAX_SAMPLES, moni_model];

    !graph_lbl = styling:new_widget :cell_channel_monitor_label;
    graph_lbl.set_ctrl :label MONITOR_LABELS.(index);

    graph_cont.add graph_mm;
    graph_cont.add graph_lbl;

    graph_cont
};

!moni_panel = styling:new_widget :monitor_panel;

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
root.add grid;

!@export on_frame = {!(matrix_records) = @;
    editor.check_pattern_data[];

    iter r matrix_records {
        #d# std:displayln "REC:" r;
        match r
            $p(:matrix_graph, _?) => {
                editor.handle_matrix_graph_change[];
            };
    };
};

!@export root = $[root, root_mid, popup_layer];
