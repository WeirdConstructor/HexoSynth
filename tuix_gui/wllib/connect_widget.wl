!@wlambda;
!@import std;
!@import vizia;
!@import node_id;
!@import observable;

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

        self
    },
    build = {!(parent) = @;
        $data.parent = parent;
        $data.root   = vizia:new_col parent ${ };
        $data.con    = vizia:new_connector $data.root ${ };
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

!init_global_connect_popup = {!(matrix) = @;
    .popup        = vizia:new_popup ${ class = :connect_popup };
    .con_wid      = ConnectWidget.new matrix;
    con_wid.build popup;
};

!@export init_global_connect_popup = init_global_connect_popup;

!@export connect = {!(cell_o, cell_i) = @;
    con_wid.setup cell_o cell_i;
    std:displayln "POPUP" cell_o cell_i;
    vizia:emit_to 0 popup $p(:popup:open_at_cursor, $n);
};

!@export ConnectWidget = ConnectWidget;

