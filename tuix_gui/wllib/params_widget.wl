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
                matrix       = matrix,
                node_id      = $n,
                root         = $n,
                rows         = $[],
            },
        };
        self
    },
    build = {!(parent) = @;
        $data.root = vizia:new_col parent ${ class = "node_params" };
        iter i 0 => 5 {
            $data.rows.(i) = vizia:new_row $data.root;
        };
    },
    update = {
        iter row $data.rows {
            vizia:remove_all_childs row;
        };

        std:displayln "PARAMS:" $data.params;
    },
    set_node_id = {!(node_id) = @;
        $data.node_id = node_id;
        $data.params  = node_id:param_list node_id;
        $self.update[];
    },
};
