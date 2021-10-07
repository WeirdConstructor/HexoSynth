!@wlambda;
!@import std;

!popup        = $n;
!popup_entity = $n;

!init_global_settings_popup = {!(ui) = @;
    .popup        = ui.new_popup ${ class = :setting_popup };
    .popup_entity = ui.new_col popup ${ class = :setting_popup };
};

!@export init_global_settings_popup = init_global_settings_popup;

# * ui       - UI Handle
# * settings - $[index => label, ...]
# * cb       - Callback
!global_settings_popup = {!(ui, settings, cb) = @;
    ui.remove_all_childs popup_entity;

    iter setting settings {
        !idx = setting.0;
        ui.new_button
            popup_entity setting.1 {
                cb _ idx;
                ui.emit_to 0 popup $p(:popup:close, $n);
                _.remove_all_childs popup_entity;
            }
            ${ class = :setting_popup_btn };
    };

    ui.emit_to 0 popup $p(:popup:open_at_cursor, $n);
};

!Observable = ${
    observable_init = {
        $data._callbacks = ${};
    },
    emit = {!args = @;
        !ev_cbs = $data._callbacks.(args.0);
        !new_cbs = $[];
        $data._callbacks.(args.0) = new_cbs;

        iter cb ev_cbs {
            cb[[args]];
            std:push new_cbs cb;
        };
    },
    listen = {!(event, cb) = @;
        if is_none[$data._callbacks.(event)] {
            $data._callbacks.(event) = $[];
        };

        std:push $data._callbacks.(event) cb;
    },
};
!@export Observable = Observable;

!@export SettingsWidget = ${
    # * settings - $[index => label, ...]
    _proto = Observable,
    new = {!(ui, settings) = @;
        !self = $&& ${
            _proto = $self,
            _data = $&& ${
                ui       = ui,
                settings = settings,

                root     = $n,
                pop_btn  = $n,
                cur_idx  = 0,
            },
        };

        self.observable_init[];

        self;
    },
    update_popbtn = {
        $data.ui.set_text
            $data.pop_btn
            $data.settings.($data.cur_idx).1;
    },
    build = {!(parent) = @;
        !self = $w& $self;
        !data = $w& $data;
        !ui   = $data.ui;

        !root_widget = ui.new_row parent;
        $data.root = root_widget;
        $self.dropper = std:to_drop { ui.remove root_widget };

        !popbtn_prev = ui.new_button $data.root "<" {!(ui) = @;
            data.cur_idx -= 1;
            data.cur_idx =
                if data.cur_idx < 0
                    (len[data.settings] - 1)
                    data.cur_idx;
            self.update_popbtn[];
            self.emit :setting_changed data.cur_idx;
        } ${ class = :popup_setting_btn_prev };

        $data.pop_btn = ui.new_button $data.root "popup param" {||
            global_settings_popup ui data.settings {!(ui, idx) = @;
                std:displayln "Choosen:" idx;
                data.cur_idx = idx;
                self.update_popbtn[];
                self.emit :setting_changed data.cur_idx;
            };
        } ${ class = :popup_setting_btn };

        !popbtn_next = ui.new_button $data.root ">" {!(ui) = @;
            data.cur_idx += 1;
            data.cur_idx =
                if data.cur_idx >= len[data.settings]
                    0
                    data.cur_idx;
            self.update_popbtn[];
            self.emit :setting_changed data.cur_idx;
        } ${ class = :popup_setting_btn_next  };

        $self.update_popbtn[];

        $data.root
    },
};
