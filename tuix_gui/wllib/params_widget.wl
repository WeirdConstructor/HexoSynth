!@wlambda;
!@import std;
!@import vizia;
!@import observable;
!@import node_id;

!@import wid_settings = settings_widget;

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
        iter i 0 => 2 {
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
                    if cur_row_count > 8 {
                        .cur_row      += 1;
                        .cur_row_count = 1;
                    };

                    !col = vizia:new_col $data.rows.(cur_row);
                    !name = param.name[];
                    if name == "trig" &or (name 0 2) == "t_" {
                        !m = $data.m;
                        !p = param;
                        !param_class =
                            if m.param_input_is_used <& p
                                { "node_params_btn_off" }
                                { "node_params_btn" };
                        vizia:new_button
                            col "Trig" {
                                m.set_param p 0.0;
                            } ${
                                class    = param_class,
                                on_press = {|| m.set_param p 1.0; }
                            };
                    } {
                        vizia:new_hexknob
                            col param_model ${ class = "node_params_knob" };
                    };
                    vizia:new_label col name ${ class = "node_params_label" };
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

!create_setting_widget = {!(param, matrix, parent) = @;
    !(min, max) = param.setting_min_max[];
    !items = $@v iter i min => (max + 1) { $+ i => param.format[i] };

    !wid = wid_settings:SettingsWidget.new items;

    !cur = matrix.get_param param;

    wid.listen :changed {!(_, setting_idx) = @;
        std:displayln "SET PARAM" param setting_idx;
        matrix.set_param param setting_idx;
    };

    wid.update cur.i[];
    wid.build parent;
    wid
};

!@export ParamSettingsWidget = ${
    _proto = observable:Observable,
    new = {!(matrix) = @;
        !self = $&& ${
            _proto = $self,
            _data = ${
                m            = matrix,
                node_id      = $n,
                set_wids     = $[],
                root         = $n,
            },
        };
        self
    },
    build = {!(parent) = @;
        $data.root = vizia:new_col parent ${ class = "node_settings" };
    },
    update = {
        !data = $data;

        vizia:remove_all_childs $data.root;

        iter param $data.params.atoms {
            std:displayln param;
            !atom = param.default_value[];

            !col = vizia:new_col $data.root ${ class = "node_settings_col" };

            !wid =
                match param.name[]
                    "keys" => {
                        !m_param = param;
                        !oct_keys =
                            vizia:new_octave_keys col ${
                                on_change = {!(mask) = @;
                                    data.m.set_param m_param mask;
                                }
                            };

                        !cur_val = $data.m.get_param param;
                        vizia:emit_to 0 oct_keys
                            $p(:octave_keys:set_mask, cur_val.i[]);

                        oct_keys
                    }
                    { create_setting_widget param $data.m col };

            std:push $data.set_wids wid;

            vizia:new_label col param.name[] ${ class = "node_settings_label" };
        };

        std:displayln "PARAMS:" $data.params;
    },
    set_node_id = {!(node_id) = @;
        $data.node_id = node_id;
        $data.params  = node_id:param_list node_id;
        $self.update[];
    },
};
