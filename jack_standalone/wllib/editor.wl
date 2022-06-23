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
};

!@export EditorClass = EditorClass;
