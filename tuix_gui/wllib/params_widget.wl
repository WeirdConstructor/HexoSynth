!@wlambda;
!@import std;
!@import vizia;
!@import observable;
!@import node_id;

!@export ParamsWidget = ${
    _proto = observable:Observable,
    new = {!(matrix) = @;
        !self = $&& ${
            _proto = $self,
            _data = ${
                m            = matrix,
                node_id      = $n,
                root         = $n,
                rows         = $[],
            },
        };
        self
    },
    build = {!(parent) = @;
        $data.root = vizia:new_col parent ${ class = "node_params" };
        iter i 0 => 4 {
            $data.rows.(i) = vizia:new_row $data.root;
        };
    },
    update = {
        iter row $data.rows {
            vizia:remove_all_childs row;
        };

        !cur_row = 0;
        !cur_row_count = 0;
        iter param $data.params.inputs {
            std:displayln param;
            !atom = param.default_value[];

            match atom.type_str[]
                :param => {
                    !param_model = $data.m.create_hex_knob_model param;

                    .cur_row_count += 1;
                    if cur_row_count > 4 {
                        .cur_row      += 1;
                        .cur_row_count = 1;
                    };

                    !col = vizia:new_col $data.rows.(cur_row);
                    vizia:new_hexknob
                        col param_model ${ class = "node_params_knob" };
                    vizia:new_label
                        col param.name[] ${ class = "node_params_label" };
                };
        };

        std:displayln "PARAMS:" $data.params;
    },
    set_node_id = {!(node_id) = @;
        $data.node_id = node_id;
        $data.params  = node_id:param_list node_id;
        $self.update[];
    },
};
