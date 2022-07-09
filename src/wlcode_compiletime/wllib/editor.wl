!@wlambda;
!@import std;
!@import hx;
!@import node_id;
!@import texts wllib:texts;

!format_txt2wichtext = {|1<2| !(txt, lbl) = @;
    !lines = txt $p("\n", 0);
    !title = lines.0;
    !body_lines = $i(1, -1) lines;

    if body_lines.0 != "" {
        std:unshift body_lines "";
    };

    !wichtext_string = $@s iter line body_lines {
        $+ ~ $F "[R:]{}" ~ line $p("]", "]]");
        $+ "\n";
    };

    if is_some[lbl] {
        $F "[R][f20c11:{}] - [f20c15:{}]\n{}"
            lbl
            title
            wichtext_string
    } {
        $F "[R][f20c15:{}]\n{}"
            title
            wichtext_string
    }
};

!EditorClass = ${
    new = {!(matrix) = @;
        !grid_model = matrix.create_grid_model[];
        ${
            _proto = $self,
            _data = ${
                matrix                  = matrix,
                grid_model              = grid_model,
                focus_cell              = $n,
                current_help_node_id    = $n,
                last_active_tracker_id  = 0,
                cbs                     = ${},
            },
        }
    },
    get_grid_model = { $data.grid_model },
    place_new_instance_at = {!(node_id, pos) = @;
        !cell = $data.matrix.get pos;
        !new_node_id = $data.matrix.get_unused_instance_node_id node_id;
        cell.node_id = new_node_id;
        $data.matrix.set pos cell;
        $data.matrix.sync[];
    },
    set_focus_cell = {!(pos) = @;
        $data.grid_model.set_focus_cell pos;
        !cell = $data.matrix.get pos;
        $data.focus_cell = cell;
        $self.emit :change_focus cell;
        $self.emit :update_param_ui;
        std:displayln "FOCUS:" cell cell.node_id cell.node_id.0 cell.node_id.0 != "nop";
        if is_some[cell.node_id] &and cell.node_id.0 != "nop" {
            $self.show_node_id_desc cell.node_id;
        };
        if cell.node_id.0 == "tseq" {
            $data.last_active_tracker_id = cell.node_id.1;
            $self.emit :pattern_editor_set_data $[
                $data.last_active_tracker_id,
                $[
                    6,
                    $data.matrix.create_pattern_data_model
                        $data.last_active_tracker_id,
                    $n
                ]
            ];
        };
    },
    get_current_graph_fun = {
        $data.matrix.create_graph_model $data.focus_cell.node_id
    },
    get_current_param_list = {
        if is_none[$data.focus_cell]
            { $[] }
            { node_id:param_list $data.focus_cell.node_id}
    },
    reg = {!(ev, cb) = @;
        if is_none[$data.cbs.(ev)] {
            $data.cbs.(ev) = $[cb];
        } {
            std:push $data.cbs.(ev) cb;
        }
    },
    emit = {!ev = _;
        !args     = $@vec iter a 1 => len[@] { $+ @.(a) };
        !call_cbs = $@vec iter cb $data.cbs.(ev) { $+ cb };
        iter cb call_cbs {
            cb[[args]];
        };
    },
    matrix_set_connection_by_io_names = {!(src, dst, out_name, in_name) = @;
        std:displayln "SET CON:" @;

        !adj = hx:pos_are_adjacent src dst;
        if is_none[adj] { return $n; };
        !edge_idx     = adj.as_edge[];
        !dst_edge_idx = adj.flip[].as_edge[];


        !src_cell = $data.matrix.get src;
        !dst_cell = $data.matrix.get dst;

        src_cell.ports.(edge_idx) = out_name;
        dst_cell.ports.(dst_edge_idx) = in_name;

        $data.matrix.set src src_cell;
        $data.matrix.set dst dst_cell;
#        !out_idx = node_id:out_name2idx src_cell.node_id out_name;
#        !in_idx  = node_id:inp_name2idx dst_cell.node_id in_name;
    },
    matrix_clear_connection = {!(src, dst) = @;
        !adj = hx:pos_are_adjacent src dst;
        if is_none[adj] { return $n; };
        !edge_idx     = adj.as_edge[];
        !dst_edge_idx = adj.flip[].as_edge[];

        !src_cell = $data.matrix.get src;
        !dst_cell = $data.matrix.get dst;

        std:displayln "CLEAR CON" src src_cell;
        std:displayln "CLEAR CON" dst dst_cell;

        src_cell.ports.(edge_idx)     = $n;
        dst_cell.ports.(dst_edge_idx) = $n;
        $data.matrix.set src src_cell;
        $data.matrix.set dst dst_cell;
    },
    matrix_apply_change = {!(cb) = @;
        !matrix = $data.matrix;
        matrix.save_snapshot[];

        !change_text = cb matrix;
        match change_text
            ($error v) => {
                std:displayln "ERROR1:" $\.v;
                matrix.restore_snapshot[];
                return $n;
            };

        !check_res = matrix.check[];
        if check_res {
            matrix.sync[];
            $t
        } {
            matrix.restore_snapshot[];
            match check_res
                ($error v) => {
                    std:displayln change_text "ERROR2:" $\.v;
                    return $f;
                };
            $t
        };
    },
    handle_drag_gesture = {!(src, dst, btn) = @;
        !this = $self;
        !adj = hx:pos_are_adjacent src dst;

        std:displayln "GRID DRAG:" adj;

        !src_cell   = $data.matrix.get src;
        !dst_cell   = $data.matrix.get dst;
        !src_exists = src_cell.node_id.0 != "nop";
        !dst_exists = dst_cell.node_id.0 != "nop";

        std:displayln "s:" src_exists dst_exists;

        if src_exists &and not[dst_exists] {
            if btn == :right {
                !move_ok = this.matrix_apply_change {!(matrix) = @;
                    matrix.set src dst_cell;
                    matrix.set dst src_cell;
                };

                if move_ok {
                    $self.set_focus_cell dst;
                };
            } {
                !clust = hx:new_cluster[];
                !move_ok = this.matrix_apply_change {!(matrix) = @;
                    clust.add_cluster_at matrix src;
                    clust.remove_cells matrix;
                    !path = hx:dir_path_from_to src dst;
                    _? ~ clust.move_cluster_cells_dir_path path;
                    _? ~ clust.place matrix;
                    $true
                };

                if move_ok {
                    $self.set_focus_cell dst;
                };
            };

            return $n;
        };

        if is_some[adj] {
            if adj.is_input[] {
                .(src,      dst)      = $p(dst,      src);
                .(src_cell, dst_cell) = $p(dst_cell, src_cell);
                .adj = adj.flip[];
            };

            !edge_idx     = adj.as_edge[];
            !dst_edge_idx = adj.flip[].as_edge[];

            if dst_cell.node_id.0 == "nop" \return $n;
            if src_cell.node_id.0 == "nop" \return $n;

            #d# std:displayln "CELLS:" src_cell dst_cell;
            #d# std:displayln "PORTS:"
            #d#     src_cell.ports.(edge_idx)
            #d#     dst_cell.ports.(dst_edge_idx);

            !src_outs = node_id:out_list   src_cell.node_id;
            !dst_ins  = node_id:param_list dst_cell.node_id;

            !out_name = src_cell.ports.(edge_idx);
            !in_name  = dst_cell.ports.(dst_edge_idx);

            !connection =
                if is_some[out_name] &and is_some[in_name] {
                    !out_idx  = node_id:out_name2idx src_cell.node_id out_name;
                    !in_idx   = node_id:inp_name2idx dst_cell.node_id in_name;
                    $i(out_idx, in_idx)
                } {
                    $none
                };

            .dst_ins =
                $@vec iter inp dst_ins.inputs {
                    $+ $p(inp.name[],
                          not $data.matrix.param_input_is_used[inp])
                };
            .src_outs = $@vec iter out src_outs { $+ out.1 };

            #d# std:displayln "INS"  dst_ins;
            #d# std:displayln "OUTS" src_outs;

            $self.emit :setup_edit_connection
                src_cell dst_cell
                src_outs dst_ins
                connection {!(con) = @;
                    this.matrix_apply_change {!(matrix) = @;
                        if is_none[con] {
                            this.matrix_clear_connection src dst;
                            return ~ $F "Clear connection between {} and {}"
                                src_cell.node_id dst_cell.node_id;
                        } {
                            this.matrix_set_connection_by_io_names
                                src dst
                                src_outs.(con.0)
                                dst_ins.(con.1).0;
                            return ~ $F "Set connection between {} output {} and {} input {}"
                                src_cell.node_id
                                src_outs.(con.0)
                                dst_cell.node_id
                                dst_ins.(con.1).0;
                        };
                    };
                };
        };
    },
    show_param_id_desc = {!(param_id) = @;
        !(node_id, idx) = param_id.as_parts[];
        !info = node_id:info node_id;
        !help = info.in_help idx;

        $self.emit
            :update_status_help_text
            ~ format_txt2wichtext help;
    },
    show_node_id_desc = {|1<2| !(node_id, source) = @;
        !info = node_id:info node_id;
        !desc = info.desc[];

        !node_lbl = node_id:label[node_id];

        !text = format_txt2wichtext desc node_lbl;
        if source == :picker {
            .text = text "\n[c17f18:(drag the button to place!)]";
        };

        $data.current_help_node_id = info;
        $self.emit :update_status_help_text text;
    },
    show_color_info = {
        !text = $@s iter clr 0 => 19 { $+ ~ $F"[c{}:XX {:02!i} XX]\n" clr clr; };
        $self.emit
            :update_status_help_text
            text;
    },
    handle_hover = {!(where, arg1) = @;
        match where
            :node_picker => {
                $self.show_node_id_desc arg1 :picker;
            }
            :param_knob => {
                $self.show_param_id_desc arg1;
            };
    },
    handle_node_help_btn = {
        if is_some[$data.current_help_node_id] {
            $self.emit :show_main_help $data.current_help_node_id.help[];
        };
    },
    handle_matrix_graph_change = {
        $self.set_focus_cell $data.focus_cell.pos;
    },
    handle_top_menu_click = {!(button_tag) = @;
        match button_tag
            :help       => { $self.emit :show_main_help texts:help; }
            :tracker    => { $self.emit :show_main_help texts:tracker; }
            :about      => { $self.emit :show_main_help texts:about; }
    },
    handle_param_trig_btn = {!(param, action) = @;
        match action
            :press   => { $data.matrix.set_param param 1.0 }
            :release => { $data.matrix.set_param param 0.0 }
    },
    check_pattern_data = {
        $data.matrix.check_pattern_data $data.last_active_tracker_id;
    },
};

!@export EditorClass = EditorClass;
