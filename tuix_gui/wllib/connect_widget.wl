!@wlambda;
!@import std;
!@import vizia;
!@import hx;
!@import node_id;
!@import observable;

!get_cell_edge_idxes = {!(cell_o, cell_i) = @;
    !adj_dir = hx:pos_are_adjacent cell_o.pos cell_i.pos;
    !out_edge = adj_dir.as_edge[];
    !in_edge  = adj_dir.flip[].as_edge[];
    out_edge => in_edge
};

!ConnectWidget = ${
    _proto = observable:Observable,
    new = {!(matrix) = @;
        !self = $&& ${
            _proto = $self,
            _data = ${
                m            = matrix,
                cell_o       = $n,
                cell_i       = $n,
                root         = $n,
            },
        };

        self.observable_init[];

        self
    },
    build = {!(parent) = @;
        !self = $w& $self;
        $data.parent = parent;
        $data.root = vizia:new_col parent ${ };
        $data.con = vizia:new_connector $data.root ${
            on_change = {!(out_idx, in_idx) = @;
                !(cell_o, cell_i) = $p(
                    self._data.cell_o,
                    self._data.cell_i
                );
                !e_idxs =
                    get_cell_edge_idxes cell_o cell_i;

                !onid = cell_o.node_id;
                !inid = cell_i.node_id;

                .cell_o = std:copy cell_o;
                cell_o.ports = std:copy cell_o.ports;
                .cell_i = std:copy cell_i;
                cell_i.ports = std:copy cell_i.ports;

                cell_o.ports.(e_idxs.0) =
                    (node_id:out_idx2name onid out_idx);
                cell_i.ports.(e_idxs.1) =
                    (node_id:inp_idx2name inid in_idx);

                # std:displayln "EEEE" e_idxs;
                # std:displayln "CHANGED CELLS O=" cell_o;
                # std:displayln "CHANGED CELLS I=" cell_i;

                self.emit :changed_cells cell_o cell_i;
            },
        };
    },
    setup = {!(cell_o, cell_i) = @;
        $data.cell_o = cell_o;
        $data.cell_i = cell_i;
        $self.update[];
    },
    update = {
        !out_params = node_id:out_list   $data.cell_o.node_id;
        !in_params  = node_id:param_list $data.cell_i.node_id;

        !max_rows = len[out_params];
        if len[in_params.inputs] > max_rows {
            .max_rows = len[in_params.inputs];
        };

        vizia:set_height $data.parent (float[max_rows] * 30.0) => :px;

        !e_idxs = get_cell_edge_idxes $data.cell_o $data.cell_i;

        !out_idx =
            node_id:out_name2idx
                $data.cell_o.node_id
                $data.cell_o.ports.(e_idxs.0);
        !in_idx =
            node_id:inp_name2idx
                $data.cell_i.node_id
                $data.cell_i.ports.(e_idxs.1);

        std:displayln "OC" e_idxs.0 $data.cell_o;
        std:displayln "IC" e_idxs.1 $data.cell_i;
        std:displayln "OI" out_idx;
        std:displayln "II" in_idx;

        if is_some[out_idx] &and is_some[in_idx] {
            vizia:emit_to 0 $data.con
                $[:connector:set_connection, out_idx, in_idx];
        } {
            vizia:emit_to 0 $data.con
                $[:connector:set_connection, $n];
        };

        vizia:emit_to 0 $data.con $[
            :connector:set_items,
            $@vec iter o out_params { $+ $p(o.1, $t) },
            $@vec iter p in_params.inputs { $+ $p(p.name[], $t) },
        ];
    },
};

!popup        = $n;
!popup_entity = $n;
!con_wid      = $n;

!init_global_connect_popup = {!(matrix, on_changed_cells) = @;
    .popup        = vizia:new_popup ${ class = :connect_popup };
    .con_wid      = ConnectWidget.new matrix;
    con_wid.listen :changed_cells {!(ev, o, i) = @;
        if (on_changed_cells o i) {
            vizia:emit_to 0 popup $p(:popup:close, $n);
        };
    };
    con_wid.build popup;
};

!@export init_global_connect_popup = init_global_connect_popup;

!@export connect = {!(cell_o, cell_i) = @;
    con_wid.setup cell_o cell_i;
    vizia:emit_to 0 popup $p(:popup:open_at_cursor, $n);
};

!@export ConnectWidget = ConnectWidget;

