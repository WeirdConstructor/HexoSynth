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
        $+ ~ $F "[R:][f13:]{}" ~ line $p("]", "]]");
        $+ "\n";
    };

    if is_some[lbl] {
        $F "[R][f16c11:{}] - [f16c15:{}]\n{}"
            lbl
            title
            wichtext_string
    } {
        $F "[R][f16c15:{}]\n{}"
            title
            wichtext_string
    }
};

!is_empty_cell = {
    is_none[_] &or is_none[_.node_id] &or _.node_id.0 == "nop"
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
                matrix_in_apply         = $f,
                matrix_center           = $i(4, 4),
                cbs                     = ${},
            },
        }
    },
    get_grid_model = { $data.grid_model },
    place_new_instance_at = {|2<3| !(node_id, pos, cb) = @;
        $self.matrix_apply_change {!(matrix) = @;
            !cell = matrix.get pos;
            !new_node_id = matrix.get_unused_instance_node_id node_id;
            cell.node_id = new_node_id;
            matrix.set pos cell;
            .cell = matrix.get pos;

            if cb \cb matrix cell;
        };
    },
    set_active_tracker = {!(node_id) = @;
        $data.last_active_tracker_id = node_id.0;
        $self.emit :pattern_editor_set_data $[
            $data.last_active_tracker_id,
            $[
                6,
                $data.matrix.create_pattern_data_model
                    $data.last_active_tracker_id,
                $data.matrix.create_pattern_feedback_model node_id,
            ]
        ];
    },
    set_focus_cell = {!(pos) = @;
        $data.grid_model.set_focus_cell pos;
        !cell = $data.matrix.get pos;
        $data.focus_cell = cell;
        $self.emit :change_focus cell;
        $self.emit :update_param_ui;
        #d# std:displayln "FOCUS:" cell cell.node_id cell.node_id.0 cell.node_id.0 != "nop";

        if not[is_empty_cell[cell]] {
            $self.show_node_id_desc cell.node_id;
            $data.matrix.monitor_cell cell;
            $self.emit :update_monitor_labels
                ~ $data.matrix.cell_edge_labels pos;
        };

        if cell.node_id.0 == "tseq" {
            $self.set_active_tracker cell.node_id;
        };
    },
    set_grid_center = {!(pos) = @;
        $data.matrix_center = pos;
    },
    get_current_graph_fun = {
        $data.matrix.create_graph_model $data.focus_cell.node_id
    },
    get_current_param_list = {
        if is_none[$data.focus_cell]
            { $[] }
            { node_id:param_list $data.focus_cell.node_id}
    },
    get_context_cell = {!(pos) = @; $data.matrix.get pos },
    remove_cell = {!(pos) = @;
        $self.matrix_apply_change {!(matrix) = @;
            $data.matrix.set pos $n;
            $true
        };
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
    # Moves a single cell from position `src` to position `dst`. Trying to keep the connection
    # to adjacent cells alive.
    matrix_move_single_cell = {!(src, dst) = @;
        !adj = hx:pos_are_adjacent src dst;

        $self.matrix_apply_change {!(matrix) = @;
            !src_cell = matrix.get src;
            !dst_cell = matrix.get dst;
            !set_other = $[];

            if is_some[adj] {
                !connections = matrix.get_connections src;

                iter con connections {
                    !adj_other = hx:pos_are_adjacent dst con.other.pos;
                    if is_some[adj_other] {
                        # Make other cell swap the ports:
                        !other_cell = matrix.get con.other.pos;
                        !edge = adj_other.flip[].as_edge[];
                        other_cell.ports.(edge) = con.other.port;
                        other_cell.ports.(con.other.dir.as_edge[]) = $n;
                        std:push set_other $p(con.other.pos, other_cell);

                        # Swap ports in moved cell:
                        !src_edge = adj_other.as_edge[];
                        src_cell.ports.(con.center.dir.as_edge[]) = $n;
                        src_cell.ports.(src_edge) = con.center.port;
                    };
                };
            };

            matrix.set src dst_cell;
            matrix.set dst src_cell;
            iter set set_other {
                matrix.set set.0 set.1;
            };
        }
    },
    matrix_split_cluster_at = {!(pos_a, pos_b) = @;
        !adj = hx:pos_are_adjacent pos_a pos_b;
        if is_none[adj] \return $n;

        $self.matrix_apply_change {!(matrix) = @;
            !cluster = hx:new_cluster[];
            cluster.ignore_pos pos_a;
            cluster.add_cluster_at matrix pos_b;

            cluster.remove_cells matrix;
            cluster.move_cluster_cells_dir_path $[adj];
            cluster.place matrix;
        };
    },
    matrix_apply_change = {!(cb) = @;
        !matrix = $data.matrix;
        if $data.matrix.in_apply {
            return ~ cb matrix;
        };
        matrix.save_snapshot[];

        $data.matrix.in_apply = $t;
        !change_text = cb matrix;
        $data.matrix.in_apply = $f;

        match change_text
            ($error v) => {
                std:displayln "ERROR1:" $\.v;
                matrix.restore_snapshot[];
                $data.matrix.in_apply = $f;
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
    open_connection_dialog_for = {!(src, dst) = @;
        !adj = hx:pos_are_adjacent src dst;
        if is_none[adj] \return $n;

        if adj.is_input[] {
            .(src, dst) = $p(dst, src);
            .adj = adj.flip[];
        };

        !src_cell   = $data.matrix.get src;
        !dst_cell   = $data.matrix.get dst;
        !src_exists = not ~ is_empty_cell src_cell;
        !dst_exists = not ~ is_empty_cell dst_cell;
        if not[src_exists] &or not[dst_exists] \return $n;

        !edge_idx     = adj.as_edge[];
        !dst_edge_idx = adj.flip[].as_edge[];

        if is_empty_cell[dst_cell] \return $n;
        if is_empty_cell[src_cell] \return $n;

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
        !this = $self;

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
    },
    handle_drag_gesture = {!(src, dst, btn) = @;
        !adj = hx:pos_are_adjacent src dst;

        std:displayln "GRID DRAG:" adj;

        !src_cell   = $data.matrix.get src;
        !dst_cell   = $data.matrix.get dst;
        !src_exists = not ~ is_empty_cell src_cell;
        !dst_exists = not ~ is_empty_cell dst_cell;

        std:displayln "s:" src_exists dst_exists;

        if src_exists &and not[dst_exists] {
            if btn == :right {
                !move_ok = $self.matrix_move_single_cell src dst;
                if move_ok { $self.set_focus_cell dst; };
            } {
                !clust = hx:new_cluster[];
                !move_ok = $self.matrix_apply_change {!(matrix) = @;
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

        if is_some[adj] &and src_exists &and dst_exists {
            if btn == :left {
                $self.open_connection_dialog_for src dst;
            } {
                $self.matrix_split_cluster_at src dst;
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
        if is_none[info] \return $n;

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
    handle_picker_help_btn = {
        $self.emit :show_main_help texts:picker;
    },
    handle_node_help_btn = {
        if is_some[$data.current_help_node_id] {
            $self.emit :show_main_help $data.current_help_node_id.help[];
        };
    },
    handle_tracker_help_btn = {
        !tseq_help = (node_id:info $p(:tseq, 0)).help[];

        $self.emit :show_main_help 
            ~ std:str:cat
                texts:tracker
                "\n----------------\n\n"
                tseq_help;
    },
    handle_matrix_graph_change = {
        $self.set_focus_cell $data.focus_cell.pos;
    },
    handle_top_menu_click = {!(button_tag) = @;
        match button_tag
            :help       => { $self.emit :show_main_help texts:help; }
            :about      => { $self.emit :show_main_help texts:about; }
    },
    handle_param_trig_btn = {!(param, action) = @;
        match action
            :press   => { $data.matrix.set_param param 1.0 }
            :release => { $data.matrix.set_param param 0.0 }
    },
    handle_picker_node_id_click = {!(node_id, btn) = @;
        !focus_pos = $data.focus_cell.pos;
        if is_some[focus_pos] &and not[is_empty_cell[$data.matrix.get focus_pos]] {
            !new_cell_dir = match btn :left => :B :right => :T;
            !inputs = $data.matrix.find_all_adjacent_free focus_pos new_cell_dir;
            !focus_cell = $data.matrix.get focus_pos;

            #d# std:displayln "FREE INPUTS:" inputs focus_cell;

            if len[inputs] > 0 {
                !free_input = inputs.(std:rand len[inputs]);
                !free_dir = free_input.dir;

                $self.place_new_instance_at
                    node_id
                    free_input.pos
                    {!(matrix, new_cell) = @;
                        #d# std:displayln "FREEE DIR" free_dir focus_pos new_cell.pos;
                        if free_dir.is_output[] {
                            !unused = $data.matrix.find_unused_inputs new_cell.node_id;
                            !outputs = node_id:out_list focus_cell.node_id;

                            if len[outputs] > 0 &and len[unused] > 0 {
                                $self.matrix_set_connection_by_io_names
                                    focus_pos
                                    new_cell.pos
                                    outputs.0.1
                                    unused.(0).name[];
                            };
                            #d# std:displayln "PLACED! unusued inputs=" unused unused.(0).name[];
                        } {
                            !unused = $data.matrix.find_unused_inputs focus_cell.node_id;
                            !outputs = node_id:out_list new_cell.node_id;

                            if len[outputs] > 0 &and len[unused] > 0 {
                                $self.matrix_set_connection_by_io_names
                                    new_cell.pos
                                    focus_pos
                                    outputs.0.1
                                    unused.(0).name[];
                            };
                        };
                    };

                $self.set_focus_cell free_input.pos;
            };
            return $n;
        };

        # otherwise take the center, or some free cell around there and insert
        # unconnected.

        !pos = $i(
           $data.matrix_center.0 - 2,
           $data.matrix_center.1 - 2
        );

        !cell = $data.matrix.get pos;
        if not[is_empty_cell[cell]] {
            !free_dir = match btn :left => :B :right => :T;
            !free = $data.matrix.find_all_adjacent_free focus_pos free_dir;

            if len[free] > 0 {
                !idx = std:rand len[free];
                .pos = free.(idx).pos;
            } {
                # TODO: Display error dialog if none free found!
                return $n;
            };
        };

        # TODO: use the focus cell as source, if the focus cell is not empty!

        !matrix_size = $data.matrix.size[];
        if pos.0 >= matrix_size.0 &or pos.1 >= matrix_size.1 {
            return $n;
        };

        .cell = $data.matrix.get pos;
        if is_empty_cell[cell] {
            $self.place_new_instance_at node_id pos;
            $self.set_focus_cell pos;
        };
    },
    check_pattern_data = {
        $data.matrix.check_pattern_data $data.last_active_tracker_id;
    },
};

!@export EditorClass = EditorClass;
