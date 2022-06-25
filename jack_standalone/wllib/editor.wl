!@wlambda;
!@import std;
!@import hx;
!@import node_id;

!EditorClass = ${
    new = {!(matrix) = @;
        !grid_model = matrix.create_grid_model[];
        ${
            _proto = $self,
            _data = ${
                matrix     = matrix,
                grid_model = grid_model,
                focus_cell = $n,
                cbs        = ${},
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

        !check_res = matrix.check[];
        if check_res {
            matrix.sync[]
        } {
            matrix.restore_snapshot[];
            match check_res
                ($error v) => {
                    std:displayln change_text "ERRROR:" $\.v;
                };
        };
    },
    handle_drag_gesture = {!(src, dst) = @;
        !adj = hx:pos_are_adjacent src dst;

        std:displayln "GRID DRAG:" adj;

        if is_some[adj] {
            !edge_idx     = adj.as_edge[];
            !dst_edge_idx = adj.flip[].as_edge[];

            !src_cell     = $data.matrix.get src;
            !dst_cell     = $data.matrix.get dst;

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

            .dst_ins  = $@vec iter inp dst_ins.inputs { $+ inp.name[] };
            .src_outs = $@vec iter out src_outs { $+ out.1 };

            #d# std:displayln "INS"  dst_ins;
            #d# std:displayln "OUTS" src_outs;

            !this   = $self;
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
                                dst_ins.(con.1);
                            return ~ $F "Set connection between {} output {} and {} input {}"
                                src_cell.node_id
                                src_outs.(con.0)
                                dst_cell.node_id
                                dst_ins.(con.1);
                        };
                    };
                };
        };
    },
};

!@export EditorClass = EditorClass;
